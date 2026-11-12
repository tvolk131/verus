#!/usr/bin/env python3
"""Independent bounded F1 source witnesses, using existing experiment helpers."""
import json,os,sys,time
sys.dont_write_bytecode=True
from pathlib import Path
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[1]
sys.path.insert(0,str(ROOT/'source/tools'))
sys.path.insert(0,str(ROOT/'source/tools/research/redb_crash_protocol/native_state_link/implementation'))
from infer_eval_support import ArtifactGuard,digest
from infer_experiment import LoggedCommands,replay_receipt,require
import check
LATER='inactive: F1 admission passed;'
def source(s):return 'use vstd::prelude::*;use core::cell::UnsafeCell;type C=UnsafeCell<u64>;verus!{'+s+'}'
def raw(s):return 'use vstd::prelude::*;'+s
CASES=[
 ('recursive_scalar_tuple',source('fn pass(v:((u64,bool),()))->(r:((u64,bool),())) ensures r==v, {v}'),True),
 ('reexport_alias', 'use vstd::prelude::*;use std::cell::UnsafeCell as Native;type One=Native<u64>;type Two=One;verus!{fn pass(v:u64)->u64{let c=Two::new(v);c.into_inner()}}',True),
 ('exclusive_cell_return',source('fn pass(c:&mut C)->&mut C{c}'),True),
 ('local_const_scalar',source('const K:u64=7;fn pass(v:u64)->u64{K}'),True),
 ('quant_scalar_capture',source('spec fn eq(a:u64,b:u64)->bool{a==b} fn pass(v:u64)->(r:u64) ensures forall|a:u64| eq(a,a)&&r==v, {v}'),True),
 ('requires_cell_capture',source('fn bad(c:C)->u64 requires {let x=c;true}, {0}'),False),
 ('selector_spec_reference',source('spec fn live(c:&C)->u64{0} fn bad(v:u64)->u64{v}'),False),
 ('empty_quant_cell',source('fn bad(v:u64)->(r:u64) ensures forall|_:C| true, {v}'),False),
 ('quant_hidden_capture',source('fn bad(c:C)->(r:u64) ensures forall|n:u64| {let ghost x=c;true}, {0}'),False),
 ('nested_spec_capture',source('fn bad(c:C)->(r:u64) ensures {let ghost s=c;forall|n:u64| true}, {0}'),False),
 ('spec_reference_binder',source('fn bad(v:u64)->(r:u64) ensures forall|x:&u64| true, {v}'),False),
 ('unused_raw_alias', 'type P=*mut u64;'+source('fn ok(v:u64)->u64{v}'),False),
 ('unused_shared_alias', 'type P=&\'static UnsafeCell<u64>;'+source('fn ok(v:u64)->u64{v}'),False),
 ('unused_cell_tuple_alias','type Hidden=(C,u64);'+source('fn ok(v:u64)->u64{v}'),False),
 ('unused_generic_alias','type Hidden<T>=UnsafeCell<T>;'+source('fn ok(v:u64)->u64{v}'),False),
 ('nested_inline_const',source('fn bad(v:u64)->u64{let k=const {0u64};v}'),False),
 ('nested_const_shared',source('fn bad(v:u64)->u64{const K:&u64=&0u64;v}'),False),
 ('nested_unused_static',source('fn bad(v:u64)->u64{static K:u64=0;v}'),False),
 ('dead_unverified_nested',raw('#[verifier::verify] fn bad(v:u64)->u64{if false{fn hidden(c:&core::cell::UnsafeCell<u64>){};}v}'),False),
 ('skipped_scalar_unrelated',raw('#[verifier::external_body] fn unrelated(v:u64)->u64{v} ')+source('fn ok(v:u64)->u64{v}'),False),
 ('skipped_ancestor_scalar',raw('#[verifier::external] mod hidden {#[verifier::verify] pub fn inner(v:u64)->u64{v}} ')+source('fn ok(v:u64)->u64{v}'),False),
 ('native_scalar_unrelated', 'fn ignored(v:u64)->u64{v} '+source('fn ok(v:u64)->u64{v}'),False),
 ('dead_raw_addr',source('fn bad(c:&mut C)->u64{if false {let p=&raw mut *c;}0}'),False),
 ('promoted_shared_scalar',source('fn bad(v:u64)->u64{let ghost p=&(0u64,true);v}'),False),
 ('raw_type_ascription',source('fn bad(v:u64)->u64{let p:*const u64=core::ptr::null();v}'),False),
 ('foreign_block', 'unsafe extern "C" {fn hidden(v:u64)->u64;}'+source('fn ok(v:u64)->u64{v}'),False),
 ('local_opaque',source('fn bad(v:u64)->impl Sized{v}'),False),
 ('real_recursion_in_predicate',source('spec fn looped(v:u64)->bool{looped(v)} fn bad(v:u64)->(r:u64) ensures looped(r), {v}'),False),
 ('recursion_in_dead_branch',source('fn bad(v:u64)->u64{if false{bad(v)}else{v}}'),False),
 ('builtin_alias_header',raw('use verus_builtin::{ensures as E,constrain_type as T};#[verifier::verify] fn pass(v:u64)->u64{E(|r:u64|[T(r,pass(v)),r==v]);v}'),True),
 ('second_constrain_real_call',raw('#[verifier::verify] fn bad(v:u64)->u64{verus_builtin::ensures(|r:u64|[verus_builtin::constrain_type(r,bad(v)),verus_builtin::constrain_type(r,bad(v))]);v}'),False),
 ('constrain_mutated_argument',raw('#[verifier::verify] fn bad(v:u64)->u64{verus_builtin::ensures(|r:u64|[verus_builtin::constrain_type(r,bad({v})),true]);v}'),False),
 ('fake_ensures',raw('#[verifier::verify] fn ensures(v:u64)->u64{v} #[verifier::verify] fn bad(v:u64)->u64{let x=ensures;v}'),False),
 ('header_real_cell_predicate',raw('#[verifier::verify] fn bad(c:core::cell::UnsafeCell<u64>)->u64{verus_builtin::ensures(|r:u64|[verus_builtin::constrain_type(r,bad(c)),{let x=c;true}]);0}'),False),
 ('header_extra_stmt',raw('#[verifier::verify] fn bad(v:u64)->u64{verus_builtin::ensures(|r:u64|{let a=v;[verus_builtin::constrain_type(r,bad(v)),true]});v}'),False),
 ('header_shared_ret',raw('#[verifier::verify] fn bad(c:&mut core::cell::UnsafeCell<u64>)->&mut u64{verus_builtin::ensures(|r:&mut u64|[verus_builtin::constrain_type(r,bad(c)),true]);c.get_mut()}'),False),
 ('header_move_capture',raw('#[verifier::verify] fn pass(v:u64)->u64{verus_builtin::ensures(move |r:u64|[verus_builtin::constrain_type(r,pass(v)),r==v]);v}'),True),
 ('spec_builtin_fn_value',source('fn bad(v:u64)->(r:u64) ensures {let f=verus_builtin::spec_eq::<u64,u64>;true}, {v}'),False),
 ('const_cell',source('const H:C=C::new(0);fn ok(v:u64)->u64{v}'),False),
 ('dead_async',source('fn bad(v:u64)->u64{if false {let f=async {v};}v}'),False),
 ('proof_fn_unused',source('proof fn hidden(){} fn ok(v:u64)->u64{v}'),False),
 ('import_unknown_scalar',source('fn bad(v:u64)->u64{core::hint::black_box(v)}'),False),
]
def main():
 out=HERE/(sys.argv[1] if len(sys.argv)>1 else 'independent');out.mkdir()
 hs=json.loads((HERE/'AUTHENTICATION.json').read_text())['tools'];b=HERE/'private/bundle'
 paths=[*[b/n for n in hs],Path(__file__),ROOT/'source/rust_verify/src/checked_owned_cell_admission.rs',ROOT/'source/rust_verify/src/fn_call_to_vir.rs']
 guard=ArtifactGuard(paths);r=dict(commands=[],cases=[])
 env=check.env(HERE/'private/environment');env['VERUS_ROOT']=str(b);env['VERUS_Z3_PATH']=str(b/'z3')
 with replay_receipt(out,r):
  for label,s,positive in CASES:
   for infer in [False,True]:
    d=out/(label+('-infer' if infer else '-ordinary'));d.mkdir()
    p=d/'input.rs';p.write_text(s)
    argv=[str(b/'verus'),str(p),'--crate-type=lib','--crate-name=probe','--edition=2024','--no-cheating','--compile','--experimental-owned-unsafe-cell','--output-json','-o',str(d/'libprobe.rlib')]
    if infer:argv+=['--infer-goals']
    env['VERUS_INFER_CACHE']=str(d/'fresh-cache')
    code=LoggedCommands(d,ROOT,r['commands'],timeout=60)(label,argv,env)
    err=(d/(label+'.stderr')).read_text();stdout=(d/(label+'.stdout')).read_text()
    try:j=json.loads(stdout)
    except ValueError:j={}
    vi=j.get('verus-infer',{});scope=vi.get('verification_scope',{})
    empty=j.get('verification-results',{}).get('success') is False and j.get('func-details')=={} and vi.get('functions')==[] and vi.get('requested_functions')==0 and vi.get('generated_functions')==0 and scope.get('checked_exec_bodies')==0 and scope.get('checked_proof_bodies')==0 and vi.get('interface_written') is False
    admitted=LATER in err
    typed='experimental owned UnsafeCell admission:' in err
    r['cases'].append(dict(label=label,infer=infer,expected_positive=positive,admitted=admitted,typed_admission_refusal=typed,empty_inventory=empty,exit=code,source_sha256=digest(p),classification_ok=(code!=0 and admitted==positive and empty and (positive or typed))))
  guard.check();r['guard_passed']=True
  r['unexpected']=[c for c in r['cases'] if not c['classification_ok']]
 print('unexpected',[(c['label'],c['infer']) for c in r['unexpected']]);print('cases',len(r['cases']))
if __name__=='__main__':main()
