"""Direct focused replays with incoming test binary identities frozen first."""
import json,os
from pathlib import Path as P
import review as e
W=P(e.read(e.HERE/'work.json')['work']);B=W/'final-bundle'
for name,count in [('constructor_function_values',13),('infer_compare_derive',7),('infer_nonzero_derive',3)]:
    bins=[p for p in (W/'target/release/deps').glob(name+'-*') if p.is_file() and os.access(p,os.X_OK) and p.suffix!='.d']
    e.need(len(bins)==1,'ambiguous own focused binary')
    binary=bins[0];logs=W/'final-test-logs'/name
    out,r=e.epoch(W,'final-test-'+name,[binary,'--test-threads=2'],extra=[binary]+[B/n for n in e.NAMES]+[B/'receipt.json'],envextra={'VERUS_EXTRA_ARGS':'','VERUS_KEEP_TEST_DIR':'1','VERUS_TEST_LOG_DIR':str(logs)},timeout=300)
    log=(out/'stdout').read_text();e.need(r['exit']==0 and f'test result: ok. {count} passed; 0 failed' in log,'focused suite failure')
    products={str(p):e.sha(p) for p in logs.rglob('*') if p.is_file()}
    v=dict(suite=name,passed=count,exit=r['exit'],binary=str(binary),binary_sha256=e.sha(binary),test_products=products,acceptance_checks_passed=True)
    e.write(out/'validation.json',v)
    with (W/'validations.jsonl').open('a') as f:f.write(json.dumps(dict(path=str(out),validation_sha256=e.sha(out/'validation.json')))+'\n')
    print(name,count,'passed',flush=True)
