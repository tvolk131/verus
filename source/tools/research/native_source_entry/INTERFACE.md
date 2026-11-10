# Actual source entry: frozen boundary, inactive

This lane owns only `native_source_entry/`. Base: `e84eed6c79d9c104cefada569de0c99221dc2b51`.
No production hook, native fact, supplied permission, assumed reference predicate,
algorithm rewrite, dependency installation, or shared baseline change is proposed.

## Selected source and candidate

The selected body remains byte-for-byte `../native_heap_lowering/fixtures/drop.rs`:
`pub fn xor_key(buf: &mut [u8; 128]) { let _it = buf.iter_mut(); }`.
The authenticated extraction remains `/tmp/native-semantic-bridge-e4a8dcab/source-v1`.
Its entry is `_1: &mut [u8;128]`; source `(*_1)` is reborrowed into `_4`, moved and
unsized into `_3`, then consumed by the actual native constructor into `_2`.
The candidate is an **initialization-route investigation**, not a replacement
program or an alleged proof. Before any trials, inspect precisely what existing
heap, lifetime, prophecy, and uniqueness initialization can produce and what the
source semantic typing context must supply. Do not compile another conditional
accessor as a substitute for the input theorem.

## Required sequent (notation for an obligation, not definitions)

For every valid original source entry configuration and its actual Rust typing
context, derive an initial global logical interpretation and semantic context
for that SAME configuration. The semantic context must provide the existing
VerusBelt `ty_own (&uniq{κ} byte_array128_ty) m d g tid [FVal #p]`, with:

- `p` representing the actual runtime input pointer, not `MutRef::unpack.ptr`;
- the initialized 128-byte subobject and its arbitrary current bytes;
- the original dynamic provenance and loan, including suspended ancestor loans;
- the existing C/F observations, not a newly chosen final value;
- the source lifetime and typing-context interpretation used by the checked v12
  context accessor, with the original resource transferred exactly once.

The global state interpretation must be derived from source execution/typing,
not made synonymous with this desired ownership judgment. Physical premises
must follow from valid source typing and the actual source semantics. No separate
PointsTo, freeable capability, owned heap, second live borrow, allocation-base
identity, zero initialization, or new user requires may supply the result.
All valid interior subobjects of larger allocations remain in scope.

## Later boundaries retained

The same original loan must survive array reborrow/unsize and each native source
primitive, then return through normal call/return/drop. Pending-reference
resolution must derive `F(B0)=C(B0)` for the original input through the existing
C/F update equations; empty runtime drop does not itself resolve prophecy.
Borrowck FakeRead, cleanup/unwind and monomorphized-phase differences need
simulation or justified exclusion. Model unbounded Offset needs a separately
checked finite/in-allocation/isize/one-past native simulation.

The full original HMAC XOR prefix AND untouched suffix remains required for
arbitrary initialized [128] and every valid key (including 0 and >128, dangling
empty keys, and disjoint pad/key subobjects in one allocation). Canonical full
varint remains required for every u64/arbitrary initialized [10], unchanged
body/domain/predicate, authored semantic invariants only. No shared necessary
proof step is inferred from the observer ablation.

## Evidence and stopping condition

Freeze source identities, premise inventories and all candidate commands before
trials. Distinguish identity/eligibility controls, setup failures, semantic proof
failures and actual native theorems. If the needed source typing/state embedding
or primitive simulation is absent, record the exact first missing layer and its
existing-rule antecedents. Do not instantiate soundness on an unrelated chosen
model program. Any eventual core proposal stays inactive and separately reviewable.

At freeze: native input embedding unproved; 44 minimal simulations and 101 full
writer effects open; inherited native-drop 0/1 and full HMAC 1/1 (not rerun);
publication=false. Parent integration and independent review remain required.
