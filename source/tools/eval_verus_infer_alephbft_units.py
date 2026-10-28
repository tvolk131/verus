#!/usr/bin/env python3
"""Verify native Unit/Hasher bodies, with an explicit verified-lock adaptation.

SCALE's deterministic encode/skip-cache behavior is a declared library model.
This is not a proof of parking_lot's RAII guards or of the encoding algorithm.
"""
import argparse
import difflib
import json
import os
from pathlib import Path

from eval_verus_infer_alephbft import REVISION, git
from eval_verus_infer_alephbft_parents import ROOT, FIXTURES, NODE, CORRECTED, item, prepare, validate_native
from eval_verus_infer_alephbft_traits import UNITS
from eval_verus_infer_alephbft_lifecycle import lifecycle_api, private_core
from eval_verus_infer_alephbft_private import validate
from infer_eval_support import ArtifactGuard, digest, host_info, measured, tool_artifacts

MODELS = FIXTURES / 'full_unit_models.rs'
CODEC_MODEL = FIXTURES / 'codec_model.rs'
CONTROL = 'consensus/src/units/control_hash.rs'
HASHER = 'mock/src/hasher.rs'


def method(block, marker, result_type, ensures, prefix='', replacements=()):
    """Keep a pinned body except for enumerated lock API/ghost adaptations."""
    original=item(block,marker)[2]
    header,body=original.split('{',1)
    old='-> '+result_type
    if header.count(old)!=1: raise ValueError('changed method signature: '+marker)
    header=header.replace(old,'-> (result: '+result_type+')')
    for old,new in replacements:
        if body.count(old)!=1: raise ValueError('changed body adaptation: '+old)
        body=body.replace(old,new)
    return header+'\n    ensures '+ensures+',\n{'+prefix+body


def real_units(units, control, hasher):
    source=MODELS.read_text()
    coord=item(units,'impl UnitCoord')[2]
    coord_methods=[]
    for name,typ,post in [('new','Self','result.parts() == (creator, round)'),
        ('creator','NodeIndex','result == self.parts().0'),('round','Round','result == self.parts().1')]:
        coord_methods.append(method(coord,'pub fn '+name+'(',typ,post))
    source=source.replace('// COORD_METHODS','\n'.join(coord_methods))
    c=item(control,'impl<H: Hasher> ControlHash<H>')[2]
    source=source.replace('// CONTROL_METHODS','\n'.join([
        method(c,'pub fn combined_hash(','H::Hash','result == self@.1'),
        method(c,'pub fn n_members(','NodeCount','result.0 == self@.0.len()')]))
    pre=item(units,'impl<H: Hasher> PreUnit<H>')[2]
    pre_methods=[]
    for name,typ,post in [('new','Self','result@ == (creator, round, control_hash@)'),
        ('n_members','NodeCount','result.0 == self@.2.0.len()'),
        ('creator','NodeIndex','result == self@.0'),('round','Round','result == self@.1'),
        ('control_hash','&ControlHash<H>','result@ == self@.2')]:
        pre_methods.append(method(pre,'pub(crate) fn '+name+'(',typ,post))
    source=source.replace('// PREUNIT_METHODS','\n'.join(pre_methods))
    full=item(units,'impl<H: Hasher, D: Data> FullUnit<H, D>')[2]
    methods=[method(full,'pub(crate) fn new(','Self','result@ == (pre_unit@, data, session_id)',
        prefix='\nlet ghost expected = H::digest(encoded_fields::<UnitView<H,D>>((pre_unit@,data,session_id)));\n',
        replacements=[('RwLock::new(None)','RwLock::new(None, Ghost(CachePredicate { expected }))')])]
    for name,typ,post in [('as_pre_unit','&PreUnit<H>','result@ == self@.0'),
        ('data','&Option<D>','result == self@.1')]:
        methods.append(method(full,'pub(crate) fn '+name+'(',typ,post))
    source=source.replace('// FULLUNIT_METHODS','\n'.join(methods))
    clone=item(units,'impl<H: Hasher, D: Data> Clone for FullUnit<H, D>')[2]
    source=source.replace('// CLONE_BODY',method(clone,'fn clone(','Self','result@ == self@',
        prefix='\nproof { use_type_invariant(self); }\n',replacements=[
            ('self.hash.try_read().and_then(|guard| *guard)','self.hash.read_copy()'),
            ('RwLock::new(hash)','RwLock::new(hash, Ghost(self.hash.pred()))')]))
    trait=item(units,'pub trait Unit:')[2]
    source=source.replace('// DEFAULT_METHODS','\n'.join(method(trait,'fn '+n+'(',t,'result == self.'+n+'_value()')
        for n,t in [('creator','NodeIndex'),('round','Round')]))
    impl=item(units,'impl<H: Hasher, D: Data> Unit for FullUnit<H, D>')[2]
    methods=[]
    for name,typ,post in [('hash','H::Hash','result == self.hash_value()'),
        ('coord','UnitCoord','result.parts() == (self.creator_value(), self.round_value())'),
        ('control_hash','&ControlHash<Self::Hasher>','result@ == self.control_value()'),
        ('session_id','SessionId','result == self.session_value()')]:
        methods.append(method(impl,'fn '+name+'(',typ,post,
            prefix='\nproof { use_type_invariant(self); }\n' if name=='hash' else '',
            replacements=[('*self.hash.read()','self.hash.read_copy()'),
                ('*self.hash.write() = Some(hash);','self.hash.replace(Some(hash));')] if name=='hash' else []))
    source=source.replace('// UNIT_METHODS','\n'.join(methods))
    h=item(hasher,'impl Hasher for Hasher64')[2]
    source=source.replace('// HASHER_BODY',method(h,'fn hash(','Self::Hash','result == Self::digest(x@)'))
    return source


def real_api(node,shared,units,control,hasher):
    source=lifecycle_api(node,shared,units)
    start=source.rindex('verus! {',0,source.index('// Reduced interface:'))
    _,end,_=item(source[start:],'verus! {')
    return source[:start]+'\n'+real_units(units,control,hasher)+source[start+end:]


def main():
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('--alephbft',type=Path,required=True)
    p.add_argument('--verified-collector',type=Path)
    p.add_argument('--output',type=Path,required=True)
    p.add_argument('--verus',type=Path,default=ROOT/'source/target-verus/release/verus')
    p.add_argument('--z3',type=Path,default=ROOT/'source/z3')
    p.add_argument('--native-tests',action='store_true')
    p.add_argument('--native-target',type=Path)
    args=p.parse_args();output=args.output.resolve();repo=args.alephbft.resolve()
    supplied=(args.verified_collector or repo/'consensus/src/creation/collector/state.rs').resolve()
    if output.exists() and any(output.iterdir()):p.error('output must be new or empty')
    output.mkdir(parents=True,exist_ok=True)
    verus=args.verus.resolve();z3=args.z3.resolve()
    extras=[Path(__file__),supplied,MODELS,CODEC_MODEL,FIXTURES/'parents_properties.rs',FIXTURES/'units_native_tests.rs']
    extras+=list(Path(__file__).parent.glob('eval_verus_infer_alephbft*.py'))+[Path(__file__).with_name('infer_eval_support.py')]
    guard=ArtifactGuard(tool_artifacts(verus,z3,extra=extras))
    result=dict(host=host_info(),tool_artifacts=guard.initial,runs=[],native=[])
    def save():(output/'results.json').write_text(json.dumps(result,indent=2)+'\n')
    try:
        shared,origin=prepare(repo,output/'checkout');checkout=output/'checkout';directory=checkout/'verification-infer'
        node,units,control,hasher=[git(checkout,'show',f'{REVISION}:{path}') for path in (NODE,UNITS,CONTROL,HASHER)]
        shared=shared.replace('(*self * 2) / 3 + NodeCount(1)',CORRECTED)
        api=real_api(node,shared,units,control,hasher);state=private_core(supplied.read_text())
        apifile=directory/'units_api.rs';statefile=directory/'units_state.rs';props=directory/'units_properties.rs'
        properties=(FIXTURES/'parents_properties.rs').read_text().replace('// BRIDGE_ATTRIBUTE','#[verus_infer]')
        a=properties.index('#[path = "../crypto/src/node_count_value.rs"]');b=properties.index('// The existing shared collector')
        properties=properties[:a]+'pub use units_api::{NodeCount,NodeIndex,NodeMap,Hasher,Unit};\n\n'+properties[b:]
        properties=properties.replace('"collector_state.rs"','"units_state.rs"')
        properties+='''
verus! {
fn check_real_full_unit(
    state: &mut collector::Collector<units_api::Hasher64>,
    unit: &units_api::FullUnit<units_api::Hasher64, u32>,
)
    requires old(state).valid(),
    ensures final(state).valid(),
{
    state.check_unit_invariant(unit);
    state.check_unit_behavior(unit);
    units_api::check_repeated_hash(unit);
    units_api::check_clone_observations(unit);
}
}
'''
        statefile.write_text(state);props.write_text(properties)
        result['source']=dict(**origin,collector=str(supplied),collector_sha256=digest(supplied),
            bodies='pinned Hasher64 hash; UnitCoord/PreUnit/ControlHash getters and constructors; FullUnit construction, clone and all four Unit implementations; creator/round defaults',
            lock_adaptation='parking_lot RwLock replaced by verified vstd RwLock: read -> read_copy; write assignment -> replace; try_read-and_then in clone -> blocking read_copy; ghost predicate fixed at construction; explicit type-invariant proofs',
            codec_boundary='opaque deterministic SCALE encoding of immutable fields; cache skipped; encode modeled externally; using_encoded default body checked',
            bounds='reduced native marker/codec bounds; Hash and Data require checked CloneIdentity; Data additionally Copy (native mock Data=u32)',
            not_claimed='parking_lot RAII/Drop verification, nonblocking clone equivalence, SCALE algorithm proof, arbitrary Data clone behavior, collision resistance, whole Aleph crate or distributed protocol correctness')
        env=dict(os.environ,VERUS_Z3_PATH=str(z3),RUST_MIN_STACK='20971520',VERUS_INFER_CACHE=str(output/'candidate-cache'))
        # Compile the declared library boundary separately. The actual Unit and
        # hashing proof target below has no local external bodies or assumptions.
        codec_lib=directory/'libcodec_model.rlib';codec_interface=directory/'codec_model.vir'
        guard.check()
        codec_run=measured([str(verus),str(CODEC_MODEL),'--crate-name=codec_model','--crate-type=lib',
            '--compile','--export',str(codec_interface),'-o',str(codec_lib),'--output-json'],env,60,ROOT)
        guard.check()
        (output/'codec-model.stdout.json').write_text(codec_run['stdout']);(output/'codec-model.stderr.txt').write_text(codec_run['stderr'])
        if codec_run['returncode'] or not json.loads(codec_run['stdout'])['verification-results']['success']:
            raise ValueError('failed to compile declared codec model')
        codec_guard=ArtifactGuard([codec_lib,codec_interface])
        result['codec_model']=dict(trusted=True,source=str(CODEC_MODEL),source_sha256=digest(CODEC_MODEL),artifacts=codec_guard.initial)
        codec_args=['--extern',f'codec_model={codec_lib}','--import',f'codec_model={codec_interface}',
            '-L',f'dependency={directory}']
        lib=directory/'libunits_api.rlib';interface=directory/'units_api.vir'
        def run(case,stage,source,command,expected,count):
            snapshot=output/'inputs'/case;snapshot.mkdir(parents=True,exist_ok=True)
            for f in source:(snapshot/f.name).write_text(f.read_text())
            guard.check();codec_guard.check();r=measured(command,env,90,ROOT);guard.check();codec_guard.check()
            (output/f'{case}-{stage}.stdout.json').write_text(r['stdout']);(output/f'{case}-{stage}.stderr.txt').write_text(r['stderr'])
            data=validate(r,expected,count)
            if not expected and 'postcondition not satisfied' not in r['stderr'] and 'precondition not satisfied' not in r['stderr']:
                raise ValueError('negative control needs a semantic proof failure')
            result['runs'].append(dict(case=case,stage=stage,verification=data['verification-results'],report=data['verus-infer'],wall_micros=r['wall_micros']))
            save();print(f'{case}/{stage}: expected outcome',flush=True)
        for case,variant,expected in [('real',api,True),
            ('bad-cache-invariant',api.replace('value.is_none() || value.unwrap() == self.expected','true'),False),
            ('wrong-hasher-input',api.replace('hasher.write(x);','hasher.write(&[]);'),False),
            ('wrong-session',api.replace('        self.session_id\n','        0\n'),False),
            ('real-warm',api,True)]:
            apifile.write_text(variant)
            if variant==api and not expected:raise ValueError('mutation did not change source')
            run(case,'provider',[apifile],[str(verus),str(apifile),'--crate-name=units_api','--crate-type=lib','--compile','--export',str(interface),'-o',str(lib),'--infer-goals','--no-cheating','--output-json']+codec_args,expected,8)
            if not expected and interface.exists():raise ValueError('failed provider exported an interface')
            if expected:
                run(case,'consumer',[statefile,props],[str(verus),str(props),'--crate-type=lib','--extern',f'units_api={lib}','--import',f'units_api={interface}','--infer-goals','--no-cheating','--output-json']+codec_args,True,5)
        if args.native_tests:
            native_units=checkout/UNITS
            end=native_units.read_text().rfind('}')
            tests=(FIXTURES/'units_native_tests.rs').read_text()
            native_units.write_text(units[:end]+tests+units[end:])
            native_env=dict(os.environ);native_env.pop('RUSTUP_TOOLCHAIN',None)
            native_env['CARGO_TARGET_DIR']=str((args.native_target or output/'native-target').resolve())
            for case in ('real-native','wrong-native-clone','restored-native'):
                current=native_units.read_text()
                if case=='wrong-native-clone':native_units.write_text(current.replace('data: self.data.clone(),','data: None,'))
                command=['cargo','test','--offline','-p','aleph-bft','--lib','units::tests::','--','--test-threads=1']
                guard.check();codec_guard.check();r=measured(command,native_env,600,checkout);guard.check();codec_guard.check()
                (output/f'{case}.stdout.txt').write_text(r['stdout']);(output/f'{case}.stderr.txt').write_text(r['stderr'])
                validate_native(r,case!='wrong-native-clone',case)
                result['native'].append(dict(case=case,returncode=r['returncode'],wall_micros=r['wall_micros']))
                native_units.write_text(current)
                save();print(f'{case}: expected outcome',flush=True)
        (output/'units.patch').write_text(''.join(difflib.unified_diff(units.splitlines(keepends=True),real_units(units,control,hasher).splitlines(keepends=True),fromfile='a/native_units.rs',tofile='b/formal_units.rs')))
        result['success']=True
    except Exception as e:result['success']=False;result['error']=str(e);raise
    finally:save()

if __name__=='__main__':main()
