#!/usr/bin/env python3
"""Check Aleph's data-bearing validation errors and original noninitial ordering.

The error enum, own-parent lookup and noninitial orchestration retain pinned
source bodies. Count/search pipelines use explicit checked collection queries.
"""
import argparse
import json
import os
import re
from pathlib import Path

from eval_verus_infer_alephbft import REVISION, git
from eval_verus_infer_alephbft_parents import item, FIXTURES, ROOT, NODE, CORRECTED, prepare, validate_native
from eval_verus_infer_alephbft_traits import UNITS
from eval_verus_infer_alephbft_units import real_api, method, CONTROL, HASHER, CODEC_MODEL
from eval_verus_infer_alephbft_native_collector import COMPARISONS, QUERY_MODELS
from eval_verus_infer_alephbft_private import validate
from infer_experiment import save_results
from infer_eval_support import ArtifactGuard, host_info, measured, tool_artifacts

MODELS = FIXTURES / 'validation_models.rs'
RUNTIME = FIXTURES / 'validation_runtime.rs'
NAMES = ('unit_creator_is_descendant_of_previous_unit',
         'previous_round_have_enough_parents',
         'check_if_parents_greater_than_previous_round', 'validate_non_initial_round')


def native_models(node, control, factory_proofs=False):
    """Fail on native factory drift before installing the checked ghost interface."""
    expected = [
        (node, 'pub fn iter(', 'self.0.iter().enumerate().filter_map(|(idx, maybe_value)| Some((NodeIndex(idx), maybe_value.as_ref()?)))'),
        (control, 'pub fn parents(', 'self.parents.iter().map(|(node_index, &round)| UnitCoord::new(round, node_index))'),
    ]
    for source, marker, body in expected:
        actual = item(source, marker)[2].split('{', 1)[1].rsplit('}', 1)[0]
        if re.sub(r'\s+', '', actual) != re.sub(r'\s+', '', body):
            raise ValueError('native iterator factory changed: '+marker)
    if factory_proofs:
        return (FIXTURES/'factory_models.rs').read_text().replace(
            '// ORIGINAL_NODE_FACTORY', item(node,'pub fn iter(')[2]).replace(
            '// ORIGINAL_PARENT_FACTORY', item(control,'pub fn parents(')[2])
    return (FIXTURES/'native_pipeline_models.rs').read_text()


def construction_api(control):
    implementation=item(control,'impl<H: Hasher> ControlHash<H>')[2]
    constructor=item(implementation,'pub fn new(')[2]
    helper=method(implementation,'pub fn create_control_hash(','H::Hash',
        'result == Self::construction_hash(parent_map)')
    return (FIXTURES/'native_construction_models.rs').read_text().replace(
        '// ORIGINAL_HASH_HELPER',helper).replace('// ORIGINAL_CONSTRUCTOR',
        '#[verifier::infer]\n'+constructor)


def full_validation_api(control):
    # Both branches and the public dispatcher infer from unchanged source.
    # Separate callers supply the complete policy and conditional safety bound.
    methods = ['#[verifier::infer]\n'+item(control, marker)[2]
        for marker in ('fn validate_initial_round(', 'pub fn validate(')]
    return (FIXTURES/'full_validation_models.rs').read_text().replace(
        '// ORIGINAL_FULL_VALIDATION', '\n'.join(methods))


def parent_validation_api(api, units, validator, signature):
    """Infer original accessors and parent orchestration; supply only interfaces/policy."""
    for marker, expected in (
        ('pub struct UncheckedSigned<', 'signable:T,signature:S,'),
        ('pub struct Signed<', 'unchecked:UncheckedSigned<T,K::Signature>,'),
    ):
        fields = item(signature, marker)[2].split('{', 1)[1].rsplit('}', 1)[0]
        if re.sub(r'\s+', '', fields) != expected:
            raise ValueError('signed wrapper representation changed: '+marker)
    for adapted, original, methods in (
        ('impl UnitCoord', 'impl UnitCoord', ['pub fn new(', 'pub fn creator(', 'pub fn round(']),
        ('impl<H: Hasher> PreUnit<H>', 'impl<H: Hasher> PreUnit<H>',
         ['pub(crate) fn new(', 'pub(crate) fn n_members(', 'pub(crate) fn creator(',
          'pub(crate) fn round(', 'pub(crate) fn control_hash(']),
        ('impl<H: Hasher, D: CloneView> FullUnit<H, D>', 'impl<H: Hasher, D: Data> FullUnit<H, D>',
         ['pub(crate) fn as_pre_unit(']),
    ):
        start, end, implementation = item(api, adapted)
        source = item(units, original)[2]
        for marker in methods:
            a, b, _ = item(implementation, marker)
            implementation = implementation[:a]+'#[verifier::infer]\n'+item(source, marker)[2]+implementation[b:]
        api = api[:start]+implementation+api[end:]
    signed = item(signature, 'impl<T: Signable + Index, K: Keychain> Signed<T, K>')[2]
    parent = item(validator, 'impl<K: Keychain> Validator<K>')[2]
    models = (FIXTURES/'parent_validation_models.rs').read_text()
    for marker, original in (
        ('// ORIGINAL_SIGNED_ACCESSOR', item(signed, 'pub fn as_signable(')[2]),
        ('// ORIGINAL_VALIDATION_ERROR', item(validator, 'pub enum ValidationError')[2]),
        ('// ORIGINAL_VALIDATOR', item(validator, 'pub struct Validator')[2]),
        ('// ORIGINAL_PARENT_VALIDATION', item(parent, 'fn validate_unit_parents<')[2]),
    ):
        if models.count(marker) != 1:
            raise ValueError('parent fixture marker changed: '+marker)
        models = models.replace(marker, original)
    return api+models


def outer_validation_api(api, validator, signature):
    """Retain the outer body and From conversion; declare authentication meaning."""
    authentication = '''
 spec fn authenticated<H:Hasher,D:Data>(&self,unit:UnitView<H,D>,creator:NodeIndex,signature:Self::Signature)->bool;
 fn authenticate<H:Hasher,D:Data>(&self,unit:&FullUnit<H,D>,signature:&Self::Signature)->(result:bool)
  ensures result==self.authenticated::<H,D>(unit@,unit.creator_value(),*signature);
'''
    site = ' type Signature:Signature;'
    if api.count(site) != 1:
        raise ValueError('authentication interface insertion changed')
    api = api.replace(site, site+authentication)
    marker = 'mod parent_validation {'
    api = api.replace(marker, (FIXTURES/'outer_validation_models.rs').read_text()+'\n'+marker)
    # The remaining marker bounds of SignatureError are outside this adapter.
    error_fields = item(signature, 'pub struct SignatureError<')[2].split('{',1)[1].rsplit('}',1)[0]
    if re.sub(r'\s+', '', error_fields) != 'pubunchecked:UncheckedSigned<T,S>,':
        raise ValueError('signature error representation changed')
    conversion = item(validator, 'impl<H: Hasher, D: Data, S: Signature> From<')[2]
    conversion = conversion.replace('    fn from(', '    #[verifier::infer]\n    fn from(', 1)
    original = item(validator, 'impl<K: Keychain> Validator<K>')[2]
    methods = '#[verifier::infer]\n'+item(original, 'pub fn new(')[2]
    methods += '\n'+method(original, 'pub fn validate_unit<', 'Result<H, D, K>',
        'self.outer_policy(uu,result)')
    methods = methods.replace('    ensures self.outer_policy(uu,result),',
        '    requires self.outer_domain(uu),\n    ensures self.outer_policy(uu,result),')
    insertion = '\n'+conversion+'\nimpl<K:Keychain> Validator<K>{\n'+methods+'\n}\n'
    insertion += (FIXTURES/'outer_validation_goals.rs').read_text()
    if not api.endswith('}}\n'):
        raise ValueError('parent module closing site changed')
    return api[:-3]+insertion+'}}\n'


def native_construction_runtime(runtime):
    old = 'let control = ControlHash::<Hasher64> { parents, combined_hash: [0; 8] };'
    new = """let mut hashes = NodeMap::with_size(NodeCount(size));
                for index in 0..size {
                    if let Some(&round) = parents.get(NodeIndex(index)) {
                        hashes.insert(NodeIndex(index), ([index as u8; 8], round));
                    }
                }
                let control = ControlHash::<Hasher64>::new(&hashes);
                for index in 0..size {
                    assert_eq!(control.parents.get(NodeIndex(index)).copied(), parents.get(NodeIndex(index)).copied());
                }
                assert_eq!(control.combined_hash, ControlHash::<Hasher64>::create_control_hash(&hashes));
                constructed += 1;"""
    if runtime.count(old)!=1:
        raise ValueError('constructor corpus state site changed')
    runtime=runtime.replace('    let mut cases = 0;', '    let mut cases = 0;\n    let mut constructed = 0;').replace(old,new)
    extra="""    assert_eq!(constructed, 378);
    let empty_input = NodeMap::with_size(NodeCount(0));
    let constructed_empty = ControlHash::<Hasher64>::new(&empty_input);
    assert_eq!(constructed_empty.parents.size().0, 0);
    assert!(constructed_empty.check_if_parents_greater_than_previous_round(0).is_ok());
    println!("NATIVE_CONSTRUCTORS=378;EMPTY=1");
"""
    return runtime.replace('    println!("INFER_VALIDATION_CASES=',extra+'    println!("INFER_VALIDATION_CASES=')


def validation_api(control, native_pipelines=False):
    error = item(control, 'pub enum Error')[2]
    implementation = item(control, 'impl<H: Hasher> ControlHash<H>')[2]
    methods = []
    for name in NAMES:
        method = item(implementation, 'fn '+name+'(')[2]
        if not native_pipelines and name == 'previous_round_have_enough_parents':
            old = '''self
            .parents()
            .filter(|&parent| parent.round == round - 1)
            .count()'''
            if method.count(old) != 1:
                raise ValueError('native filtered-count pipeline changed')
            method = method.replace(old, 'self.parents.rounds_in_range_checked(round - 1, round - 1)')
        elif not native_pipelines and name == 'check_if_parents_greater_than_previous_round':
            old = '''self
            .parents()
            .any(|unit_coord| unit_coord.round > round - 1)'''
            if method.count(old) != 1:
                raise ValueError('native future-parent pipeline changed')
            method = method.replace(old, 'self.parents.contains_after_checked(round - 1)')
        methods.append('#[verifier::infer]\n'+method)
    models = MODELS.read_text()
    if native_pipelines:
        start = models.index('impl NodeMap<u16>')
        end = models.index('// NATIVE_ERROR_ENUM')
        models = models[:start]+models[end:]
        models = models.replace('self.parents.rounds_in_range((coord.parts().1 - 1) as u16, (coord.parts().1 - 1) as u16)',
            'self.parents_value().filter(|p:UnitCoord|p.round==coord.parts().1 as int-1).len()')
        models = models.replace('self.parents.contains_after((coord.parts().1 - 1) as u16)',
            '(self.parents_value().filter(|p:UnitCoord|p.round>coord.parts().1 as int-1).len()>0)')
        models = models.replace('self.parents.rounds_in_range((round - 1) as u16, (round - 1) as u16)',
            'self.parents_value().filter(|p:UnitCoord|p.round==round as int-1).len()')
        models = models.replace('self.parents.contains_after((round - 1) as u16)',
            '(self.parents_value().filter(|p:UnitCoord|p.round>round as int-1).len()>0)')
    return models.replace('// NATIVE_ERROR_ENUM', error).replace(
        '// NATIVE_VALIDATION_METHODS', '\n'.join(methods))


def infer_member_count(api, control):
    """The original count wrapper is already within bounded record inference."""
    start, end, _ = item(api, 'pub fn n_members(')
    original = item(control, 'pub fn n_members(')[2]
    return api[:start]+'#[verifier::infer]\n'+original+api[end:]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--alephbft', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--verus', type=Path, default=ROOT/'source/target-verus/release/verus')
    parser.add_argument('--z3', type=Path, default=ROOT/'source/z3')
    parser.add_argument('--native-tests', action='store_true')
    parser.add_argument('--native-pipelines', action='store_true')
    parser.add_argument('--native-construction', action='store_true')
    parser.add_argument('--full-validation', action='store_true')
    parser.add_argument('--parent-validation', action='store_true')
    parser.add_argument('--factory-proofs', action='store_true')
    parser.add_argument('--outer-validation', action='store_true')
    parser.add_argument('--native-target', type=Path)
    args = parser.parse_args()
    if args.outer_validation:
        args.factory_proofs = True
    if args.factory_proofs:
        args.parent_validation = True
    if args.parent_validation:
        args.full_validation = True
    if args.full_validation:
        args.native_construction = True
    if args.native_construction:
        args.native_pipelines = True
    inferred_count = 31 if args.outer_validation else 29 if args.parent_validation else 18 if args.full_validation else 16 if args.native_construction else 15
    output = args.output.resolve()
    if output.exists() and any(output.iterdir()):
        parser.error('output must be new or empty')
    output.mkdir(parents=True, exist_ok=True)
    verus, z3 = args.verus.resolve(), args.z3.resolve()
    extras = list(Path(__file__).parent.glob('eval_verus_infer_alephbft*.py'))
    extras += list(FIXTURES.glob('*.rs')) + [Path(__file__).with_name('infer_eval_support.py')]
    guard = ArtifactGuard(tool_artifacts(verus, z3, extra=extras))
    result = dict(host=host_info(), tool_artifacts=guard.initial, runs=[], native=[])
    def save():
        save_results(output, result)
    try:
        shared, origin = prepare(args.alephbft.resolve(), output/'checkout')
        checkout = output/'checkout'
        directory = checkout/'verification-infer'
        node, units, control, hasher = [git(checkout, 'show', f'{REVISION}:{path}')
            for path in (NODE, UNITS, CONTROL, HASHER)]
        shared = shared.replace('(*self * 2) / 3 + NodeCount(1)', CORRECTED)
        api = infer_member_count(real_api(node, shared, units, control, hasher), control)
        api += COMPARISONS + (native_models(node, control, args.factory_proofs) if args.native_pipelines else QUERY_MODELS.read_text()) + validation_api(control, args.native_pipelines)
        if args.native_pipelines:
            api = api.replace('vstd::seq::group_seq_axioms};',
                'vstd::seq::group_seq_axioms,vstd::seq::group_seq_lemmas,vstd::std_specs::iter::group_iter_axioms,vstd::std_specs::iter_adapters::group_native_adapter_axioms};')
        if args.native_construction:
            api += construction_api(control)
        if args.full_validation:
            api = api.replace('type Hash: Copy + CloneIdentity;',
                'type Hash: Copy + CloneIdentity + vstd::std_specs::cmp::PartialEqIdentity;')
            api = api.replace('vstd::std_specs::iter_adapters::group_native_adapter_axioms};',
                'vstd::std_specs::iter_adapters::group_native_adapter_axioms,vstd::std_specs::cmp::group_eq_identity};')
            api += full_validation_api(control)
        if args.parent_validation:
            validator, signature = [git(checkout, 'show', f'{REVISION}:{path}') for path in
                ('consensus/src/units/validator.rs', 'crypto/src/signature.rs')]
            api = parent_validation_api(api, units, validator, signature)
        if args.outer_validation:
            api = outer_validation_api(api, validator, signature)
        result['source'] = dict(**origin,
            bodies='original six-variant Error<H>, own-parent lookup, n_members, runtime assertion and three-stage ? orchestration',
            adaptations='two iterator pipelines replaced by ordinary checked finite collection queries; positive-round validation scope',
            supplied='direct collection view, query equations, eligible policy, exact error goals, state constructor',
            boundaries='existing explicit SCALE codec and verified-lock boundaries; corrected quorum arithmetic; round remains u16',
            not_claimed='native iterator-pipeline inference, initial-round hash validation, full crate/protocol correctness; isolated future-parent leaf at round zero')
        if args.native_pipelines:
            result['source'].update(
                bodies='original six-variant Error<H> and all four noninitial validation bodies, including filter/count and any callbacks',
                adaptations='native iterator factories retain their executable operations with named locals and supplied ghost contracts; no collection-query implementation adapters',
                supplied='direct collection/iterator views, checked factory proofs, eligible policy, exact typed-error goals, state constructor',
                boundaries='new explicit native Rust-library count/enumerate/filter_map models; existing SCALE codec and verified lock; corrected quorum; original u16 rounds',
                not_claimed='automatic iterator-factory or arbitrary-loop inference, ControlHash::new, initial-round hash validation, full crate/protocol correctness')
        if args.native_construction:
            result['source'].update(
                construction='original ControlHash::new body and native for loop infer an exact unrestricted-domain fold summary; original create_control_hash is an ordinary checked generic value query',
                construction_goals='supplied exact fold/hash goals and constructed-value-to-typed-validation caller',
                construction_runtime='untouched native constructors exercise real SCALE/hasher implementations; proof-target runtime retains direct supplied states because the declared codec bridge is proof-only',
                not_claimed='automatic factory/view discovery, partial-domain or arbitrary loops, initial-round validation, proof of SCALE bytes or full protocol correctness')
        if args.full_validation:
            result['source'].update(
                validation='original initial branch and public dispatcher infer exact typed errors and maximal safety domains; separate ordinary callers check the supplied complete policy',
                construction_goals='input-map length and per-slot round projection; full construction-to-validation caller without supplied fold induction',
                equality='explicit checked PartialEqIdentity bound on the supplied Hasher interface; no assumption that Rust Eq implies value identity',
                not_claimed='automatic factory/view or policy discovery, proof of SCALE bytes/hash security, arbitrary loops, full crate/protocol correctness')
        if args.parent_validation:
            result['source'].update(
                parent_validation='original validate_unit_parents, map_err closure, signed accessor and nine unit accessors/constructors infer exact summaries',
                parent_policy='supplied member-check priority, complete PreUnit error views, exact wrapped ControlHash error, unchanged complete Signed success value and conditional creator bound',
                signed_boundary='already checked Signed input; source field layout and as_signable body retained; reduced marker traits and supplied Keychain count equation; no authentication/signing proof',
                not_claimed='outer validate_unit signature/session/max-round validation, automatic policy/view discovery, SCALE/hash security, full crate/protocol correctness')
        if args.factory_proofs:
            result['source'].update(
                factories='original NodeMap::iter and ControlHash::parents bodies with one iterator_via_view annotation each; no handwritten callback contracts, named-local adaptations or factory proof blocks',
                factory_proofs='ordinary verified iterator_view combinators and inferred total immutable callback equations; factory view contracts remain explicit author choices, not exact iterator-object inference',
                boundaries='existing native iterator specifications, supplied collection/output views, SCALE codec and verified-lock adaptation, corrected quorum and already checked Signed input')
        if args.outer_validation:
            result['source'].update(
                outer_validation='original validate_unit body is ordinarily checked directly against the supplied complete policy; original constructor and generic SignatureError From conversion infer exact contracts',
                outer_policy='authentication, session, inclusive maximum round, member count, then parent policy; complete original unchecked signature error and Signed success identity, logical views of cloned unit errors',
                authentication_boundary='supplied Keychain authenticated predicate and authenticate contract; ordinary checked UncheckedSigned::check adapter binds the original logical unit, creator and signature; native hash/as_ref/index/verify chain and cryptographic security are not proved',
                not_claimed='exact inference of cache-bearing clone outputs or maximality of the outer domain, automatic application policy discovery, signature implementation correctness, SCALE/hash security, full crate/protocol correctness')
        env = dict(os.environ, VERUS_Z3_PATH=str(z3), RUST_MIN_STACK='20971520',
                   VERUS_INFER_CACHE=str(output/'candidate-cache'))
        codec_lib, codec_vir = directory/'libcodec_model.rlib', directory/'codec_model.vir'
        guard.check()
        run = measured([str(verus), str(CODEC_MODEL), '--crate-name=codec_model', '--crate-type=lib',
            '--compile', '--export', str(codec_vir), '-o', str(codec_lib), '--output-json'], env, 90, ROOT)
        guard.check()
        (output/'codec.stdout.json').write_text(run['stdout'])
        (output/'codec.stderr.txt').write_text(run['stderr'])
        if run['returncode'] or not json.loads(run['stdout'])['verification-results']['success']:
            raise ValueError('declared codec model failed')
        codec_guard = ArtifactGuard([codec_lib, codec_vir])
        path, interface, library = directory/'validation.rs', directory/'validation.vir', directory/'libvalidation.rlib'
        command = [str(verus), str(path), '--crate-name=validation', '--crate-type=lib', '--compile',
            '--export', str(interface), '-o', str(library), '--infer-goals', '--no-cheating', '--output-json',
            '--extern', f'codec_model={codec_lib}', '--import', f'codec_model={codec_vir}',
            '-L', f'dependency={directory}']
        cases = [
            ('validation', api, True),
            ('wrong-own-payload', api.replace('Error::DescendantOfPreviousUnitHasWrongRound(parent_round)',
                'Error::DescendantOfPreviousUnitHasWrongRound(unit_coord.round)'), False),
            ('wrong-error-tag', api.replace('Error::NotEnoughParentsForRound(round - 1)',
                'Error::ParentsHigherThanRound(round - 1)'), False),
            ('ignored-error', api.replace('self.unit_creator_is_descendant_of_previous_unit(unit_coord)?;',
                'let _ = self.unit_creator_is_descendant_of_previous_unit(unit_coord);'), False),
            ('wrong-error-priority', api.replace(
                'self.previous_round_have_enough_parents(unit_coord.round)?;\n        self.check_if_parents_greater_than_previous_round(unit_coord.round)?;',
                'self.check_if_parents_greater_than_previous_round(unit_coord.round)?;\n        self.previous_round_have_enough_parents(unit_coord.round)?;'), False),
            ('wrong-query-callback', api.replace('lo <= *entry.1 && *entry.1 <= hi',
                'lo < *entry.1 && *entry.1 <= hi'), False),
            ('missing-creator-bound', api.replace('coord.parts().1 > 0, coord.parts().0.0 < self@.0.len(),',
                'coord.parts().1 > 0,'), False),
            ('missing-round-bound', api.replace('coord.parts().1 > 0, coord.parts().0.0 < self@.0.len(),',
                'coord.parts().0.0 < self@.0.len(),'), False),
            ('validation-warm', api, True),
        ]
        if args.native_pipelines:
            cases = [row for row in cases if row[0] != 'wrong-query-callback']
            cases.insert(-1, ('wrong-query-callback', api.replace('parent.round == round - 1', 'parent.round < round - 1'), False))
        if args.native_construction:
            cases.insert(-1, ('wrong-construction-round', api.replace('insert(parent_index, *parent_round)', 'insert(parent_index, 0)'), False))
            cases.insert(-1, ('wrong-construction-hash', api.replace('parent_map.using_encoded(H::hash)', 'H::hash(&[])'), False))
        if args.full_validation:
            cases.insert(-1, ('missing-public-creator-bound', api.replace(
                'coord.parts().1 == 0 || coord.parts().0.0 < self@.0.len(),',
                'true,'), False))
            cases.insert(-1, ('wrong-initial-payload', api.replace(
                'Error::RoundZeroWithSomeParents(NodeCount(parents_count))',
                'Error::RoundZeroWithSomeParents(NodeCount(0))'), False))
            cases.insert(-1, ('wrong-initial-hash-test', api.replace(
                'self.combined_hash != recalculated_control_hash',
                'self.combined_hash == recalculated_control_hash'), False))
            cases.insert(-1, ('wrong-initial-hash-payload', api.replace(
                'Error::RoundZeroBadControlHash(\n                self.combined_hash,\n                recalculated_control_hash,',
                'Error::RoundZeroBadControlHash(\n                recalculated_control_hash,\n                self.combined_hash,'), False))
            cases.insert(-1, ('wrong-round-dispatch', api.replace(
                '0 => self.validate_initial_round()', '1 => self.validate_initial_round()'), False))
        if args.parent_validation:
            cases.insert(-1, ('wrong-member-test', api.replace(
                'n_members != self.keychain.node_count()', 'n_members == self.keychain.node_count()'), False))
            cases.insert(-1, ('wrong-parent-error', api.replace(
                'ValidationError::ParentValidationFailed(pre_unit.clone(), e)',
                'ValidationError::ParentValidationFailed(pre_unit.clone(), Error::NotEnoughParentsForRound(0))'), False))
            cases.insert(-1, ('wrong-parent-copy-goal', api.replace(
                'copy@==pre@', 'copy@.1==pre@.1 + 1'), False))
            # Change just this call: other validators must retain their own ?.
            original = item(item(validator, 'impl<K: Keychain> Validator<K>')[2], 'fn validate_unit_parents<')[2]
            ignored = original.replace('pre_unit\n            .control_hash',
                'let _: core::result::Result<(),ValidationError<H,D,K::Signature>> = pre_unit\n            .control_hash').replace('))?;', '));')
            cases.insert(-1, ('ignored-parent-error', api.replace(original, ignored), False))
            cases.insert(-1, ('missing-parent-creator-bound', api.replace(
                '|| su.pre_value().0.0<su.pre_value().2.0.len(),', '|| true,'), False))
        if args.factory_proofs:
            cases.insert(-1, ('wrong-factory-index', api.replace(
                'Some((NodeIndex(idx), maybe_value.as_ref()?))',
                'Some((NodeIndex(0), maybe_value.as_ref()?))'), False))
            cases.insert(-1, ('wrong-factory-round', api.replace(
                'UnitCoord::new(round, node_index)', 'UnitCoord::new(0, node_index)'), False))
        if args.outer_validation:
            cases.insert(-1, ('wrong-authentication-adapter', api.replace(
                'if !keychain.authenticate(&self.signable,&self.signature)',
                'if keychain.authenticate(&self.signable,&self.signature)'), False))
            cases.insert(-1, ('ignored-authentication', api.replace(
                'let su = uu.check(&self.keychain)?;', 'let su = Signed{unchecked:uu};'), False))
            cases.insert(-1, ('wrong-signature-conversion', api.replace(
                'ValidationError::WrongSignature(se.unchecked)',
                'ValidationError::RoundTooHigh(se.unchecked.signable)'), False))
            cases.insert(-1, ('wrong-session-test', api.replace(
                'full_unit.session_id() != self.session_id', 'full_unit.session_id() == self.session_id'), False))
            cases.insert(-1, ('wrong-maximum-round', api.replace(
                'full_unit.round() > self.max_round', 'full_unit.round() >= self.max_round'), False))
            cases.insert(-1, ('wrong-session-payload', api.replace(
                'ValidationError::WrongSession(full_unit.clone())',
                'ValidationError::WrongSession(FullUnit::new(full_unit.as_pre_unit().clone(),None,0))'), False))
            session = item(item(validator, 'impl<K: Keychain> Validator<K>')[2], 'pub fn validate_unit<')[2]
            body = session.split('{',1)[1]
            a = body.index('        if full_unit.session_id()')
            b = body.index('        if full_unit.round()')
            c = body.index('        self.validate_unit_parents(su)')
            reordered = body[:a]+body[b:c]+body[a:b]+body[c:]
            cases.insert(-1, ('wrong-outer-priority', api.replace(body, reordered), False))
            cases.insert(-1, ('ignored-outer-parent-error', api.replace(
                'self.validate_unit_parents(su)\n    }', 'Ok(su)\n    }'), False))
            cases.insert(-1, ('missing-outer-creator-bound', api.replace(
                '|| unit@.0.0.0<unit@.0.2.0.len()', '|| true'), False))
        for case, text, expected in cases:
            if not expected and text == api:
                raise ValueError('ineffective mutation: '+case)
            path.write_text(text)
            (output/(case+'.rs')).write_text(text)
            guard.check(); codec_guard.check()
            run = measured(command, env, 120, ROOT)
            guard.check(); codec_guard.check()
            (output/(case+'.stdout.json')).write_text(run['stdout'])
            (output/(case+'.stderr.txt')).write_text(run['stderr'])
            data = validate(run, expected, inferred_count)
            if args.factory_proofs:
                assisted = data['verus-infer'].get('factory_assistance', [])
                if len(assisted) != 2 or any(f['callbacks'] != 1 for f in assisted):
                    raise ValueError('missing regenerated factory callback proofs')
                target = {'wrong-factory-index': 'NodeMap::iter',
                          'wrong-factory-round': 'ControlHash::parents'}.get(case)
                if target and not any(n.endswith('::'+target) for n in data['verus-infer']['failed_functions']):
                    raise ValueError('factory control did not fail its ordinary proof: '+case)
            if not expected:
                if interface.exists():
                    raise ValueError('failed provider published interface')
                if not any(s in run['stderr'] for s in ('postcondition not satisfied', 'precondition not satisfied', 'assertion failed',
                        'unable to prove post-condition of closure')):
                    raise ValueError('negative control lacks semantic proof failure')
            result['runs'].append(dict(case=case, verification=data['verification-results'],
                report=data['verus-infer'], wall_micros=run['wall_micros']))
            save()
            print(case+': expected outcome', flush=True)
        runtime = RUNTIME.read_text()
        if args.full_validation:
            runtime = runtime.replace('let result = control.validate_non_initial_round(',
                'let result = control.validate(')
        if args.native_pipelines:
            extra = '''    assert!(empty.check_if_parents_greater_than_previous_round(0).is_ok());
    let mut populated = ControlHash::<Hasher64> {
        parents: NodeMap::with_size(NodeCount(1)), combined_hash: [0; 8],
    };
    populated.parents.insert(NodeIndex(0), 0);
    assert!(std::panic::catch_unwind(|| populated.check_if_parents_greater_than_previous_round(0)).is_err());
    println!("NATIVE_PIPELINE_ZERO_ROUND=2");
'''
            runtime = runtime.replace('    println!("INFER_VALIDATION_CASES=', extra+'    println!("INFER_VALIDATION_CASES=')
        runtime_source = api + '\n#[cfg(verus_keep_ghost_body)] fn main() {}\n' + (
            '#[cfg(not(verus_keep_ghost_body))] mod runtime { use super::*;\n'+runtime+
            '\npub fn run(){validation_corpus();}}\n#[cfg(not(verus_keep_ghost_body))] fn main(){runtime::run();}\n')
        path.write_text(runtime_source)
        runtime_command = [arg for arg in command if arg != '--crate-type=lib']
        runtime_command[runtime_command.index(str(library))] = str(directory/'validation-runtime')
        guard.check(); codec_guard.check()
        run = measured(runtime_command, env, 120, ROOT)
        guard.check(); codec_guard.check()
        (output/'runtime.stdout.json').write_text(run['stdout'])
        (output/'runtime.stderr.txt').write_text(run['stderr'])
        data = validate(run, True, inferred_count)
        result['runs'].append(dict(case='runtime-build', verification=data['verification-results'],
            report=data['verus-infer'], wall_micros=run['wall_micros']))
        run = measured([str(directory/'validation-runtime')], env, 30, ROOT)
        (output/'runtime.txt').write_text(run['stdout']+run['stderr'])
        if run['returncode'] or 'INFER_VALIDATION_CASES=7704;ZERO_ROUND_PANIC=1' not in run['stdout'] or (args.native_pipelines and 'NATIVE_PIPELINE_ZERO_ROUND=2' not in run['stdout']):
            raise ValueError('adapted runtime corpus did not complete')
        result['native'].append(dict(case='adapted-runtime', returncode=run['returncode'], wall_micros=run['wall_micros']))
        if args.native_tests:
            end = control.rfind('}')
            native_runtime = runtime
            if args.native_construction:
                native_runtime = native_construction_runtime(runtime)
            if args.full_validation:
                native_runtime += (FIXTURES/'initial_validation_runtime.rs').read_text()
                native_runtime += '\n#[test] fn full_initial_validation_corpus(){initial_validation_corpus();}\n'
            (checkout/CONTROL).write_text(control[:end]+native_runtime+
                '\n#[test] fn inferred_validation_corpus(){validation_corpus();}\n'+control[end:])
            native_env = dict(os.environ)
            native_env.pop('RUSTUP_TOOLCHAIN', None)
            native_env['CARGO_TARGET_DIR'] = str((args.native_target or output/'native-target').resolve())
            guard.check(); codec_guard.check()
            run = measured(['cargo', 'test', '--offline', '-p', 'aleph-bft', '--lib',
                'units::control_hash::tests::', '--', '--test-threads=1', '--nocapture'], native_env, 600, checkout)
            guard.check(); codec_guard.check()
            (output/'native.stdout.txt').write_text(run['stdout'])
            (output/'native.stderr.txt').write_text(run['stderr'])
            validate_native(run, True, 'native-validation')
            if 'INFER_VALIDATION_CASES=7704;ZERO_ROUND_PANIC=1' not in run['stdout']:
                raise ValueError('untouched native corpus did not complete')
            if args.native_construction and 'NATIVE_CONSTRUCTORS=378;EMPTY=1' not in run['stdout']:
                raise ValueError('native constructor corpus did not execute')
            if args.full_validation and 'NATIVE_INITIAL_VALIDATION_CASES=254' not in run['stdout']:
                raise ValueError('native initial validation corpus did not complete')
            result['native'].append(dict(case='untouched-native', returncode=run['returncode'], wall_micros=run['wall_micros']))
            if args.parent_validation:
                if args.outer_validation:
                    native_validator = checkout/'consensus/src/units/validator.rs'
                    original = native_validator.read_text()
                    end = original.rfind('}')
                    native_validator.write_text(original[:end]+(FIXTURES/'outer_validation_runtime.rs').read_text()+original[end:])
                guard.check(); codec_guard.check()
                run = measured(['cargo', 'test', '--offline', '-p', 'aleph-bft', '--lib',
                    'units::validator::tests::', '--', '--test-threads=1', '--nocapture'], native_env, 600, checkout)
                guard.check(); codec_guard.check()
                (output/'native-validator.stdout.txt').write_text(run['stdout'])
                (output/'native-validator.stderr.txt').write_text(run['stderr'])
                validate_native(run, True, 'native-parent-validation')
                expected_tests = '7 passed' if args.outer_validation else '6 passed'
                if expected_tests not in run['stdout']:
                    raise ValueError('original native validator tests did not all execute')
                if args.outer_validation and 'NATIVE_OUTER_VALIDATION_CASES=95;CREATOR_PANIC=1' not in run['stdout']:
                    raise ValueError('native outer validation corpus did not complete')
                result['native'].append(dict(case='untouched-native-validator', returncode=run['returncode'], wall_micros=run['wall_micros']))
        result['success'] = True
    except Exception as error:
        result['success'], result['error'] = False, str(error)
        raise
    finally:
        save()


if __name__ == '__main__':
    main()
