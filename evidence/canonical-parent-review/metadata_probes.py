#!/usr/bin/env python3
"""Intentional metadata rejections, kept separate from semantic evidence."""
import argparse
import copy
import json
import os
from pathlib import Path
import sys

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT/'source/tools'))
from infer_eval_support import ArtifactGuard, digest, tool_artifacts
from infer_experiment import (LoggedCommands, replay_receipt, require,
    require_exact, require_function_coverage, validate_run)


def main():
    p = argparse.ArgumentParser()
    p.add_argument('--bundle', type=Path, required=True)
    p.add_argument('--replay', type=Path, required=True)
    p.add_argument('--output', type=Path, required=True)
    a = p.parse_args()
    a.bundle = a.bundle.resolve()
    a.replay = a.replay.resolve()
    a.output = a.output.resolve()
    a.output.mkdir(mode=0o700)
    original = (a.replay/'provider.vir').read_bytes()
    at = original.rindex(b'{"schema":')
    meta = json.loads(original[at:-8])
    require(meta['policy']=='candidate-bound-evidence-v68' and meta['schema']==38, 'runtime metadata policy')
    require(len(meta['proofs'])==1 and meta['trusted_models'], 'missing provider proof/model inventory')
    variants = {}
    def variant(label, change, diagnostic):
        data = copy.deepcopy(meta)
        change(data)
        path = a.output/(label+'.vir')
        path.write_bytes(original[:at]+json.dumps(data,separators=(',',':')).encode()+b'VRINFER1')
        path.chmod(0o444)
        variants[label] = dict(path=str(path), sha256=digest(path), diagnostic=diagnostic)
    variant('producer', lambda m:m.update(producer='independent-wrong-producer'), 'incompatible inference schema, safety policy, or producer')
    variant('model', lambda m:m['trusted_models'][0].__setitem__(1,'0'*64), 'stale or missing trusted-model fingerprint')
    variant('dependency', lambda m:m['dependencies'].append(dict(crate_id=1,fingerprint='0'*64)), 'stale or missing inferred dependency fingerprint')
    variant('proofs', lambda m:m.update(proofs=[]), 'incomplete inference proof guarantees')
    variant('summary', lambda m:m.update(summaries='0'*64), 'inferred contract fingerprint mismatch')
    consumer = a.output/'consumer.rs'
    consumer.write_text('use vstd::prelude::*;verus! { fn caller(n:u64,b:&mut[u8;10]) { let r=provider::u64(n,b); assert(provider::canonical(n,r@)); } }')
    consumer.chmod(0o444)
    freeze = a.output/'freeze.json'
    freeze.write_text(json.dumps(variants,indent=2)+'\n')
    guard = ArtifactGuard(tool_artifacts(a.bundle/'verus',a.bundle/'z3',extra=[
        Path(__file__), consumer, freeze, a.replay/'provider.vir',a.replay/'libprovider.rlib',
        *[Path(v['path']) for v in variants.values()]]))
    env = dict(os.environ, PATH='/home/dev/.cargo/bin:'+os.environ['PATH'],
        RUSTUP_TOOLCHAIN='1.99.1',VERUS_Z3_PATH=str(a.bundle/'z3'),VERUS_INFER_CACHE=str(a.output/'cache'))
    receipt = dict(commands=[],runs=[],artifacts=guard.initial)
    with replay_receipt(a.output,receipt):
        invoke=LoggedCommands(a.output,a.output,receipt['commands'],timeout=180,guard=guard,status_key='returncode')
        for label,v in variants.items():
            argv=[str(a.bundle/'verus'),str(consumer),'--crate-type=lib','--no-cheating',
                '--infer-goals','--output-json','--extern','provider='+str(a.replay/'libprovider.rlib'),
                '--import','provider='+v['path']]
            code=invoke(label,argv,env)
            stdout=(a.output/(label+'.stdout')).read_text()
            stderr=(a.output/(label+'.stderr')).read_text()
            data=validate_run(dict(returncode=code,stdout=stdout,stderr=stderr),False,allow_frontend=True)
            require(v['diagnostic'] in stderr,'missing metadata diagnostic')
            require(not any(k in stderr for k in ('postcondition not satisfied','precondition not satisfied','assertion failed','error[E')), 'mixed metadata failure')
            helpers=data.get('verus-infer',{}).get('functions',[])
            require_function_coverage(helpers,set())
            require_exact(helpers)
            receipt['runs'].append(dict(label=label,**v,verification=data['verification-results'],
                stdout_sha256=digest(a.output/(label+'.stdout')),stderr_sha256=digest(a.output/(label+'.stderr'))))
            print(label,'rejected',flush=True)
        guard.check()
        receipt['artifacts_after']=guard.snapshot()


if __name__=='__main__':
    main()
