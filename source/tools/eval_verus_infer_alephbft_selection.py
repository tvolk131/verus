#!/usr/bin/env python3
"""Check inferred borrowed Result parent-selection against the supplied Aleph goals.

This retains the existing supplied core/invariant and private NodeMap adapters.
Native Clone and FullUnit integration are separate stages of the broader work.
"""
import argparse
import json
import os
from pathlib import Path
from eval_verus_infer_alephbft import REVISION, git
from eval_verus_infer_alephbft_parents import ROOT, NODE, METHOD, CORRECTED, FIXTURES, prepare, item
from eval_verus_infer_alephbft_lifecycle import direct_core, private_core, lifecycle_api, mutate
from eval_verus_infer_alephbft_private import validate
from eval_verus_infer_alephbft_traits import UNITS
from infer_experiment import save_results
from infer_eval_support import ArtifactGuard, digest, host_info, measured, tool_artifacts


def selection_core(original, private=False):
    source=private_core(original) if private else direct_core(original)
    if not private:
        start,end,_=item(source,'pub fn prospective_parents(')
        attr=source.rindex('    #[cfg_attr(verus_keep_ghost, verus_spec(',0,start)
        source=source[:attr]+source[end:]
    start,end,method=item(original,'pub fn prospective_parents(')
    attr=original.rindex('    #[cfg_attr(verus_keep_ghost, verus_spec(',0,start)
    contract=original[attr:start]
    if contract.count('        ensures')!=1:raise ValueError('selection contract layout changed')
    contract=contract.replace('        ensures', '''        ensures
            self.count() < (2 * self.parents().len()) / 3 + 1 ==>
                (result matches Err(SelectionError::NotEnoughParents)),
            self.count() >= (2 * self.parents().len()) / 3 + 1 && !self.ready(creator) ==>
                (result matches Err(SelectionError::MissingOwnParent)),''',1)
    # The private wrapper changes storage access spelling and borrowed return
    # type, while preserving branch order, bounds check and arithmetic policy.
    if private:
        method=method.replace('&ParentSlots<T>','&NodeMap<(H::Hash, u16)>')
        method=method.replace('self.direct_parents <','self.direct_parents.0 <')
        method=method.replace('consensus_threshold(self.candidates.len())','self.candidates.size().consensus_threshold().0')
        method=method.replace('self.candidates.len()','self.candidates.size().0')
        method=method.replace('self.candidates[creator]','self.candidates.get(NodeIndex(creator))')
        method=method.replace('u32::from(r)','u32::from(*r)')
        method=method.replace('-> Result<&NodeMap','-> Result<&NodeMap')
    else:
        # Vec preserves the same logical view; returned slice inference is a
        # separate unsizing model, so this checkpoint keeps the vector borrow.
        method=method.replace('&ParentSlots<T>','&Vec<Option<(T, u16)>>')
    harness=method[:method.index('{')].replace('pub fn prospective_parents(','pub fn check_selection(')
    observation='let n = self.candidates.size(); if creator < n.0 {let _ = self.candidates.get(NodeIndex(creator));}' if private else 'let _ = self.candidates.len();'
    check='\n    #[cfg(verus_keep_ghost)]\n'+contract+'    '+harness+'{'+observation+'self.prospective_parents(creator)}\n'
    addition='\n    #[cfg_attr(verus_keep_ghost,verus_infer)]\n    '+method+check
    marker='\n}\n' if not private else '\n}\n'
    # Insert in the executable impl, immediately after an existing goal method.
    _,end,_=item(source,'pub fn check_lifecycle(')
    source=source[:end]+addition+source[end:]
    return source


def without_index_guard(source):
    _,_,method=item(source,'pub fn prospective_parents(')
    _,_,guard=item(method,'if creator >=')
    return mutate(source,'prospective_parents',guard,'')


def validate_selection(run,expected,count,goal=False):
    data=validate(run,expected,count)
    owned=[f for f in data['verus-infer']['functions'] if f['summary_kind']=='owned_value']
    if len(owned)!=3:raise ValueError('missing exact lifecycle/selection summaries')
    if goal:
        if not any(n.endswith('::check_selection') for n in data['verus-infer']['failed_functions']):raise ValueError('selection oracle must fail')
        if not any(s in run['stderr'] for s in ('postcondition not satisfied','precondition not satisfied')):raise ValueError('missing semantic query rejection')
    return data


def main():
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--alephbft',type=Path,required=True);p.add_argument('--output',type=Path,required=True)
    p.add_argument('--verus',type=Path,default=ROOT/'source/target-verus/release/verus');p.add_argument('--z3',type=Path,default=ROOT/'source/z3')
    a=p.parse_args();out=a.output.resolve();repo=a.alephbft.resolve();supplied=repo/'consensus/src/creation/collector/state.rs'
    if out.exists() and any(out.iterdir()):p.error('output must be new or empty')
    out.mkdir(parents=True,exist_ok=True);verus=a.verus.resolve();z3=a.z3.resolve()
    extras=[Path(__file__),supplied,FIXTURES/'parents_properties.rs']+[Path(__file__).with_name(n) for n in ('eval_verus_infer_alephbft.py','eval_verus_infer_alephbft_parents.py','eval_verus_infer_alephbft_lifecycle.py','eval_verus_infer_alephbft_private.py','eval_verus_infer_alephbft_records.py','eval_verus_infer_alephbft_traits.py','eval_verus_infer_alephbft_wrappers.py','eval_verus_infer_alephbft_counting.py','infer_eval_support.py')]
    guard=ArtifactGuard(tool_artifacts(verus,z3,extra=extras));results=dict(host=host_info(),artifacts=guard.initial,runs=[])
    def save():
        save_results(out, results)
    try:
        shared,origin=prepare(repo,out/'checkout');checkout=out/'checkout';d=checkout/'verification-infer'
        shared=shared.replace('(*self * 2) / 3 + NodeCount(1)',CORRECTED);(checkout/METHOD).write_text(shared)
        original=supplied.read_text();node=git(checkout,'show',f'{REVISION}:{NODE}');units=git(checkout,'show',f'{REVISION}:{UNITS}')
        api=lifecycle_api(node,shared,units);direct=selection_core(original);private=selection_core(original,True)
        results['source']=dict(**origin,collector_sha256=digest(supplied),scope='supplied core parent-selection; Result borrowed views; original branch order and explicit bounds policy',adaptations='Vec borrow instead of slice; private API access spellings; existing invariant/count/round/Copy/trait adapters retained')
        properties=(FIXTURES/'parents_properties.rs').read_text().replace('// BRIDGE_ATTRIBUTE','#[verus_infer]').replace('"collector_state.rs"','"selection_state.rs"')
        private_properties=properties;start=properties.index('#[path = "../crypto/src/node_count_value.rs"]');end=properties.index('// The existing shared collector')
        private_properties=properties[:start]+'pub use selection_api::{NodeCount,NodeIndex,NodeMap,Hasher,Unit};\n\n'+properties[end:]
        state=d/'selection_state.rs';props=d/'selection_properties.rs';apifile=d/'selection_api.rs';lib=d/'libselection_api.rlib';interface=d/'selection_api.vir'
        env=dict(os.environ,VERUS_Z3_PATH=str(z3),RUST_MIN_STACK='20971520',VERUS_INFER_CACHE=str(out/'cache'))
        def run(case,stage,cmd,count,expected=True,goal=False):
            snap=out/'inputs'/case;snap.mkdir(parents=True,exist_ok=True)
            for f in [state,props,apifile]:
                if f.exists():(snap/f.name).write_text(f.read_text())
            guard.check();r=measured(cmd,env,60,ROOT);guard.check()
            (out/f'{case}-{stage}.stdout.json').write_text(r['stdout']);(out/f'{case}-{stage}.stderr.txt').write_text(r['stderr'])
            data=validate_selection(r,expected,count,goal) if stage=='consumer' else validate(r,expected,count)
            results['runs'].append(dict(case=case,stage=stage,command=cmd,report=data['verus-infer'],verification=data['verification-results']));save();print(case,stage,'expected outcome',flush=True)
        for private_flag,positive,prop in [(False,direct,properties),(True,private,private_properties)]:
            prefix='private' if private_flag else 'direct'
            if private_flag:
                apifile.write_text(api)
                run(prefix,'provider',[str(verus),str(apifile),'--crate-name=selection_api','--crate-type=lib','--export',str(interface),'--compile','-o',str(lib),'--infer-goals','--no-cheating','--output-json'],8)
            cases=[('positive',positive,True),('wrong-error',mutate(positive,'prospective_parents','SelectionError::NotEnoughParents','SelectionError::MissingOwnParent'),False),('missing-own-parent',mutate(positive,'prospective_parents','== self.for_round','<= self.for_round'),False),('unsafe-index',without_index_guard(positive),False),('warm',positive,True)]
            for name,variant,expected in cases:
                state.write_text(variant);props.write_text(prop)
                cmd=[str(verus),str(props),'--crate-type=lib','--infer-goals','--no-cheating','--output-json']
                if private_flag:cmd+=['--extern',f'selection_api={lib}','--import',f'selection_api={interface}']
                run(prefix+'-'+name,'consumer',cmd,6,expected,not expected)
        results['success']=True
    except Exception as e:results['success']=False;results['error']=str(e);raise
    finally:save()

if __name__=='__main__':main()
