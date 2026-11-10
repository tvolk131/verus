"""Independent exact-command recorder. Invoke with python3 -B [-O]."""
import hashlib, json, os, pathlib, subprocess, sys, time
ROOT = pathlib.Path(__file__).resolve().parents[2]
PRIVATE = pathlib.Path('/tmp/constructor-function-value-review-t3-755cb0d1')
WORK = PRIVATE/'work'
TC = pathlib.Path('/home/dev/.rustup/toolchains/1.99.1-x86_64-unknown-linux-gnu')
BUNDLE = pathlib.Path('/tmp/verus-infer-buffer-cycle-20261109/parent-stable-key-bundle-a93b774a')
def require(ok, why):
    if not ok: raise RuntimeError(why)
def digest(p): return hashlib.sha256(pathlib.Path(p).read_bytes()).hexdigest()
def dump(p, data): pathlib.Path(p).write_text(json.dumps(data, indent=2, sort_keys=True)+'\n')
def freeze(extra=()):
    names = subprocess.check_output(['git','ls-files','-z'],cwd=ROOT).decode().split('\0')
    paths = [ROOT/n for n in names if n and (ROOT/n).is_file()]
    paths += list((ROOT/'evidence/constructor-function-value-review').glob('*.py'))
    paths += list((ROOT/'evidence/constructor-function-value-review').glob('*.rs'))
    paths += [ROOT/'evidence/constructor-function-value-review/INTERFACE.md']
    manifest = PRIVATE/'source-copy.json'
    if manifest.exists():
        for n,h in json.loads(manifest.read_text()).items():
            require(digest(WORK/n)==h, 'private source copy '+n)
        paths += [manifest]
    paths += list(TC.glob('lib/librustc_driver*')) + [TC/'bin/rustc',TC/'bin/cargo',BUNDLE/'z3']
    paths += [pathlib.Path(x).resolve() for x in extra]
    return {str(p):digest(p) for p in sorted(set(paths))}
def run(label, argv, extra=(), env=None, cwd=None, products=()):
    d=PRIVATE/'epochs'/label; d.mkdir(parents=True,exist_ok=False)
    before=freeze(extra)
    clean={k:v for k,v in os.environ.items() if not k.startswith(('CARGO','RUST','VERUS')) and k not in ('LD_LIBRARY_PATH','TMPDIR')}
    effective={**clean, **(env or {})}
    r=dict(argv=argv,cwd=str(cwd or WORK/'source'),environment=effective,frozen=before)
    dump(d/'before.json',r); start=time.monotonic()
    with (d/'stdout').open('wb') as out,(d/'stderr').open('wb') as err:
        p=subprocess.Popen(argv,cwd=r['cwd'],env=effective,stdout=out,stderr=err); code=p.wait()
    after=freeze(extra)
    r.update(exit=code,seconds=time.monotonic()-start,final_guards=before==after,
             stdout_sha256=digest(d/'stdout'),stderr_sha256=digest(d/'stderr'),
             products={str(p):digest(p) for p in products if pathlib.Path(p).is_file()})
    dump(d/'receipt.json',r)
    print(json.dumps({k:v for k,v in r.items() if k not in ('frozen','environment')}),flush=True)
    require(before==after,'epoch identities changed'); return r
