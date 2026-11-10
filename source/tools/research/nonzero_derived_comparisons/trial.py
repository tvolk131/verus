"""Source-frozen checked-comparison trials; historical paths are receipt keys only."""
import argparse
import json
import os
from pathlib import Path
import sys
import tempfile

sys.dont_write_bytecode = True
HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[3]
sys.path.insert(0, str(HERE.parents[1]))
from infer_eval_support import ArtifactGuard, digest, tool_artifacts
from infer_experiment import LoggedCommands, replay_receipt, require

BUNDLE = Path('/tmp/verus-infer-buffer-cycle-20261109/parent-stable-key-bundle-a93b774a')
RECEIPT = Path('/tmp/verus-infer-buffer-cycle-20261109/stable-key-validation-a93b774a/canonical/canonical-zgtzh85h/receipt.json')
RECEIPT_SHA = '2dda5d93603528fe2623f4e37bc7cbaa48a537e3184c653a1abe1a4028f4ed3a'
UPSTREAM = Path('/tmp/native-miri-policy-pin-20261110/sources/src/tools/miri/src/borrow_tracker/mod.rs')
UPSTREAM_SHA = '8dec93f2a161caff23bbae3d52ee95eaf690d5d921d46847caf80ceb5d29a896'
RUSTC = Path('/home/dev/.rustup/toolchains/1.99.1-x86_64-unknown-linux-gnu/bin/rustc')
TOOLS = {'verus','rust_verify','vstd.vir','libvstd.rlib','libverus_builtin.rlib',
         'libverus_builtin_macros.so','libverus_state_machines_macros.so','z3'}


def authenticate():
    require(digest(RECEIPT) == RECEIPT_SHA, 'canonical receipt drift')
    accepted = json.loads(RECEIPT.read_bytes())
    require(accepted['success'] is True, 'canonical success absent')
    files = tool_artifacts(BUNDLE/'verus', BUNDLE/'z3')
    require({p.name for p in files if p.parent == BUNDLE} == TOOLS, 'eight tools required')
    for p in files:
        keys = ([str(p)] if p.parent == BUNDLE else
                [k for k in accepted['artifacts'] if k.endswith('/source/tools/'+p.name)])
        require(len(keys) == 1, 'ambiguous support label')
        require(digest(p) == accepted['artifacts'][keys[0]]['sha256'], 'tool/support drift: '+str(p))
    require(digest(UPSTREAM) == UPSTREAM_SHA, 'upstream drift')
    return files


def source(case):
    text = UPSTREAM.read_text()
    original = '#[derive(Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]\npub struct BorTag(NonZero<u64>);'
    require(text.count(original) == 1, 'genuine declaration absent')
    carrier = '#[verus_verify]\n'+original
    prelude = 'use vstd::prelude::*;\nuse std::num::NonZero;\n'
    view = 'spec fn tag_value(t: &BorTag) -> u64 { t.0@ }\n'
    identity = '''proof fn identity(a: &BorTag, b: &BorTag)
    ensures (a.0@ == b.0@) == (*a == *b),
{
    vstd::std_specs::nonzero::axiom_view_nonzero_from_primitive_eq(a.0);
    vstd::std_specs::nonzero::axiom_view_nonzero_from_primitive_eq(b.0);
}
'''
    eq_model = '''impl BorTag {pub open spec fn __verus_infer_derived_eq(&self)->bool {true}}
impl vstd::std_specs::cmp::PartialEqSpecImpl for BorTag {
    open spec fn obeys_eq_spec()->bool {true}
    closed spec fn eq_spec(&self,rhs:&Self)->bool {self.0@ == rhs.0@}
}
'''
    goal = '''fn compare_equal(a:&BorTag,b:&BorTag)->(r:bool)
    ensures r == (tag_value(a) == tag_value(b)),
{ a == b }
'''
    if case == 'standard_eq':
        return prelude+'''verus! {
fn standard_equal(a:&NonZero<u64>,b:&NonZero<u64>)->(r:bool)
    ensures r == (a@ == b@),
{ a == b }
}
'''
    if case == 'identity':
        return prelude+carrier+'\nverus! {\n'+view+identity+'}\n'
    if case == 'unmarked':
        return prelude+carrier+'\nverus! {\n'+view+identity+goal+'}\n'
    if case == 'typed_eq':
        return prelude+carrier+'\nverus! {\n'+view+identity+eq_model+goal+'}\n'
    if case == 'typed_ord':
        return prelude+carrier+'''\nverus! {
impl BorTag {pub open spec fn __verus_infer_derived_cmp(&self)->bool {true}}
impl vstd::std_specs::cmp::OrdSpecImpl for BorTag {
    open spec fn obeys_cmp_spec()->bool {true}
    closed spec fn cmp_spec(&self,rhs:&Self)->core::cmp::Ordering {
        vstd::std_specs::cmp::OrdSpec::cmp_spec(&self.0,&rhs.0)
    }
}
}
'''
    raise ValueError('unknown case')


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('case', choices=['standard_eq','identity','unmarked','typed_eq','typed_ord'])
    ap.add_argument('--off', action='store_true')
    a = ap.parse_args()
    out = Path(tempfile.mkdtemp(prefix='nonzero-derive-'+a.case+'-', dir='/tmp'))
    out.chmod(0o700)
    rec = dict(schema=1, case=a.case, output=str(out), commands=[], publication=False,
               goal_complete=False, source_acceptance=False)
    print(str(out), flush=True)
    with replay_receipt(out, rec):
        files = authenticate()
        src = out/'component.rs'
        src.write_text(source(a.case))
        (out/'trial.py').write_bytes(Path(__file__).read_bytes())
        inputs = files+[RECEIPT, UPSTREAM, RUSTC, Path(sys.executable).resolve(), src,
                        Path(__file__), out/'trial.py']
        inputs += [ROOT/'source/builtin_macros/src/infer_derive_ops.rs',
                   ROOT/'source/vir/src/infer/wrapper_compare.rs',
                   ROOT/'source/vir/src/infer/wrapper_binary.rs',
                   ROOT/'source/vir/src/infer/newtypes.rs',
                   ROOT/'source/rust_verify/src/automatic_derive.rs',
                   ROOT/'source/rust_verify/src/external.rs',
                   ROOT/'source/vstd/std_specs/nonzero.rs', ROOT/'source/vstd/std_specs/cmp.rs']
        guard = ArtifactGuard(inputs)
        env = dict(os.environ, PATH='/home/dev/.cargo/bin:'+os.environ['PATH'], RUSTUP_TOOLCHAIN='1.99.1', VERUS_Z3_PATH=str(BUNDLE/'z3'),
                   VERUS_INFER_CACHE=str(out/'cache'), TMPDIR=str(out), XDG_CACHE_HOME=str(out/'cache'),
                   CARGO_TARGET_DIR=str(out/'target'))
        argv = [str(BUNDLE/'verus'),str(src),'--crate-type=lib','--no-cheating','--output-json',
                '--multiple-errors','20','--log-all','--log-dir',str(out/'vir')]
        if not a.off:
            argv += ['--infer-goals']
        freeze = dict(inputs=guard.initial, argv=argv, environment={k:env[k] for k in
                      ['RUSTUP_TOOLCHAIN','VERUS_Z3_PATH','VERUS_INFER_CACHE','TMPDIR','XDG_CACHE_HOME','CARGO_TARGET_DIR']},
                      before_first_trial=True, full_domain='every genuine NonZero<u64>',
                      requested_helpers=['<BorTag as PartialEq>::eq'] if a.case=='typed_eq' else [])
        (out/'input-freeze.json').write_text(json.dumps(freeze,indent=2)+'\n')
        fg = ArtifactGuard([out/'input-freeze.json'])
        rec['freeze_sha256'] = digest(out/'input-freeze.json')
        run = LoggedCommands(out,out,rec['commands'],timeout=120,guard=guard)
        try:
            rc = run('check',argv,env)
            rec['terminal_exit'] = rc
            stderr = (out/'check.stderr').read_text()
            require(not any(s in stderr.lower() for s in ['panicked at','timed out','resource limit','internal compiler error']), 'resource/compiler failure')
            data = json.loads((out/'check.stdout').read_text())
            rec['verification'] = data['verification-results']
            inf = data['verus-infer']
            rec['inference'] = inf
            rec['classification'] = ('frontend_or_profile_barrier' if
                rec['verification'].get('encountered-error') or rec['verification'].get('encountered-vir-error') else
                'ordinary_checked_proof' if rc==0 else 'ordinary_proof_failure')
            require(inf['interface_written'] is False, 'unexpected interface publication')
            print(json.dumps(dict(exit=rc,verification=rec['verification'],classification=rec['classification'])),flush=True)
        finally:
            guard.check(); fg.check()
            rec['final_guards_passed'] = True
            rec['artifacts'] = guard.initial
            rec['products'] = {str(p.relative_to(out)):digest(p) for p in sorted(out.rglob('*'))
                               if p.is_file() and p.name != 'receipt.json'}

if __name__ == '__main__':
    main()
