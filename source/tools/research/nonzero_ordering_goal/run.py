"""Original NonZero ordering baseline; raw products stay in a fresh private /tmp root.
Run with python3 -O -B. No historical tool path is followed from a receipt.
"""
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[3]
BUNDLE = Path('/tmp/verus-infer-buffer-cycle-20261109/parent-combined72-bundle-5d79f711')
BUNDLE_SHA = '43b29d75d63b897675449ac1bfd56024c4245ff6475d88ef85563349b495f679'
TC = Path('/home/dev/.rustup/toolchains/1.99.1-x86_64-unknown-linux-gnu')
DECL = '#[derive(Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]\npub struct BorTag(NonZero<u64>);'
DECL_SHA = '0107890f5ffee6b94a4179c0351a4ff121e0e61b9886067e94370a65391e9abd'
TOOLS = {'verus', 'rust_verify', 'z3', 'vstd.vir', 'libvstd.rlib', 'libverus_builtin.rlib',
         'libverus_builtin_macros.so', 'libverus_state_machines_macros.so'}
SUPPORT = ['source/builtin_macros/src/infer_derive_ops.rs', 'source/vir/src/infer/wrapper_compare.rs',
           'source/vir/src/infer/wrapper_binary.rs', 'source/vir/src/infer/newtypes.rs',
           'source/rust_verify/src/automatic_derive.rs', 'source/rust_verify/src/checked_foreign.rs',
           'source/rust_verify/src/rust_to_vir_impl.rs', 'source/vstd/std_specs/nonzero.rs',
           'source/vstd/std_specs/cmp.rs']

def need(ok, message):
    if not ok:
        raise RuntimeError(message)

def sha(p):
    with Path(p).open('rb') as f:
        return hashlib.file_digest(f, 'sha256').hexdigest()

def write(p, value):
    Path(p).write_text(json.dumps(value, indent=2, sort_keys=True) + '\n')

def authenticate():
    need(sys.flags.dont_write_bytecode, 'use Python -B')
    need(sha(BUNDLE/'receipt.json') == BUNDLE_SHA, 'bundle receipt drift')
    br = json.loads((BUNDLE/'receipt.json').read_text())
    need(br['success'] and br['all_final_guards_passed'], 'accepted bundle required')
    need(set(br['tools']) == TOOLS, 'exact eight tools required')
    for n, h in br['tools'].items():
        need(sha(BUNDLE/n) == h, 'immutable tool drift: ' + n)
    # Rebind source by own/Git bytes, never the historical build's paths.
    for p in SUPPORT:
        own = (ROOT/p).read_bytes()
        accepted = subprocess.check_output(['git', 'show', '5d79f711:'+p], cwd=ROOT)
        need(own == accepted, 'engine source differs: '+p)
    pin = json.loads((ROOT/'source/tools/research/native_miri_policy_pin/pins.json').read_text())
    row = pin['sources']['src/tools/miri/src/borrow_tracker/mod.rs']
    upstream = Path(row['path'])
    need(sha(upstream) == row['sha256'], 'pinned source drift')
    need(upstream.read_text().count(DECL) == 1, 'original declaration absent')
    need(hashlib.sha256(DECL.encode()).hexdigest() == DECL_SHA, 'declaration drift')
    return upstream

PRE = 'use vstd::prelude::*;\nuse std::num::NonZero;\nuse core::cmp::Ordering;\n'

def clients(typ, view):
    out = ''
    for name, op in [('less', '<'), ('less_equal', '<='), ('greater', '>'), ('greater_equal', '>=')]:
        out += f'pub fn {name}(a:&{typ},b:&{typ})->(r:bool)\n ensures r == ({view[0]} {op} {view[1]}),\n{{ a {op} b }}\n'
    out += f'''pub fn partial(a:&{typ},b:&{typ})->(r:Option<Ordering>)
 ensures r == Some(vstd::std_specs::cmp::OrdSpec::cmp_spec(&{view[0]},&{view[1]})),
{{ a.partial_cmp(b) }}
'''
    return out

def cmp_client(typ, view):
    return f'''pub fn total(a:&{typ},b:&{typ})->(r:Ordering)
 ensures r == vstd::std_specs::cmp::OrdSpec::cmp_spec(&{view[0]},&{view[1]}),
{{ a.cmp(b) }}
'''

def source(case):
    if case == 'carrier_partial':
        return PRE+'verus! {\n'+clients('NonZero<u64>', ('a@','b@'))+'}\n'
    if case == 'carrier_ord':
        return PRE+'verus! {\n'+cmp_client('NonZero<u64>', ('a@','b@'))+'}\n'
    if case == 'original_calls':
        return PRE+'#[verus_verify]\n'+DECL+'\nverus! {\npub closed spec fn value(a:&BorTag)->u64 { a.0@ }\n'+clients('BorTag', ('value(a)','value(b)'))+cmp_client('BorTag', ('value(a)','value(b)'))+'}\n'
    if case == 'request':
        return PRE+'#[verus_verify(infer_derive(PartialEq,PartialOrd,Ord))]\n'+DECL+'\n'
    if case == 'ordinary_derive':
        # Authored numeric semantics select the ORIGINAL generated ordering bodies.
        # No runtime impl, observation lemma, assumed law or proof block is added.
        return PRE+'#[verus_verify]\n'+DECL+'''\nverus! {
impl BorTag {
 pub open spec fn __verus_infer_derived_cmp(&self)->bool {true}
 pub open spec fn __verus_infer_derived_partial_cmp(&self)->bool {true}
}
impl vstd::std_specs::cmp::OrdSpecImpl for BorTag {
 open spec fn obeys_cmp_spec()->bool {true}
 closed spec fn cmp_spec(&self,rhs:&Self)->Ordering {
  vstd::std_specs::cmp::OrdSpec::cmp_spec(&self.0@,&rhs.0@)
 }
}
impl vstd::std_specs::cmp::PartialOrdSpecImpl for BorTag {
 open spec fn obeys_partial_cmp_spec()->bool {true}
 closed spec fn partial_cmp_spec(&self,rhs:&Self)->Option<Ordering> {
  Some(vstd::std_specs::cmp::OrdSpec::cmp_spec(&self.0@,&rhs.0@))
 }
}
}\n'''
    if case in ['eq', 'unrelated_eq']:
        text = PRE+'#[verus_verify(infer_derive(PartialEq))]\n'+DECL+'''\nverus! {
pub closed spec fn value(a:&BorTag)->u64 { a.0@ }
pub fn equal(a:&BorTag,b:&BorTag)->(r:bool)
 ensures r == (value(a) == value(b)),
{ a == b }
}\n'''
        return text if case == 'eq' else text.replace('BorTag','CounterKey')
    raise ValueError(case)

def environment(out):
    return dict(os.environ, PATH='/home/dev/.cargo/bin:/usr/bin:/bin', RUSTUP_TOOLCHAIN='1.99.1',
                RUSTC_BOOTSTRAP='1', LD_LIBRARY_PATH=str(TC/'lib'), CARGO_BUILD_JOBS='2',
                CARGO_NET_OFFLINE='true', VERUS_Z3_PATH=str(BUNDLE/'z3'),
                VERUS_INFER_CACHE=str(out/'cache'), TMPDIR=str(out), XDG_CACHE_HOME=str(out/'cache'))

def execute(out, label, argv, inputs, env):
    d = out/label
    d.mkdir()
    before = {str(p):sha(p) for p in inputs}
    freeze = dict(argv=list(map(str,argv)), cwd=str(d), inputs=before,
                  environment={k:env[k] for k in ['PATH','RUSTUP_TOOLCHAIN','RUSTC_BOOTSTRAP','LD_LIBRARY_PATH','CARGO_BUILD_JOBS','CARGO_NET_OFFLINE','VERUS_Z3_PATH','VERUS_INFER_CACHE','TMPDIR','XDG_CACHE_HOME']},
                  before_execution=True)
    write(d/'freeze.json',freeze)
    freeze_sha = sha(d/'freeze.json')
    with (d/'stdout').open('wb') as stdout, (d/'stderr').open('wb') as stderr:
        p = subprocess.run(freeze['argv'], cwd=d, env=env, stdout=stdout, stderr=stderr, timeout=180)
    need(before == {str(p):sha(p) for p in inputs}, 'input changed')
    need(sha(d/'freeze.json') == freeze_sha, 'freeze changed')
    err = (d/'stderr').read_text()
    need(not any(x in err.lower() for x in ['panicked at','internal compiler error','resource limit','timed out']), 'compiler/resource failure')
    row = dict(exit=p.returncode, input_guards_passed=True, freeze_sha256=freeze_sha,
               stdout_sha256=sha(d/'stdout'), stderr_sha256=sha(d/'stderr'))
    try:
        data = json.loads((d/'stdout').read_text())
        row['verification'] = data['verification-results']
        row['inference'] = data['verus-infer']
    except (ValueError, KeyError):
        pass
    write(d/'receipt.json',row)
    print(label, json.dumps({k:v for k,v in row.items() if k not in ['inference']}),flush=True)
    return row

def main():
    upstream = authenticate()
    out = Path(tempfile.mkdtemp(prefix='nonzero-ordering-baseline-'))
    out.chmod(0o700)
    print(out,flush=True)
    env = environment(out)
    inputs = [BUNDLE/'receipt.json', *[BUNDLE/n for n in sorted(TOOLS)], upstream,
              TC/'bin/rustc', Path(__file__), *[ROOT/p for p in SUPPORT]]
    write(out/'freeze.json', dict(inputs={str(p):sha(p) for p in inputs}, declaration_sha256=DECL_SHA,
          base='131ffc3575ab340a4b74292c0d51cd4ee7f6adee', versions=dict(schema=39,policy=72,cache=75),
          domain='every genuine NonZero<u64>, up to u64::MAX; no extra premise',
          before_first_provider=True, publication=False,goal_complete=False))
    inputs.append(out/'freeze.json')
    results = {}
    for case in ['carrier_partial','carrier_ord','original_calls','request','ordinary_derive','eq','unrelated_eq']:
        src = out/(case+'.rs')
        src.write_text(source(case))
        argv = [BUNDLE/'verus',src,'--crate-type=lib','--no-cheating','--output-json','--infer-goals',
                '--multiple-errors','20','--log-all','--log-dir',out/(case+'-vir'),
                '--export',out/(case+'.vir'),'--compile','-o',out/('lib'+case+'.rlib')]
        results[case] = execute(out,case,argv,inputs+[src],env)
        if case in ['eq','unrelated_eq']:
            results[case+'_warm'] = execute(out,case+'_warm',argv,inputs+[src],env)
    raw = out/'native.rs'
    raw.write_text('use std::num::NonZero;\n'+DECL+'\n'+'''pub fn less(a:&BorTag,b:&BorTag)->bool {a < b}
pub fn less_equal(a:&BorTag,b:&BorTag)->bool {a <= b}
pub fn greater(a:&BorTag,b:&BorTag)->bool {a > b}
pub fn greater_equal(a:&BorTag,b:&BorTag)->bool {a >= b}
pub fn partial(a:&BorTag,b:&BorTag)->Option<core::cmp::Ordering> {a.partial_cmp(b)}
pub fn total(a:&BorTag,b:&BorTag)->core::cmp::Ordering {a.cmp(b)}
''')
    for mode, flags in [('expanded',['-Zunpretty=expanded']),('mir',['--emit=mir','-o',str(out/'native.mir')])]:
        results[mode] = execute(out,mode,[TC/'bin/rustc',raw,'--crate-type=lib','--edition=2021',*flags],inputs+[raw],env)
        need(results[mode]['exit']==0, 'native compiler failed')
    write(out/'receipt.json', dict(results=results, private_root=str(out), tool_receipt_sha256=BUNDLE_SHA,
          final_inputs={str(p):sha(p) for p in inputs}, publication=False,goal_complete=False))

if __name__ == '__main__':
    main()
