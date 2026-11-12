#!/usr/bin/env python3
"""Frozen source-specific baseline, not a native-state correspondence proof."""
import argparse
import json
import os
from pathlib import Path
import shutil
import sys
HERE=Path(__file__).resolve().parent
ROOT=HERE.parents[4]
sys.path.insert(0,str(ROOT/'source/tools'))
from infer_eval_support import ArtifactGuard,digest
from infer_experiment import LoggedCommands,replay_receipt,require,validate_run,require_function_coverage

def main():
    p=argparse.ArgumentParser();p.add_argument('--output',type=Path,required=True)
    a=p.parse_args();out=a.output.resolve();out.mkdir(parents=True,exist_ok=False)
    r=dict(kind='FROZEN_NATIVE_OWNED_EXCLUSIVE_SOURCE_BASELINE',commands=[],outcomes=[],native_bridge_proved=False)
    with replay_receipt(out,r):
        f=json.loads((HERE/'BASELINE_FREEZE.json').read_text())
        for path,h in f['inputs'].items():require(digest(Path(path))==h,'input '+path)
        rt=f['runtime'];src=Path(rt['bundle']);bundle=out/'bundle';bundle.mkdir()
        require(digest(src/'receipt.json')==rt['receipt_sha256'],'accepted receipt')
        for name,h in rt['tool_hashes'].items():
            require(digest(src/name)==h,'accepted tool '+name);shutil.copy2(src/name,bundle/name)
            require(digest(bundle/name)==h,'private tool '+name)
        shutil.copy2(src/'verus-root',bundle/'verus-root')
        guard=ArtifactGuard([HERE/'BASELINE_FREEZE.json',*[Path(p) for p in f['inputs']],*[bundle/n for n in rt['tool_hashes']]])
        tc=Path(f['toolchain'])
        env=dict(os.environ,PATH='/home/dev/.cargo/bin:'+str(tc/'bin')+':/usr/bin:/bin',
            RUSTUP_HOME='/home/dev/.rustup',RUSTUP_TOOLCHAIN=str(tc),LD_LIBRARY_PATH=str(tc/'lib'),VERUS_Z3_PATH=str(bundle/'z3'))
        run=LoggedCommands(out,out,r['commands'],timeout=120,guard=guard)
        require(run('native_mir',[str(tc/'bin/rustc'),str(HERE/'owned_native.rs'),'--crate-type=lib',
            '--crate-name=owned_native','--edition=2024','--emit=mir','-Copt-level=2',
            '-Coverflow-checks=on','-o',str(out/'owned_native.mir')],env)==0,'ordinary native compilation')
        r['native_mir_sha256']=digest(out/'owned_native.mir')
        for infer in [False,True]:
            label='infer' if infer else 'ordinary'
            argv=[str(bundle/'verus'),str(HERE/'owned_goals.rs'),'--crate-type=lib','--edition=2024',
                '--no-cheating','--output-json','--time','--multiple-errors','8']+(['--infer-goals'] if infer else [])
            code=run(label,argv,dict(env,VERUS_INFER_CACHE=str(out/('cache_'+label))))
            err=(out/(label+'.stderr')).read_text();data=validate_run(dict(returncode=code,
                stdout=(out/(label+'.stdout')).read_text(),stderr=err),False,allow_frontend=True)
            require(data['verification-results'].get('encountered-vir-error') is True and
                'UnsafeCell' in err and 'is not supported' in err,'owned/exclusive admission boundary')
            require_function_coverage(data.get('verus-infer',{}).get('functions',[]),[])
            r['outcomes'].append(dict(label=label,data=data,stderr_sha256=digest(out/(label+'.stderr'))))
            print(label,data['verification-results'],flush=True)
        guard.check();r['final_guard_passed']=True
if __name__=='__main__':main()
