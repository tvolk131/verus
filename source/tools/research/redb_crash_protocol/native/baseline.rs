use redb::{ConcurrencyMode, Database, Durability, ReadableDatabase, ReadableTable, StorageBackend, TableDefinition};
use std::io;
use std::sync::{Arc, Mutex};

const TABLE: TableDefinition<u64, &[u8]> = TableDefinition::new("crash_protocol");
const VALUE_LEN: usize = 5120;

#[derive(Debug, Clone)]
enum Event {
    Write { offset: u64, len: usize, old_len: usize, new_len: usize },
    Resize { old_len: usize, new_len: usize },
    Sync { len: usize },
}

impl Event {
    fn json(&self) -> String {
        match self {
            Self::Write { offset, len, old_len, new_len } => format!(
                "{{\"kind\":\"write\",\"offset\":{offset},\"len\":{len},\"old_len\":{old_len},\"new_len\":{new_len}}}"
            ),
            Self::Resize { old_len, new_len } => format!(
                "{{\"kind\":\"resize\",\"old_len\":{old_len},\"new_len\":{new_len}}}"
            ),
            Self::Sync { len } => format!("{{\"kind\":\"sync_success\",\"len\":{len}}}"),
        }
    }
}

#[derive(Debug, Default)]
struct Image {
    live: Vec<u8>,
    stable: Vec<u8>,
    events: Vec<Event>,
    dead: bool,
}

#[derive(Debug, Clone, Default)]
struct Disk(Arc<Mutex<Image>>);

impl Disk {
    fn from_stable(bytes: Vec<u8>) -> Self {
        Self(Arc::new(Mutex::new(Image { live: bytes.clone(), stable: bytes, ..Image::default() })))
    }

    fn alive(state: &Image) -> io::Result<()> {
        if state.dead { Err(io::Error::other("frozen crashed backend")) } else { Ok(()) }
    }
}

impl StorageBackend for Disk {
    fn len(&self) -> io::Result<u64> {
        Ok(self.0.lock().unwrap().live.len() as u64)
    }

    fn read(&self, offset: u64, out: &mut [u8]) -> io::Result<()> {
        let state = self.0.lock().unwrap();
        let start = usize::try_from(offset).map_err(|_| io::Error::other("offset overflow"))?;
        let end = start.checked_add(out.len()).ok_or_else(|| io::Error::other("read overflow"))?;
        let bytes = state.live.get(start..end).ok_or_else(|| io::Error::from(io::ErrorKind::UnexpectedEof))?;
        out.copy_from_slice(bytes);
        Ok(())
    }

    fn set_len(&self, len: u64) -> io::Result<()> {
        let mut state = self.0.lock().unwrap();
        Self::alive(&state)?;
        let new_len = usize::try_from(len).map_err(|_| io::Error::other("length overflow"))?;
        let old_len = state.live.len();
        state.live.resize(new_len, 0);
        state.events.push(Event::Resize { old_len, new_len });
        Ok(())
    }

    fn sync_data(&self) -> io::Result<()> {
        let mut state = self.0.lock().unwrap();
        Self::alive(&state)?;
        state.stable = state.live.clone();
        let len = state.stable.len();
        state.events.push(Event::Sync { len });
        Ok(())
    }

    fn write(&self, offset: u64, data: &[u8]) -> io::Result<()> {
        let mut state = self.0.lock().unwrap();
        Self::alive(&state)?;
        let start = usize::try_from(offset).map_err(|_| io::Error::other("offset overflow"))?;
        let end = start.checked_add(data.len()).ok_or_else(|| io::Error::other("write overflow"))?;
        let old_len = state.live.len();
        if end > old_len { state.live.resize(end, 0); }
        state.live[start..end].copy_from_slice(data);
        let new_len = state.live.len();
        state.events.push(Event::Write { offset, len: data.len(), old_len, new_len });
        Ok(())
    }
}

fn open(disk: Disk) -> Result<Database, redb::DatabaseError> {
    let mut builder = Database::builder();
    builder.set_cache_size(1024 * 1024);
    builder.set_concurrency_mode(ConcurrencyMode::ExclusiveWriter);
    builder.create_with_backend(disk)
}

fn put_pair(db: &Database, two_phase: bool, a: u8, b: u8) -> Result<(), Box<dyn std::error::Error>> {
    let mut txn = db.begin_write()?;
    txn.set_durability(Durability::Immediate)?;
    txn.set_two_phase_commit(two_phase);
    {
        let mut table = txn.open_table(TABLE)?;
        table.insert(0u64, vec![a; VALUE_LEN].as_slice())?;
        table.insert(1u64, vec![b; VALUE_LEN].as_slice())?;
    }
    txn.commit()?;
    Ok(())
}

fn check_pair(db: &Database, a: u8, b: u8) -> Result<(), Box<dyn std::error::Error>> {
    let txn = db.begin_read()?;
    let table = txn.open_table(TABLE)?;
    for (key, expected) in [(0u64, a), (1u64, b)] {
        let value = table.get_owned(key)?.ok_or("missing record")?;
        if value.len() != VALUE_LEN || !value.iter().all(|x| *x == expected) {
            return Err("mixed or wrong record contents".into());
        }
    }
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let prior = match std::env::args().nth(1).as_deref() {
        Some("false") => false,
        Some("true") => true,
        _ => return Err("expected prior mode false or true".into()),
    };
    let disk = Disk::default();
    let db = open(disk.clone())?;
    put_pair(&db, prior, 0x11, 0x22)?;
    check_pair(&db, 0x11, 0x22)?;
    let (event_start, before_len, prior_mode) = {
        let state = disk.0.lock().unwrap();
        (state.events.len(), state.stable.len(), state.stable[9] & 4 != 0)
    };
    if prior_mode != prior { return Err("prior header mode differs from registered profile".into()); }
    put_pair(&db, true, 0x33, 0x44)?;
    let (bytes, events, after_len, final_mode) = {
        let mut state = disk.0.lock().unwrap();
        let image = state.stable.clone();
        let events = state.events[event_start..].to_vec();
        let mode = image[9] & 4 != 0;
        state.dead = true;
        (image, events, state.stable.len(), mode)
    };
    // Process loss must not run a graceful-close commit over the frozen image.
    std::mem::forget(db);
    let reopened = open(Disk::from_stable(bytes))?;
    check_pair(&reopened, 0x33, 0x44)?;
    let fixed_layout = before_len == after_len && events.iter().all(|e| match e {
        Event::Write { old_len, new_len, .. } | Event::Resize { old_len, new_len } => old_len == new_len,
        _ => true,
    });
    println!("{{\"prior_requested\":{prior},\"prior_header_mode\":{prior_mode},\"current_header_mode\":{final_mode},\"commit_ok\":true,\"reopened_pair\":\"new\",\"before_len\":{before_len},\"after_len\":{after_len},\"fixed_layout\":{fixed_layout},\"events\":[{}],\"formal_source_proof\":false}}", events.iter().map(Event::json).collect::<Vec<_>>().join(","));
    Ok(())
}
