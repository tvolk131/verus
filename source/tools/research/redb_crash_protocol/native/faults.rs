use redb::{ConcurrencyMode, Database, Durability, ReadableDatabase, ReadableTable, StorageBackend, TableDefinition};
use std::io;
use std::sync::{Arc, Mutex};

#[path = "../source/original/src/tree_store/page_store/xxh3.rs"]
mod checksum;

const TABLE: TableDefinition<u64, &[u8]> = TableDefinition::new("crash_protocol");
const VALUE_LEN: usize = 5120;
const HEADER_LEN: usize = 320;
const SLOT_LEN: usize = 128;
const SLOT_CHECKSUM: usize = 112;
const TXN_ID: usize = 80;

#[derive(Debug, Default)]
struct Image {
    live: Vec<u8>,
    stable: Vec<u8>,
    events: Vec<String>,
    dead: bool,
    armed: bool,
    scenario: String,
    writes: usize,
    syncs: usize,
    fired: bool,
    fixed_layout: bool,
    partial_slot_invalid: Option<bool>,
}

#[derive(Debug, Clone, Default)]
struct Disk(Arc<Mutex<Image>>);

impl Disk {
    fn from_stable(bytes: Vec<u8>) -> Self {
        Self(Arc::new(Mutex::new(Image { live: bytes.clone(), stable: bytes, fixed_layout: true, ..Image::default() })))
    }
    fn alive(state: &Image) -> io::Result<()> {
        if state.dead { Err(io::Error::other("injected permanent backend loss")) } else { Ok(()) }
    }
    fn fail(state: &mut Image, operation: &str, persisted: &str) -> io::Result<()> {
        state.fired = true;
        state.dead = true;
        state.events.push(format!("{{\"kind\":\"failure\",\"operation\":\"{operation}\",\"persisted\":\"{persisted}\"}}"));
        Err(io::Error::other("registered failure injection"))
    }
}

fn slot_valid(bytes: &[u8], slot: usize) -> bool {
    let start = 64 + slot * SLOT_LEN;
    let actual = checksum::hash128_with_seed(&bytes[start..start + SLOT_CHECKSUM], 0);
    let stored = u128::from_le_bytes(bytes[start + SLOT_CHECKSUM..start + SLOT_LEN].try_into().unwrap());
    actual == stored
}

impl StorageBackend for Disk {
    fn len(&self) -> io::Result<u64> { Ok(self.0.lock().unwrap().live.len() as u64) }
    fn read(&self, offset: u64, out: &mut [u8]) -> io::Result<()> {
        let state = self.0.lock().unwrap();
        let start = usize::try_from(offset).map_err(|_| io::Error::other("offset overflow"))?;
        let end = start.checked_add(out.len()).ok_or_else(|| io::Error::other("read overflow"))?;
        out.copy_from_slice(state.live.get(start..end).ok_or_else(|| io::Error::from(io::ErrorKind::UnexpectedEof))?);
        Ok(())
    }
    fn set_len(&self, len: u64) -> io::Result<()> {
        let mut state = self.0.lock().unwrap(); Self::alive(&state)?;
        let len = usize::try_from(len).map_err(|_| io::Error::other("length overflow"))?;
        let old = state.live.len();
        if state.armed && old != len { state.fixed_layout = false; }
        state.live.resize(len, 0);
        state.events.push(format!("{{\"kind\":\"resize\",\"old_len\":{old},\"new_len\":{len}}}"));
        Ok(())
    }
    fn sync_data(&self) -> io::Result<()> {
        let mut state = self.0.lock().unwrap(); Self::alive(&state)?;
        if state.armed { state.syncs += 1; }
        let scenario = state.scenario.clone();
        let at_first = state.armed && state.syncs == 1;
        let at_second = state.armed && state.syncs == 2;
        if (at_first && scenario == "sync1_none") || (at_second && scenario == "sync2_none") {
            return Self::fail(&mut state, "sync", "none");
        }
        if at_first && scenario == "sync1_header" {
            let header = state.live[..HEADER_LEN].to_vec();
            state.stable[..HEADER_LEN].copy_from_slice(&header);
            return Self::fail(&mut state, "sync", "header_only");
        }
        if (at_first && scenario == "sync1_all") || (at_second && scenario == "sync2_all") {
            state.stable = state.live.clone();
            return Self::fail(&mut state, "sync", "all");
        }
        state.stable = state.live.clone();
        state.events.push("{\"kind\":\"sync_success\"}".into());
        Ok(())
    }
    fn write(&self, offset: u64, data: &[u8]) -> io::Result<()> {
        let mut state = self.0.lock().unwrap(); Self::alive(&state)?;
        let start = usize::try_from(offset).map_err(|_| io::Error::other("offset overflow"))?;
        let end = start.checked_add(data.len()).ok_or_else(|| io::Error::other("write overflow"))?;
        let old = state.live.len();
        if end > old { state.live.resize(end, 0); if state.armed { state.fixed_layout = false; } }
        if state.armed { state.writes += 1; }
        let ordinal = state.writes;
        state.events.push(format!("{{\"kind\":\"write\",\"offset\":{offset},\"len\":{},\"ordinal\":{ordinal}}}", data.len()));
        let first = state.armed && state.writes == 1;
        let publication = state.armed && state.syncs == 1 && offset == 0;
        let scenario = state.scenario.clone();
        let mut partial = None;
        if first && scenario == "write1_none" { partial = Some(0); }
        if first && scenario == "write1_partial" {
            if offset != 0 || data.len() != HEADER_LEN { return Err(io::Error::other("first write shape changed")); }
            let secondary = usize::from(data[9] & 1 == 0);
            partial = Some(64 + secondary * SLOT_LEN + SLOT_LEN / 2);
        }
        if publication && scenario == "publish_none" { partial = Some(0); }
        if publication && scenario == "publish_control" { partial = Some(10); }
        if state.armed && state.writes == 2 && scenario == "page1_none" {
            if offset < HEADER_LEN as u64 { return Err(io::Error::other("first page write shape changed")); }
            partial = Some(0);
        }
        if let Some(prefix) = partial {
            state.live[start..start + prefix].copy_from_slice(&data[..prefix]);
            state.stable[start..start + prefix].copy_from_slice(&data[..prefix]);
            if scenario == "write1_partial" {
                let secondary = usize::from(data[9] & 1 == 0);
                state.partial_slot_invalid = Some(!slot_valid(&state.stable, secondary));
            }
            return Self::fail(&mut state, "write", &format!("prefix_{prefix}"));
        }
        state.live[start..end].copy_from_slice(data);
        Ok(())
    }
}

fn open(disk: Disk, abort_repair: bool, callbacks: Arc<Mutex<usize>>) -> Result<Database, redb::DatabaseError> {
    let mut builder = Database::builder();
    builder.set_cache_size(1024 * 1024);
    builder.set_concurrency_mode(ConcurrencyMode::ExclusiveWriter);
    builder.set_repair_callback(move |session| { *callbacks.lock().unwrap() += 1; if abort_repair { session.abort(); } });
    builder.create_with_backend(disk)
}

fn put_pair(db: &Database, two_phase: bool, a: u8, b: u8) -> Result<(), Box<dyn std::error::Error>> {
    let mut txn = db.begin_write()?;
    txn.set_durability(Durability::Immediate)?; txn.set_two_phase_commit(two_phase);
    { let mut table = txn.open_table(TABLE)?;
      table.insert(0u64, vec![a; VALUE_LEN].as_slice())?;
      table.insert(1u64, vec![b; VALUE_LEN].as_slice())?; }
    txn.commit()?; Ok(())
}

fn pair(db: &Database) -> Result<&'static str, Box<dyn std::error::Error>> {
    let txn = db.begin_read()?; let table = txn.open_table(TABLE)?;
    let mut values = Vec::new();
    for key in [0u64, 1u64] {
        let value = table.get_owned(key)?.ok_or("missing record")?;
        let value = value.value();
        if value.len() != VALUE_LEN || !value.iter().all(|x| *x == value[0]) { return Err("torn record".into()); }
        values.push(value[0]);
    }
    match values.as_slice() { [0x11, 0x22] => Ok("old"), [0x33, 0x44] => Ok("new"), _ => Err("mixed records".into()) }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let prior: bool = args.get(1).ok_or("missing prior")?.parse()?;
    let scenario = args.get(2).ok_or("missing scenario")?.clone();
    let callbacks = Arc::new(Mutex::new(0));
    let disk = Disk::default(); let db = open(disk.clone(), false, callbacks.clone())?;
    put_pair(&db, prior, 0x11, 0x22)?;
    if pair(&db)? != "old" { return Err("setup pair".into()); }
    let initial = disk.0.lock().unwrap().stable.clone();
    if (initial[9] & 4 != 0) != prior { return Err("prior mode".into()); }
    if scenario == "corrupt_rewrite" {
        disk.0.lock().unwrap().dead = true; std::mem::forget(db);
        let mut damaged = initial;
        let primary = usize::from(damaged[9] & 1 != 0); let secondary = primary ^ 1;
        let p = 64 + primary * SLOT_LEN; let s = 64 + secondary * SLOT_LEN;
        let valid = damaged[p..p + SLOT_LEN].to_vec();
        damaged[s..s + SLOT_LEN].copy_from_slice(&valid);
        let id = u64::from_le_bytes(damaged[s + TXN_ID..s + TXN_ID + 8].try_into()?);
        damaged[s + TXN_ID..s + TXN_ID + 8].copy_from_slice(&id.checked_add(1).ok_or("txid overflow")?.to_le_bytes());
        let wrong = checksum::hash128_with_seed(&damaged[s..s + SLOT_CHECKSUM], 0) ^ 1;
        damaged[s + SLOT_CHECKSUM..s + SLOT_LEN].copy_from_slice(&wrong.to_le_bytes());
        damaged[9] |= 2; damaged[9] &= !4;
        if slot_valid(&damaged, secondary) { return Err("diagnostic checksum is valid".into()); }
        let raw = damaged[s..s + SLOT_LEN].to_vec();
        let recovered_disk = Disk::from_stable(damaged); let count = Arc::new(Mutex::new(0));
        let outcome = open(recovered_disk.clone(), true, count.clone());
        if !matches!(outcome, Err(redb::DatabaseError::RepairAborted)) { return Err("expected repair abort".into()); }
        let state = recovered_disk.0.lock().unwrap();
        let preserved = state.live[s..s + SLOT_LEN] == raw && !slot_valid(&state.live, secondary);
        let writes = state.events.iter().filter(|x| x.contains("\"kind\":\"write\"")).count();
        let calls = *count.lock().unwrap();
        if !preserved || writes == 0 || calls == 0 { return Err("corruption preservation was vacuous or failed".into()); }
        println!("{{\"prior\":{prior},\"scenario\":\"{scenario}\",\"forced_recovery_prior_mode\":false,\"repair_aborted\":true,\"raw_slot_preserved\":{preserved},\"header_rewrites\":{writes},\"repair_callbacks\":{calls},\"formal_source_proof\":false}}");
        return Ok(());
    }
    { let mut state = disk.0.lock().unwrap(); state.events.clear(); state.scenario = scenario.clone(); state.armed = true; state.fixed_layout = true; }
    let result = put_pair(&db, true, 0x33, 0x44);
    let commit_ok = result.is_ok();
    if let Err(error) = result { eprintln!("public commit error: {error}"); }
    let (bytes, events, fired, fixed, invalid) = { let mut state = disk.0.lock().unwrap(); state.dead = true;
        (state.stable.clone(), state.events.clone(), state.fired, state.fixed_layout, state.partial_slot_invalid) };
    std::mem::forget(db);
    let count = Arc::new(Mutex::new(0)); let reopened = open(Disk::from_stable(bytes), false, count.clone())?;
    let observed = pair(&reopened)?;
    let calls = *count.lock().unwrap();
    let invalid = invalid.map(|x| x.to_string()).unwrap_or("null".into());
    println!("{{\"prior\":{prior},\"scenario\":\"{scenario}\",\"commit_ok\":{commit_ok},\"injection_fired\":{fired},\"fixed_layout\":{fixed},\"reopened_pair\":\"{observed}\",\"repair_callbacks\":{calls},\"partial_slot_invalid\":{invalid},\"events\":[{}],\"formal_source_proof\":false}}", events.join(","));
    Ok(())
}
