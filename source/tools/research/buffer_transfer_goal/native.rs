#[path="baseline-adapter.rs"] mod baseline_adapter;
#[path="manual-adapter.rs"] mod manual_adapter;
#[path="mutant-adapter.rs"] mod mutant_adapter;
use bitcoin_hashes::{hmac::HmacEngine, sha256, HashEngine};
fn selected(ipad: &mut [u8;128], key: &[u8]) {
            for (b_i, b_h) in ipad.iter_mut().zip(key) {
                *b_i ^= *b_h;
            }
}
fn main() {
    let mut cases = 0usize;
    let mut mutant_differences = 0usize;
    let mut seed = 0x123456789abcdef0u64;
    for len in [0usize,1,2,63,64,65,127,128,129,255,1024] {
        for pattern in 0u16..=255 {
            let mut key = vec![0u8;len];
            let mut pad = [pattern as u8;128];
            for b in &mut key { seed ^= seed << 13; seed ^= seed >> 7; seed ^= seed << 17; *b=seed as u8; }
            for b in &mut pad { seed ^= seed << 13; seed ^= seed >> 7; seed ^= seed << 17; *b ^= seed as u8; }
            let before=pad; let key_before=key.clone();
            let mut adapted=pad; baseline_adapter::xor_key(&mut adapted,&key);
            let mut manual=pad; manual_adapter::xor_key(&mut manual,&key);
            let mut mutant=pad; mutant_adapter::xor_key(&mut mutant,&key);
            selected(&mut pad, &key);
            assert_eq!(pad,adapted); assert_eq!(pad,manual);
            if pad != mutant { mutant_differences+=1; }
            for i in 0..128 { assert_eq!(pad[i], if i<len {before[i]^key[i]} else {before[i]}); }
            assert_eq!(key,key_before);
            let mut engine=HmacEngine::<sha256::Hash>::new(&key);
            engine.input(b"source-connected compilation");
            let _=engine.midstate();
            cases+=1;
        }
    }
    assert!(mutant_differences>0);
    println!("{cases} source-slice byte/frame and real-macro-erasure comparisons; {mutant_differences} OR-mutant differences; upstream HmacEngine<sha256::Hash> compiled and instantiated");
}
