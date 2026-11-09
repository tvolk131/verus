# Native Fedimint prefix inference, bounded cycle79

This cycle changes candidate construction, not the accepted application theorem.
The original `PrefixDatabase::get_full_key`,
`PrefixDatabaseTransaction::get_full_key`, and
`PrefixDatabaseTransaction::get_full_range` items are extracted from unchanged
Fedimint pin `b8bd3d8ef61e1410bd4ad01d2e3b6425f7973236`. Their complete item and
executable body bytes, generic domains, record field order/types/bounds, and
`GlobalDBTxAccessToken(u32)` are retained. Only Infer attributes are inserted
outside the native method items. The original record and method visibility is
not changed.

The provider methods have no authored preconditions or postconditions. The
frozen generated contracts in `contracts.json` have precondition `true` and
complete byte-sequence concatenation/endpoint equations. The explicit authored
namespace/cancellation lemmas remain ordinary Verus proofs from cycle78. Their
two record-taking goal functions are private in this detached proof crate so
that the original private record declarations need not be made public.

`wrappers.verus.rs` contains two explicitly auxiliary public Infer functions.
These construct an instance with concrete `Inner = u8` and call the original
transaction helpers. They permit VIR + rlib import/composition without changing
native visibility. They are not additional extracted Fedimint functions.
`consumer.verus.rs` includes one auxiliary inferred composition and explicit
authored goals. The original generic native methods themselves are checked for
their complete original input domains, not just for those concrete wrappers.

## Bounded implementation and trust

- Canonical `Vec<u8, Global>::extend_from_slice` updates only a mutable local
  receiver's complete sequence view. The borrowed slice argument is an inert
  immutable local read. Unsupported receivers/effects decline rather than being
  ignored. Successive calls preserve evaluation order and accumulated contents.
- Existing vstd vector clone/extend specifications and primitive clone semantics
  remain the authority. No new assumption, axiom, vstd behavior contract, trusted
  application summary, or desired-conclusion helper is added.
- The mathematical operation is the actual `vstd::seq::Seq::add` signature and
  identity, not arbitrary overloaded `Add`. Slice observation uses the actual
  resolved `View` implementation and its resolved generic arguments.
- The existing canonical `Range<u64>` profile is retained; only the concrete
  `Range<&[u8]>` and `Range<Vec<u8, Global>>` endpoint profiles are added. No
  global static/generic call gate or mutable-record policy is relaxed.
- Soundness/safety, safe-domain maximality, exactness, and publication checks
  remain mandatory. Contents do not establish whole-vector identity, capacity,
  allocation, or allocator behavior beyond existing standard-library trust.

## Replay and acceptance

Run `source/tools/check_fedimint_prefix_infer79.py` with explicit `--verus`,
`--solver`, `--application`, `--sysroot`, and a fresh `--output` directory.
An optional `--baseline-verus` records the former native append decline using
the same selected native items. Every invocation records its exact argument
vector, source, stdout/stderr hashes, and export state. Joined, duplicate, or
unrecorded options are not accepted. Successful provider and consumer reports
must match `scope.json` and the complete contracts exactly, with fresh solver
work for every mandatory obligation. Imports record actual VIR, rlib, and
certificate hashes and both distinct import kinds.

Owned-value candidates deliberately use `regenerated_state`: cold, warm, and
poisoned invocations regenerate candidates and reprove their obligations.
There is no candidate-cache hit claim and no hostile-cache decoder coverage
claim for this profile. Poisoned candidate-key files must not inject contracts.

Native omit-prefix, duplicate-suffix, and wrong-endpoint mutants are first
checked as exact transformations, then rejected by one explicit authored
semantic goal. Frontend declines, malformed goals, unsafe callers, and corrupt
import metadata are separate categories. Failed inference-aware exports are
seeded from actual successful interfaces and must be removed.

The synthetic two-slice reversed-order Rust integration test also infers an
exact transformation, then fails the authored original-order goal. A native
reversed-key probe using `&self.prefix` instead declines at the implicit
Vec-to-slice dereference: `only static, nongeneric inferred calls are supported`.
That probe is a frontend diagnostic, not semantic proof credit. Supporting that
conversion/effect shape is outside the inert-local-slice append profile; the
unchanged native helpers do not need it.

## Compatibility and limits

This production/model change reserves cache78 / policy72 / schema38:
`verus-infer-v78:byte-append:4096` and `candidate-bound-evidence-v72`.
Schema38 is unchanged. Shared controller/CI/identity-reader reconciliation is
the parent's responsibility; the shared identity reader still expects77/71.
Old dedicated evidence and runners are preserved, not rewritten to accept a
different engine or silently relabeled as fresh new-engine proofs.

This is a detached, source-connected selection, not verification of the entire
native module or application build. The high-level separation theorem requires
prefix incomparability explicitly. Actual BigSize encoding is variable length;
no fixed-length module ID or proof that actual module encodings are
prefix-incomparable is introduced. There is no transaction-wide confinement,
backend, async, fsync/crash-durability, consensus, external exactly-once, or funds
claim. Independent review remains the parent's subsequent responsibility.
