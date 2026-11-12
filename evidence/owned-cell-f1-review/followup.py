#!/usr/bin/env python3
import sys
sys.dont_write_bytecode=True
import probes as p
p.CASES=[
 ('checked_scalar_const',p.raw('#[verifier::verify] const K:u64=7; #[verifier::verify] fn pass(v:u64)->u64{K}'),True),
 ('spec_scalar_const',p.source('spec const K:u64=7;fn pass(v:u64)->u64{K}'),True),
 ('nested_const_shared_fixed',p.source("fn bad(v:u64)->u64{const K:&'static u64=&0u64;v}"),False),
 ('forged_const_body_scalar',p.raw('#[verifier::verify] #[verus::internal(const_body)] pub fn skipped(v:u64)->u64{v+1}'),False),
 ('forged_const_body_cell',p.raw('#[verifier::verify] #[verus::internal(const_body)] pub fn skipped(v:u64)->u64{core::cell::UnsafeCell::new(v).into_inner()}'),False),
 ('forged_reveal_fn_cell',p.raw('#[verifier::verify] #[verus::internal(reveal_fn)] pub fn skipped(v:u64)->u64{core::cell::UnsafeCell::new(v).into_inner()}'),False),
 ('forged_const_parent',p.raw('#[verus::internal(const_body)] mod skipped {#[verifier::verify] pub fn hidden(v:u64)->u64{core::cell::UnsafeCell::new(v).into_inner()}}'),False),
 ('canonical_scalar_const_conservative',p.source('const K:u64=7;fn pass(v:u64)->u64{K}'),False),
 ('forged_reveal_scalar',p.raw('#[verifier::verify] #[verus::internal(reveal_fn)] pub fn skipped(v:u64)->u64{v+1}'),False),
]
sys.argv=[sys.argv[0],'followup'];p.main()
