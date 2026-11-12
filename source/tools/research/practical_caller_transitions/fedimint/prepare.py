#!/usr/bin/env python3
"""Authenticate named immutable inputs and copy the runtime privately; no trials."""
import hashlib, json, shutil, subprocess, tarfile, tomllib
from pathlib import Path
HERE = Path(__file__).resolve().parent
ROOT = Path('/tmp/verus-infer-cycle-20261109/applications/fedimint')
RUNTIME = Path('/tmp/practical-combined78-5ecea5fe-core/bundle')
RECEIPT = '3e04cec7545fd1b44781ba6e8c2447e2109339a9f5257ef7b605d81896a16452'
COMMIT = 'b8bd3d8ef61e1410bd4ad01d2e3b6425f7973236'
def sha(p): return hashlib.sha256(p.read_bytes()).hexdigest()
def require(b, msg):
    if not b: raise ValueError(msg)
def prepare(out):
    out = Path(out); out.mkdir(parents=True, exist_ok=False)
    require(sha(RUNTIME/'receipt.json') == RECEIPT, 'receipt pin')
    receipt = json.loads((RUNTIME/'receipt.json').read_text())
    require(receipt['success'] is True and receipt['source_commit'] == '5ecea5fe81c2bc2ef0c9050cdb29cea0c7ef21c4' and len(receipt['tools']) == 8, 'runtime identity')
    for n, h in receipt['tools'].items(): require(sha(RUNTIME/n) == h, 'tool '+n)
    shutil.copytree(RUNTIME, out/'bundle')
    require((out/'bundle/verus-root').is_file(), 'runtime marker')
    for n,h in receipt['tools'].items(): require(sha(out/'bundle'/n) == h, 'copied tool '+n)
    require(subprocess.check_output(['git','-C',str(ROOT),'rev-parse','HEAD'],text=True).strip() == COMMIT, 'source commit')
    paths = ['Cargo.toml','Cargo.lock','fedimint-server/Cargo.toml','fedimint-server/src/consensus/aleph_bft/keychain.rs','fedimint-server/src/consensus/aleph_bft/mod.rs','fedimint-server/src/consensus/engine.rs','fedimint-server/src/config/mod.rs','fedimint-core/src/peer_id.rs','fedimint-core/src/session_outcome.rs']
    pins = {}
    for n in paths:
        b = subprocess.check_output(['git','-C',str(ROOT),'show',COMMIT+':'+n])
        require((ROOT/n).read_bytes()==b, 'clean pinned source '+n)
        dest = out/'source'/n; dest.parent.mkdir(parents=True,exist_ok=True); dest.write_bytes(b)
        pins[n] = sha(dest)
    require(pins[paths[3]] == '76225d0be3e169f438e9ddc6ce506cffdbda8fa9544dbb8d536a85210731f2fd', 'keychain pin')
    require(pins[paths[5]] == 'dffbb300941944f3360ec15377f86f6befd203319de321b52c7e924f901323d1', 'engine pin')
    lock=tomllib.loads((ROOT/'Cargo.lock').read_text())
    wanted = {('fedimint-aleph-bft','0.36.0'),('aleph-bft-types','0.13.0'),('aleph-bft-crypto','0.9.1'),('secp256k1','0.29.1'),('secp256k1-sys','0.10.1'),('bitcoin','0.32.8'),('bitcoin_hashes','0.14.0')}
    deps=[]
    for p in lock['package']:
        if (p['name'],p['version']) not in wanted: continue
        n=p['name']+'-'+p['version']; archive=Path('/home/dev/.cargo/registry/cache/index.crates.io-1949cf8c6b5b557f')/(n+'.crate')
        require(sha(archive)==p['checksum'], 'locked archive '+n)
        # Read only named registry artifacts authenticated by the project lock.
        with tarfile.open(archive) as t:
            for member in t.getmembers():
                if not member.isfile(): continue
                rel=Path(member.name); require(not rel.is_absolute() and '..' not in rel.parts, 'archive path')
                dest=out/'dependencies'/rel; dest.parent.mkdir(parents=True,exist_ok=True); dest.write_bytes(t.extractfile(member).read())
        deps.append(p)
    require({(p['name'],p['version']) for p in deps} == wanted, 'dependency pins')
    data=dict(source_commit=COMMIT,source_files=pins,runtime_receipt_sha256=RECEIPT,runtime=receipt,dependencies=deps,private_root=str(out))
    (out/'authentication.json').write_text(json.dumps(data,indent=2)+'\n')
    return data
if __name__=='__main__':
    import sys
    print(json.dumps(prepare(sys.argv[1]),indent=2))
