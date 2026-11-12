#!/usr/bin/env python3
"""Compact original initializer replay; existing experiment utilities, no new models."""
import argparse,json,os,sys
from pathlib import Path
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[4]
sys.path.insert(0,str(ROOT/'source/tools'))
from infer_experiment import LoggedCommands,replay_receipt,require,validate_run,require_exact,require_function_coverage,require_publication
from infer_eval_support import ArtifactGuard,digest
TC=Path('/home/dev/.rustup/toolchains/1.99.1-x86_64-unknown-linux-gnu')
def main():
    p=argparse.ArgumentParser();p.add_argument('--output',type=Path,required=True);p.add_argument('--source',choices=['baseline','corrected-baseline','manual'],default='baseline');a=p.parse_args()
    a.output.mkdir(parents=True,exist_ok=False);r=dict(kind='F3C_ORIGINAL_BOOTSTRAP',commands=[],outcomes=[])
    with replay_receipt(a.output,r):
        f=json.loads((HERE/'BOOTSTRAP_INPUTS.json').read_text());base=Path(f['private_root']);bundle=base/'bundle'
        for n,h in f['files'].items():require(digest(HERE/n)==h,'frozen source '+n)
        require(digest(Path(f['context_receipt']))==f['original_context_receipt_sha256'],'registered context')
        require(digest(Path(f['expansion_receipt']))==f['expansion_receipt_sha256'],'actual expansion receipt')
        require(json.loads(Path(f['expansion_receipt']).read_text())['success'],'failed expansion')
        for n,h in f['artifacts'].items():require(digest(Path(n))==h,'registered private native lib '+n)
        require(digest(bundle/'receipt.json')==f['runtime_receipt_sha256'],'runtime receipt')
        for n,h in f['runtime']['tools'].items():require(digest(bundle/n)==h,'private accepted tool '+n)
        require((bundle/'verus-root').is_file(),'complete runtime marker')
        source=HERE/(a.source+'.rs');guarded=[source,Path(__file__),HERE/'BOOTSTRAP_INPUTS.json',Path(f['context_receipt']),Path(f['expansion_receipt'])]+[Path(n) for n in f['artifacts']]+[bundle/n for n in f['runtime']['tools']]
        guard=ArtifactGuard(guarded);run=LoggedCommands(a.output,a.output,r['commands'],timeout=180,guard=guard)
        r['inputs']={str(p):digest(p) for p in guarded};r['manual_nominal_domains']=3
        env=dict(os.environ,PATH='/home/dev/.cargo/bin:'+str(TC/'bin')+':/usr/bin:/bin',RUSTUP_HOME='/home/dev/.rustup',RUSTUP_TOOLCHAIN=str(TC),LD_LIBRARY_PATH=str(TC/'lib'),VERUS_Z3_PATH=str(bundle/'z3'),VERUS_INFER_CACHE=str(a.output/'cache'))
        for mode in ['ordinary','infer']:
            artifact=a.output/(mode+'.vir')
            cmd=[str(bundle/'verus'),str(source),'--edition=2024','--crate-type=lib','--crate-name','f3c_bootstrap','--no-cheating','--output-json','--time','--compile','--export',str(artifact),'-o',str(a.output/('lib'+mode+'.rlib')),'--extern','secp256k1='+str(base/'context/libs/libsecp256k1.rlib'),'-L','dependency='+str(base/'context/libs')]
            if mode=='infer':cmd+=['--infer-goals']
            code=run(mode,cmd,env);stdout=(a.output/(mode+'.stdout')).read_text();stderr=(a.output/(mode+'.stderr')).read_text()
            data=validate_run(dict(returncode=code,stdout=stdout,stderr=stderr),code==0,allow_frontend=True)
            r['outcomes'].append(dict(mode=mode,returncode=code,data=data,stderr_sha256=digest(a.output/(mode+'.stderr')),export_exists=artifact.exists()))
            print(mode,data['verification-results'],flush=True)
        r['baseline_inspection_only']=True
if __name__=='__main__':main()
