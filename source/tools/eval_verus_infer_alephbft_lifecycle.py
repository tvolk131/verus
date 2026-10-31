#!/usr/bin/env python3
"""Infer construction and round advancement for the supplied Aleph collector.

The direct Vec core retains executable new/next bodies. The private-API variant
retains NodeMap::with_size and native derived Clone. A checked CloneIdentity
law is an explicit formal-interface requirement for its Hash payload.
"""
import argparse
import difflib
import json
import os
from pathlib import Path

from eval_verus_infer_alephbft import REVISION, git
from eval_verus_infer_alephbft_parents import ROOT, NODE, FIXTURES, CORRECTED, METHOD, item, prepare
from eval_verus_infer_alephbft_records import infer_add
from eval_verus_infer_alephbft_private import private_api, validate
from eval_verus_infer_alephbft_traits import getter_adapter, generic_collector, wrapper_api, wrapper_collector, UNITS
from infer_experiment import save_results
from infer_eval_support import ArtifactGuard, digest, host_info, measured, tool_artifacts


def remove_loop_specs(method):
    marker = '#[cfg_attr(verus_keep_ghost, verus_spec('
    while marker in method:
        start = method.index(marker); end = start + 1; depth = 0
        while end < len(method):
            depth += (method[end] == '[') - (method[end] == ']')
            end += 1
            if depth == 0: break
        if depth: raise ValueError('unbalanced loop annotation')
        method = method[:start] + method[end:]
    marker = '#[cfg(verus_keep_ghost)]'
    while marker in method:
        start = method.index(marker)
        _, end, _ = item(method[start:], 'proof!')
        method = method[:start] + method[start + end:]
    return method


def direct_core(original):
    source = infer_add(original)
    harnesses = []
    for name, signature, call in (
        ('new', 'pub fn check_new(node_count: usize) -> Self', 'Self::new(node_count)'),
        ('next', 'pub fn check_next(&self) -> Option<Self>', 'self.next()'),
    ):
        start, end, method = item(source, f'pub fn {name}(')
        attr = source.rindex('    #[cfg_attr(verus_keep_ghost, verus_spec(', 0, start)
        contract = source[attr:start]
        cleaned = remove_loop_specs(method)
        source = source[:attr] + '    #[cfg_attr(verus_keep_ghost, verus_infer)]\n    ' + cleaned + source[end:]
        harnesses.append('\n    #[cfg(verus_keep_ghost)]\n' + contract + '    ' + signature + ' { ' + call + ' }\n')
    closing = source.index('\n}\n\n#[cfg(test)]')
    source = source[:closing] + ''.join(harnesses) + trace_harness(False) + source[closing:]
    source=source[:source.index('\n#[cfg(test)]')]
    for name in ('new','next'):
        old=remove_loop_specs(item(original,f'pub fn {name}(')[2])
        new=item(source,f'pub fn {name}(')[2]
        if ''.join(old[old.index('{'):].split())!=''.join(new[new.index('{'):].split()):
            raise ValueError('lifecycle executable body changed')
    return source


def trace_harness(private):
    return '''
    #[cfg(verus_keep_ghost)]
    #[verus_spec()]
    pub fn check_lifecycle(node_count: usize, creator: usize, payload: PAYLOAD, round: u16) {
        let mut first = Self::check_new(node_count);
        first.check_add_invariant(creator, payload, round);
        match first.check_next() {
            None => {},
            Some(mut next) => {
                next.check_add_invariant(creator, payload, round);
                proof! { assert(next.valid()); }
            },
        }
    }
'''.replace('PAYLOAD', 'H::Hash' if private else 'T')


def lifecycle_api(node, shared, units):
    source = wrapper_api(private_api(node, shared) + getter_adapter(units), node)
    marker = '#[cfg_attr(not(verus_keep_ghost), derive(Clone, Eq, PartialEq, Debug))]'
    if source.count(marker) != 1: raise ValueError('NodeMap derive marker changed')
    source = source.replace(marker, '#[cfg_attr(verus_keep_ghost, verus_verify(clone_via_view))]\n#[derive(Clone)]')
    source = source.replace('pub trait Hasher { type Hash: Copy; }',
        'pub trait Hasher { type Hash: Copy + vstd::std_specs::clone::CloneIdentity; }')
    _, _, method = item(node, 'pub fn with_size(')
    source += '''
#[cfg_attr(verus_keep_ghost, verus_infer)]
impl From<NodeCount> for usize { fn from(value: NodeCount) -> Self { value.0 } }
#[cfg_attr(verus_keep_ghost, verus_verify)]
impl<T> NodeMap<T> {
    #[cfg_attr(verus_keep_ghost, verus_infer)]
    METHOD
}
'''.replace('METHOD', method)
    if item(source, 'pub fn with_size(')[2] != method: raise ValueError('original constructor changed')
    return source


def private_core(original):
    source = wrapper_collector(generic_collector(original))
    methods = '''
    #[cfg_attr(verus_keep_ghost, verus_infer)]
    pub fn new(node_count: usize) -> Self {
        let candidates = NodeMap::with_size(NodeCount(node_count));
        Collector { candidates, for_round: 1, direct_parents: NodeCount(0) }
    }
    #[cfg_attr(verus_keep_ghost, verus_infer)]
    pub fn next(&self) -> Option<Self> {
        if self.for_round == u32::from(u16::MAX) + 1 { return None; }
        let candidates = self.candidates.clone();
        Some(Collector { candidates, for_round: self.for_round + 1, direct_parents: NodeCount(0) })
    }
    #[cfg(verus_keep_ghost)]
    #[verus_spec(result => ensures
        result.valid(), result.round() == 1, result.count() == 0,
        result.parents().len() == node_count,
        forall|i:int| 0 <= i < node_count ==> result.parents()[i].is_none(),
    )]
    pub fn check_new(node_count: usize) -> Self { Self::new(node_count) }
    #[cfg(verus_keep_ghost)]
    #[verus_spec(result => requires self.valid(), ensures
        result.is_none() == (self.round() == u16::MAX as int + 1),
        result.is_some() ==> {
            let next = result.unwrap();
            &&& next.valid()
            &&& next.round() == self.round() + 1
            &&& next.count() == 0
            &&& next.parents() == self.parents()
        },
    )]
    pub fn check_next(&self) -> Option<Self> { self.next() }
'''
    start, end, _ = item(source, 'pub fn check_unit_behavior')
    return (source[:end] + methods + trace_harness(True) + source[end:]
        + '\n#[cfg(verus_keep_ghost)]\nverus! { broadcast use vstd::std_specs::clone::lemma_cloned_identity; }\n')


def mutate(source, name, old, new):
    start,end,method=item(source,f'pub fn {name}(')
    if method.count(old)!=1: raise ValueError('mutation target must be unique')
    return source[:start]+method.replace(old,new,1)+source[end:]


def validate_stage(run,expected,count,goal=None,owned_count=2):
    data=validate(run,expected,count)
    kinds=[f['summary_kind'] for f in data['verus-infer']['functions']]
    if kinds.count('owned_value')!=owned_count: raise ValueError('missing exact owned summaries')
    if goal:
        names=data['verus-infer'].get('failed_functions',[])
        if not any(n.endswith('::'+goal) for n in names):
            raise ValueError('expected named lifecycle goal did not fail')
        if not any(s in run['stderr'] for s in ('postcondition not satisfied','precondition not satisfied')):
            raise ValueError('missing semantic lifecycle failure')
    return data


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--alephbft', type=Path, required=True)
    parser.add_argument('--verified-collector', type=Path)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--verus', type=Path, default=ROOT/'source/target-verus/release/verus')
    parser.add_argument('--z3', type=Path, default=ROOT/'source/z3')
    args = parser.parse_args(); repo=args.alephbft.resolve(); output=args.output.resolve()
    supplied=(args.verified_collector or repo/'consensus/src/creation/collector/state.rs').resolve()
    if output.exists() and any(output.iterdir()): parser.error('output must be new or empty')
    output.mkdir(parents=True,exist_ok=True)
    verus=args.verus.resolve(); z3=args.z3.resolve()
    extras=[Path(__file__),supplied,FIXTURES/'parents_properties.rs']
    extras += [Path(__file__).with_name(n) for n in ('eval_verus_infer_alephbft.py','eval_verus_infer_alephbft_parents.py','eval_verus_infer_alephbft_records.py','eval_verus_infer_alephbft_counting.py','eval_verus_infer_alephbft_wrappers.py','eval_verus_infer_alephbft_private.py','eval_verus_infer_alephbft_traits.py','infer_eval_support.py')]
    guard=ArtifactGuard(tool_artifacts(verus,z3,extra=extras)); result=dict(host=host_info(),tool_artifacts=guard.initial,runs=[])
    def save():
        save_results(output, result)
    try:
        shared,origin=prepare(repo,output/'checkout'); checkout=output/'checkout'; directory=checkout/'verification-infer'
        original=supplied.read_text(); node=git(checkout,'show',f'{REVISION}:{NODE}'); units=git(checkout,'show',f'{REVISION}:{UNITS}')
        shared=shared.replace('(*self * 2) / 3 + NodeCount(1)',CORRECTED)
        (checkout/METHOD).write_text(shared)
        direct=direct_core(original); api=lifecycle_api(node,shared,units); private=private_core(original)
        result['source']=dict(**origin,collector=str(supplied),collector_sha256=digest(supplied),
            direct='supplied Vec core new/next executable bodies; contracts, loop annotations and local proof blocks removed',
            private='original NodeMap size/get/insert/with_size bodies and native derived Clone; supplied core factories delegate to those APIs',
            clone_law='Hash: CloneIdentity and its checked broadcast lemma; Copy alone is insufficient',
            retained='supplied invariant/count equivalence/behavior oracles, threshold correction, widened u32 round and Option terminal policy, reduced Unit/Hasher interface',
            not_claimed='full clean-main collector, arbitrary generic Clone equality, cached FullUnit hashing, new production bug discovery')
        properties=(FIXTURES/'parents_properties.rs').read_text().replace('// BRIDGE_ATTRIBUTE','#[verus_infer]')
        private_properties=properties
        start=private_properties.index('#[path = "../crypto/src/node_count_value.rs"]'); end=private_properties.index('// The existing shared collector')
        private_properties=private_properties[:start]+'pub use lifecycle_api::{NodeCount,NodeIndex,NodeMap,Hasher,Unit};\n\n'+private_properties[end:]
        env=dict(os.environ,VERUS_Z3_PATH=str(z3),RUST_MIN_STACK='20971520',VERUS_INFER_CACHE=str(output/'candidate-cache'))
        def run(case,stage,source,command,count,expected=True,goal=None):
            snapshot=output/'inputs'/case; snapshot.mkdir(parents=True,exist_ok=True)
            for path in source: (snapshot/path.name).write_text(path.read_text())
            guard.check(); run=measured(command,env,60,ROOT); guard.check()
            (output/f'{case}-{stage}.stdout.json').write_text(run['stdout']);(output/f'{case}-{stage}.stderr.txt').write_text(run['stderr'])
            data=validate_stage(run,expected,count,goal,1 if stage=='provider' else 2)
            result['runs'].append(dict(case=case,stage=stage,command=command,verification=data['verification-results'],report=data['verus-infer'],wall_micros=run['wall_micros']))
            save(); print(f'{case}/{stage}: expected outcome',flush=True)
        state=directory/'lifecycle_state.rs'; props=directory/'lifecycle_properties.rs'; apifile=directory/'lifecycle_api.rs'
        direct_cases=[('direct',direct,True,None),
            ('direct-wrong-initial',mutate(direct,'new','for_round: 1','for_round: 2'),False,'check_new'),
            ('direct-wrong-advance',mutate(direct,'next','self.for_round + 1,','self.for_round + 0,'),False,'check_next'),
            ('direct-restrictive-new',mutate(direct,'new','-> Self {','-> Self { let _unused = node_count + 1;'),False,'check_new'),
            ('direct-warm',direct,True,None)]
        for case,variant,expected,goal in direct_cases:
            state.write_text(variant); props.write_text(properties.replace('"collector_state.rs"','"lifecycle_state.rs"'))
            run(case,'consumer',[state,props],[str(verus),str(props),'--crate-type=lib','--infer-goals','--no-cheating','--output-json'],5,expected,goal)
        lib=directory/'liblifecycle_api.rlib'; interface=directory/'lifecycle_api.vir'
        private_cases=[('private',private,True,None),
            ('private-wrong-initial',mutate(private,'new','for_round: 1','for_round: 2'),False,'check_new'),
            ('private-wrong-reset',mutate(private,'next','direct_parents: NodeCount(0)','direct_parents: self.direct_parents'),False,'check_next'),
            ('private-premature-terminal',mutate(private,'next','u32::from(u16::MAX) + 1','1'),False,'check_next'),
            ('private-restrictive-new',mutate(private,'new','-> Self {','-> Self { let _unused = node_count + 1;'),False,'check_new'),
            ('private-warm',private,True,None)]
        for case,variant,expected,goal in private_cases:
            apifile.write_text(api); state.write_text(variant); props.write_text(private_properties.replace('"collector_state.rs"','"lifecycle_state.rs"'))
            run(case,'provider',[apifile],[str(verus),str(apifile),'--crate-type=lib','--crate-name=lifecycle_api','--export',str(interface),'--compile','-o',str(lib),'--infer-goals','--no-cheating','--output-json'],8)
            run(case,'consumer',[state,props],[str(verus),str(props),'--crate-type=lib','--extern',f'lifecycle_api={lib}','--import',f'lifecycle_api={interface}','--infer-goals','--no-cheating','--output-json'],5,expected,goal)
        (output/'lifecycle.patch').write_text(''.join(difflib.unified_diff(original.splitlines(keepends=True),private.splitlines(keepends=True),fromfile='a/supplied_state.rs',tofile='b/private_lifecycle.rs')))
        result['success']=True
        (output/'summary.md').write_text('# Aleph lifecycle checkpoint\n\n'+json.dumps(result['source'],indent=2)+'\n')
    except Exception as error:result['success']=False;result['error']=str(error);raise
    finally:save()

if __name__=='__main__':main()
