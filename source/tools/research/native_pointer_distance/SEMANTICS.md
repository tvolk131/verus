# Meaning and boundary of the selected intrinsic

The installed Rust compiler is rustc 1.99.1,
commit `58cf8f9c35b355199727627d07ae34a4e2846d67`, LLVM23.1.1,
x86_64-unknown-linux-gnu. `pins.json` authenticates the completed extractor's
actual rustc/core dependency identities, the completed full graph and the core
source hashes; compiler sources were downloaded at that exact commit into a
private read-only input directory. Historical checkout paths are never opened.

## Justification of the arithmetic interpretation

The pinned compiler's
[SSA intrinsic lowering](https://github.com/rust-lang/rust/blob/58cf8f9c35b355199727627d07ae34a4e2846d67/compiler/rustc_codegen_ssa/src/mir/intrinsic.rs#L564)
gets argument0 and argument1 in that order, converts both pointers to integers
of pointer width, computes unsigned subtraction with an underflow prohibition,
then exact unsigned division by the pointee layout size. The pinned
[LLVM builder](https://github.com/rust-lang/rust/blob/58cf8f9c35b355199727627d07ae34a4e2846d67/compiler/rustc_codegen_llvm/src/builder.rs#L520)
maps these calls to LLVMBuildNUWSub and LLVMBuildExactUDiv (lines520/539).
Thus neither sign extension nor unconstrained mathematical pointer subtraction
is the selected arithmetic. For u8 the divisor is1. A valid unsigned address
pair b<=a within64 bits makes subtraction free of underflow; exact division by1
is total and returns the difference. `unsigned_sub` and `exact_udiv` expose
those side conditions, including the actual modulo-2^bits arithmetic.
`span_bounds`/`lowered_u8_exact` PROVE that the conditional span supplies them
and the result is exactly the supplied length. The signed byte-distance bound
is proved separately, not inferred from the unsigned return type.

Core `ptr/const_ptr.rs:550–570,645–733` requires same allocation/in-bounds
between nonidentical endpoints, or equal addresses, and byte distance <=isizeMAX.
Core `intrinsics/mod.rs:2460–2464` declares the compiler intrinsic; it supplies
no checked implementation theorem. A legal span retains one containing model
allocation plus native window address and offset. End formation calls the
previously checked `allocated_formation` dependencies / `offset_result_bounds`
and `pointer_loc_shift` rather than redefining Offset. Strict-interior [128]
has arbitrary surrounding allocation bytes and requires no allocation-base
identity. One-past is legal to form and is not a readable array cell. Byte
alignment is1; the window's positive address ensures nonnull ends without
requiring any pad/key byte value to be zero.

An empty span may use an `EmptyAddress` with only nonnull64-bit representation.
It has no block, live allocation, heap lookup or readable-cell premise. Equal
endpoints produce0, including the same address as the pad. Nonempty spans
require the SAME containing allocation and in-bounds interval. The source-entry
worker must derive the span from actual iterator fields; the span is never a
new application precondition. The representation is deliberately not a physical
permission, lawful-history definition or future-defined-execution restriction.

The downloaded const-interpreter intrinsic implementation is retained as
corroborating source, not used as a new rule. It computes relative offsets,
checks signed distance and in-allocation ranges, then exact-divides. This lane
proves the runtime chain selected in the completed MIR. The u8 IS_ZST/SIZE
constants are compiler-evaluated facts checked in the extraction. Core's
const_eval_select uses a distinct const-evaluation path for the optional order
check; no claim about running this entire writer in CTFE is made.

## Exact source binding and branch obligations

Full graph:32 instances/408 operations, completed source pin in the audit.
Both size_hint instances21/22 have the chain:

- bb0 evaluates u8 IS_ZST=false and selects bb2; bb1 is retained and excluded.
- bb2 loads field1 into `_5`, casts to NonNull `_4`; loads field0 into `_6`;
  casts `_4` into `_9`/`_11` and `_6` into `_10`. These typed loads and casts
  have no proved original-source interpretation in this lane.
- optional UbChecks selects bb5 or bb7. bb5 casts `_4`/`_6` to `_13`/`_14`
  and invokes actual precondition instance26. Its bb0 tests argument1>=argument2;
  its bb2 panic_nounwind_fmt branch is retained, conditionally excluded.
- bb7 tests0<sizeof(u8); bb8 tests sizeof(u8)<=isizeMAX. Both are proved true
  for size1. Their bb10 panic branch is retained, conditionally excluded.
- bb9 calls actual intrinsic27 with `_11` (END), `_10` (START), stores `_2`;
  bb4 goes to bb3; bb3 returns `_7=_2, _8=Some(_2)` as `(len,Some(len))`.
- instances15/16 project size_hint.0; specialized Zip::new instance11 obtains
  both sizes then calls usize min instance17 and stores index0/len.

`chain_step` is an explicit authored scalar/control fragment. `u8_branch_path`
proves the normal branch chain for BOTH settings of UbChecks. `failure_exclusion`
proves no failure transition in that fragment for a legal u8 span; it does not
assume a no-unwind flag. Therefore unavailable panic internals need not be
invented for that fragment. Original cleanup/drop/resume edges of min and Zip
are retained in the source inventory, with source simulation still REJECTED.
Termination of the authored fragment is not cleanup erasure in Rust.

The structured producer authenticates the completed input but checks typed
operands, fields, comparisons, projections, return tuple, constants and branches
independently of hashes. Mutations leave source identity/debug strings intact.
Eligibility is not a proof: the producer and structured extractor are untrusted.
They do not prove that MIR field reads/casts preserve the span or that an actual
Rust intrinsic step implements the authored scalar/control semantics.

## What is and is not established

The numeric result and branch exclusions are normally checked Coq theorems in
explicit finite scalar semantics. Operational Offset and subtraction reductions
are checked against the EXISTING lrust head_step relation. lrust has no native
ptr_offset_from_unsigned instruction; these operational lemmas are fragment
reductions, not a new native intrinsic semantics axiom. Scalar subtraction is
related to the proved finite result only under span bounds.

A complete native correspondence still requires: lawful incoming history and
source validity -> actual typed field/cast operands -> span; a source-machine
interpretation for pointer/integer conversion and intrinsic/branch execution;
then composition with constructor, calls, source phases, exceptions and original
reference observations. No such theorem is exported or presumed. The checked
conditional fragment is useful to discharge those arithmetic leaves once the
source relation is proved. Existing compiler implementation trust explains the
chosen arithmetic, but source-code inspection is not a Coq compiler-preservation
theorem. Native remains0, publication=false, goal incomplete.
