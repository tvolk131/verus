# Checked array-to-slice carrier MODEL

All33 declarations passed normal compilation, individual closed-assumption
inspection and complete recursive kernel checking. This is the representation
suboperation before reference validation, retagging and local write hooks.
**The complete Rust unsizing operation and native correspondence remain unproved.**

The accepted root replay is trial004, controller commit `dfe1a04a` and freeze
`trial004-freeze.json`. Exact commands, input/product hashes and all33 assumption
entries are in [results.json](results.json). Independent review is pending.

## Exact mathematical result

`Carrier.array_to_slice` builds `SliceValue r n` from `RefValue r` when `n`
fits64 bits. `Carrier.carrier_store` additionally requires distinct source and
destination locals, an initialized immediate reference carrier in the source,
and an uninitialized immediate destination. It changes only that destination's
data representation. The source local, caller/private physical cells, original
reference, incoming history, private allocation state, storage trace and control
location are preserved.

These are premises of a partial representation MODEL, not new application
preconditions. Carrier data does not establish live allocation, dereferenceability,
alias permissions, reference validity, initialized bytes or C/F equations. The
generic laws include lengths10/128; they do not prove common native assistance
for the canonical and writer goals.

The generated `Root` module binds the frozen root `bb0[4]` assignment from
`_4: &mut [u8;128]` to `_3: &mut [u8]`, including the exact `Unsize` cast and
`move` operand. It proves carrier progress for that local configuration. Control
stays at the same site because the statement has not completed validation.
A separate composition with the existing storage model shows that discarding
the source local does not discard the destination carrier. This is a composition
of MODEL operations, not a reachability or source-loan-resolution theorem.

The algebraic `changed_tag` counterexample preserves every represented physical
address while changing the provenance tag. Address equality cannot justify an
unchanged-tag claim after validation.

## Pinned source boundary

The source graph and original six-instance/44-operation extraction are both
authenticated; the original extraction fixes x86_64 and64-bit pointer width.
Selection is checked against structured MIR, not debug text. The compiler pin
is `58cf8f9c35b355199727627d07ae34a4e2846d67`.

The pinned interpreter's [array/slice cast implementation](https://raw.githubusercontent.com/rust-lang/rust/58cf8f9c35b355199727627d07ae34a4e2846d67/compiler/rustc_const_eval/src/interpret/cast.rs)
constructs a pointer/length pair, writes it and validates the result. The
[validation implementation](https://raw.githubusercontent.com/rust-lang/rust/58cf8f9c35b355199727627d07ae34a4e2846d67/compiler/rustc_const_eval/src/interpret/validity.rs)
can check dereferenceability and invoke machine retagging. Those machine effects
are not implemented by this carrier operation. Both the write's validation and
the outer cast's validation remain part of the source obligation.

The [operand evaluator](https://raw.githubusercontent.com/rust-lang/rust/58cf8f9c35b355199727627d07ae34a4e2846d67/compiler/rustc_const_eval/src/interpret/operand.rs)
currently reads MIR `Move` and `Copy` identically and records unfinished move
invalidation. Preserving the source carrier here follows that representation
suboperation; it does not establish Rust move semantics or duplicate permission.
Source inspection and exact hashing supply no formal Rust simulation theorem.

## Validation and retained trials

- Trial001 passed33 closed declarations and recursive kernel checking with the
  original controller. Trial003 added direct original-source/target-width checks
  and passed. Trial004 pins the completed parent receipt independently of later
  review prose and passed in normal Python. Mathematical sources/objects match
  byte-for-byte across all three successful trials.
- Trial002 stopped before compilation: the controller applied `len` to the
  extraction's scalar operation count. Its failure receipt has success=false
  and model proof status=false. This is a setup error, not semantic evidence.
- Nine pure typed-source selector tests passed under normal and optimized Python:
  healthy generation, wrong destination, copy/move change, array-length change,
  immutable target, projected source, unknown field, duplicate and missing site.
  Their sources were frozen before those trials. No filesystem mutation or race
  experiments were used.
- The complete inventory contains28 declarations in Carrier and5 in Root.
  The generated inspection module adds no declarations or marker helpers. All
  individual assumptions and recursive kernel axioms are empty. There are no
  new axioms, admissions, Program obligations or unsafe checking modes.
- Fresh Interface, Graph and Storage objects match the authenticated parent
  objects. The checker freezes its code/support/candidates, guards current goal
  requirements and all retained products, and pins the prover tools, GMP archive
  manifest and all1,121 Coq standard-library/plugin inputs. This pure component
  imports no Iris/VerusBelt logical module.

Replay uses the [README command](README.md). Raw receipts are retained under
`/tmp/native-source-unsize-20261110/trial00{1,2,3,4}/receipt.json`; none is erased.

## Remaining obligations

Actual incoming-reference interpretation, validation/retag/write-hook effects,
move semantics, source control/phase simulation and original/pending C/F bindings
remain open. There is no actual Rust source instance or certificate here.
Canonical6/0 and full HMAC1/1 are inherited, not rerun. Native correspondences0,
all44+3/101 source/effect obligations open, publication=false, goal incomplete.
