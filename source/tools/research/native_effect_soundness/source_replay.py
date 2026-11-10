#!/usr/bin/env python3
"""Rebuild inherited extractor, audit its real typed IR; never publish a fact."""
import argparse
import json
import os
from pathlib import Path
import sys
sys.dont_write_bytecode=True
HERE=Path(__file__).resolve().parent
ROOT=HERE.parents[3]
sys.path.insert(0,str(ROOT/'source/tools'))
from infer_eval_support import ArtifactGuard,digest,support_artifacts
from infer_experiment import LoggedCommands,replay_receipt,require
from source_audit import obligations,validate_inventory,controls,binding
p=argparse.ArgumentParser()
p.add_argument('--output',type=Path,required=True)
p.add_argument('--solver',type=Path,required=True)
a=p.parse_args();a.output=a.output.resolve();a.output.mkdir(parents=True,exist_ok=False)
receipt=dict(schema=1,native_bridge_proved=False,commands=[])
with replay_receipt(a.output,receipt):
    frozen=json.loads((HERE/'source-freeze.json').read_text())
    for name,sha in frozen['files'].items():require(digest(ROOT/name)==sha,'source freeze: '+name)
    inputs=[ROOT/n for n in frozen['files']]+[HERE/'source-freeze.json',*support_artifacts()]
    guard=ArtifactGuard(inputs);receipt['artifacts']=guard.initial
    run=LoggedCommands(a.output,ROOT/'source',receipt['commands'],timeout=240,guard=guard)
    env=dict(os.environ,PATH='/home/dev/.cargo/bin:'+os.environ['PATH'],RUSTUP_TOOLCHAIN='1.99.1')
    inherited=HERE.parent/'native_iterator_certificate'
    upstream=a.output/'extraction'
    # The read-only inherited controller binds rustc/core/rust-src, rebuilds in
    # private target/build dirs, and executes eight honestly rebound Rust mutants.
    require(run('inherited-extraction',[sys.executable,'-B','-O',str(inherited/'replay.py'),
        '--output',str(upstream),'--solver',str(a.solver.resolve())],env)==0,'inherited extraction failed')
    data=json.loads((upstream/'original.json').read_text())
    sha=digest(inherited/'fixtures/original.rs')
    ledger=obligations(data,sha)
    validate_inventory(data,sha,ledger)
    receipt['controls']=controls(data,sha)
    receipt['inherited_controls']=json.loads((upstream/'receipt.json').read_text())['controls']
    receipt['inherited_source_controls']=json.loads((upstream/'receipt.json').read_text())['source_controls']
    # Additional separately compiled controls for mutable/key alias effects.
    sysroot=Path('/home/dev/.rustup/toolchains/1.99.1-x86_64-unknown-linux-gnu')
    binary=upstream/'target/debug/native-iterator-extract'
    binary_guard=ArtifactGuard([binary]);receipt['extractor']=binary_guard.initial
    env.update(LD_LIBRARY_PATH=str(sysroot/'lib'),RUSTC_BOOTSTRAP='1')
    receipt['additional_source_controls']=[]
    for name in ['key-write','alias-key']:
        source=HERE/'fixtures'/(name+'.rs');evidence=a.output/(name+'.json')
        require(run(name,[str(binary),str(evidence),'256','100000',str(source),'--crate-name','native_writer',
            '--edition','2024','--sysroot',str(sysroot),'-Zmir-opt-level=0','-Coverflow-checks=on'],env)==0,'source mutant must independently compile')
        variant=json.loads(evidence.read_text())
        require(variant['input_sha256']==digest(source) and digest(source)!=sha,'honest source rebind')
        vl=obligations(variant,digest(source))
        require('writer_call_inventory' in vl['writer_diagnostics'],'source mutant effect not detected')
        require(vl['checked_native_source_correspondences']==[] and vl['native_bridge_proved'] is False,'source mutant must not publish')
        receipt['additional_source_controls'].append(dict(id=name,classification='source-effect/eligibility-rejection',
            compiled=True,source_sha256=digest(source),evidence_sha256=digest(evidence),ledger_binding=binding(vl),diagnostics=vl['writer_diagnostics']))
    binary_guard.check();guard.check()
    (a.output/'source-ledger.json').write_text(json.dumps(ledger,indent=2)+'\n')
    summary={k:v for k,v in ledger.items() if k not in ('instruction_obligations','inherited_blockers','inherited_effect_obligations','drop_queries','call_edges')}
    summary.update(instruction_obligations=len(ledger['instruction_obligations']),inherited_effect_obligations=len(ledger['inherited_effect_obligations']),
        call_edges=len(ledger['call_edges']),drop_queries=len(ledger['drop_queries']),
        distinct_empty_drops=len({x['callee'] for x in ledger['drop_queries'] if x['empty_return']}),
        native_ledger_sha256=digest(a.output/'source-ledger.json'))
    receipt['summary']=summary
    (a.output/'summary.json').write_text(json.dumps(summary,indent=2)+'\n')
    print(json.dumps(summary))
