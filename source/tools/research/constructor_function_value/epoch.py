"""Private exact-command evidence; run with Python -B. No assert-based guards."""
import hashlib, json, os, pathlib, subprocess, sys, time
ROOT = pathlib.Path(__file__).resolve().parents[4]
PRIVATE = pathlib.Path('/tmp/constructor-function-value-t3-3c5e9110')
def digest(p):
    return hashlib.sha256(pathlib.Path(p).read_bytes()).hexdigest()
def freeze(extra=()):
    paths = subprocess.check_output(['git', 'ls-files', '-z'], cwd=ROOT).decode().split('\0')
    paths = [ROOT / p for p in paths if p and (ROOT / p).is_file()]
    paths += list((ROOT / 'source/tools/research/constructor_function_value').glob('*.py'))
    paths += list((ROOT / 'source/tools/research/constructor_function_value').glob('*.rs'))
    paths += [pathlib.Path(p).resolve() for p in extra]
    return {str(p): digest(p) for p in sorted(set(paths))}
def run(label, argv, extra=(), env=None, cwd=None):
    directory = PRIVATE / label
    directory.mkdir(parents=True, exist_ok=False)
    before = freeze(extra)
    record = dict(argv=argv, cwd=str(cwd or ROOT), environment=env or {}, frozen=before)
    (directory/'before.json').write_text(json.dumps(record, indent=2)+'\n')
    t=time.monotonic()
    with (directory/'stdout').open('wb') as out, (directory/'stderr').open('wb') as err:
        result=subprocess.run(argv,cwd=cwd or ROOT,env={**os.environ,**(env or {})},stdout=out,stderr=err)
    after=freeze(extra)
    record.update(exit=result.returncode,seconds=time.monotonic()-t,final_guards=before==after,
                  stdout_sha256=digest(directory/'stdout'),stderr_sha256=digest(directory/'stderr'))
    (directory/'receipt.json').write_text(json.dumps(record,indent=2)+'\n')
    print(json.dumps({k:v for k,v in record.items() if k!='frozen'}))
    if before!=after: raise RuntimeError('identity changed within epoch')
    return record
if __name__=='__main__':
    run(sys.argv[1],sys.argv[2:],extra=[p for p in sys.argv[2:] if pathlib.Path(p).is_file()])
