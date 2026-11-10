# Independent constructor function-value review

**REQUEST CHANGES at frontend/test scope on 868b6a19.** One actionable integration
finding; no soundness defect or eager-registration regression found in the bounded
review. Production, engine, vstd, policy, caches/schema and focused test sources
were not edited.

- **P2 — update stale zero-arity constructor rejection tests.**
  `source/rust_verify_test/tests/fndef_types.rs:1416` and `:1427` still expect
  “using a datatype constructor as a function value” for `let a = A` and
  `let a = Self`, with `struct A();`. Both existing tests fail (exit 101,
  `Err(_) does not match $result`). Their unchanged emitted sources independently
  verify **2/0** each with the fresh verifier, normal checks and `--no-cheating`.
  These are test expectation failures, not verifier panics or semantic negatives.
  Change both expectations to success, rename appropriately, and preferably
  exercise invocation as well. Parent should make this small test fix.

Fresh successful checks:

- Own offline/locked compiler and macros, Rust **1.99.1**, compiler commit
  `58cf8f9c35b355199727627d07ae34a4e2846d67`, two Cargo jobs; normal matching
  vstd **2250/0**, without forwarding `--no-cheating` to inherited axioms.
- Literal original `NonZero::new(i).map(BorTag)`, every u64, no requires:
  **2/0 off and on**. Exact local function-detail/body inventory is
  `bortag::BorTag::new`, `bortag::BorTag::ctor%`; generated inference functions
  and all assistance lists are empty. No inference causal gain.
- All **13 focused tests pass**, and all 13 independently replayed delivered
  direct cases match: five positives, two proof rejections, three conservative
  frontend rejections, three Rust visibility/move/lifetime rejections. Actual
  `NonZero::get` roundtrip is **4/0**. Wrong variant/order/postcondition fixture
  is **3 verified / 4 errors**, with healthy constructor bodies. The generic
  missing-requires fixture is an ordinary callback precondition rejection,
  not itself a constructor-body negative.
- Eight selected existing tests pass: FnDef `basic_test`; closures `test1`;
  mut_refs `test_basic`, `test_paren_ctors_with_mut_refs`,
  `test_structs_and_boxes`, `two_phase_ctor`; mut_refs_closures
  `fn_mut_call_ensures_preserves_call_reborrow`,
  `closure_returning_mut_ref_arg_basic`.
- Fourteen additional positives cover unused constructors with explicit mutable
  and shared references, nested lifetime bounds, associated-type and HRTB bounds,
  unsized shapes, skipped spec fields, Tracked wrapper fields, ordinary direct
  construction, local import aliases, FnDef fields, proof-mode use and a mutable
  callback. A false mutation assertion is **1/1**, at its intended assertion;
  the constructor body is healthy. This is bounded evidence, not universal eager
  compatibility. Eager registration still adds checks for unused constructors
  (24 additional vstd queries relative to authenticated inherited 2226).
- Normal compile/export/import replay of existing ordinary-library and inferred
  base/middle/app fixture bodies passes, including **6/0, 3/0, 4/0** for the
  inferred chain. An exported constructor provider and imported direct call
  pass. Imported constructor-as-value is clearly rejected, as scoped. A failed
  application assertion prevents both `.vir` and `.rlib` publication. No
  metadata replacement, stale-artifact substitution, security or race trials.

**Nonblocking limitation:** the premise-free opaque-return probe
`fn mk() -> (f: impl Fn(u64)->R) ensures f.requires((7u64,)),
forall|x:u64,r:R| f.ensures((x,),r) ==> r.0==x { R }`, with `struct R(u64)`,
fails to establish `mk`'s postconditions (**2/1**). Returning an ordinary checked
`wrap` with the same output contract instead passes **4/0**. This is an unproved
valid-looking contract, not a false-semantic-query fixture or a panic. Opaque
return callbacks were explicitly outside delivery coverage; do not extend the
support claim to this shape. The two source hashes and exact commands are bound
in `evidence.json`; complete sources stay private.

Concrete translation assessment: registration uses the actual Rust constructor
DefId/path and checks path expression FnDef identity. Ordered VIR fields are
zipped with the Rust fields for copy/move classification; parameter/result types
and ADT bounds are preserved. Fresh internal spans/binders feed an ordinary Ctor
body and a checked exact-result postcondition. Constructor stubs/full bodies enter
normal WF/mode/resolution/proof handling, and existing FnDef contract lifting
supplies the Fn traits. There is no application closure, new callback axiom,
function-type equivalence, or trusted generated body. Visibility/move/lifetime
negatives and local alias positives exercise these boundaries; spec/mixed fields,
opaque/proxy datatypes and user type invariants are conservatively excluded from
the new value path. Imported direct construction continues to work. Source hashes
and generated equalities do not establish arbitrary Rust semantics.

Reproduction and identities: `epoch.py`, `run.py`, `validate.py` and the hashes of
private `batch.py` / `finish.py` are recorded. Commands run with **python3 -B -O**;
all validation guards are explicit and remain active under optimization. Build:
`run.py build`, then `run.py vstd`; private batch stages `original`, `probes`,
`delivered`, `interfaces`, `test-build`, `tests`; corrected focused suite via
`finish.py`; two direct `run.py proof stale-zero-arity[-self] ...` epochs.
Every verifier argv, relevant environment, exit, log/product hash, controller/test
binary/source identity and selected local function inventory is recorded in
`evidence.json`; full before manifests, receipts, sources, logs and binaries stay
under `/tmp/constructor-function-value-review-t3-755cb0d1/`. Accepted inputs and
products were rehashed at final validation. System Python/Git/process execution
and the pinned Rust compiler/solver remain trusted infrastructure.

Authentication: canonical receipt's prescribed SHA and success and all eight
bundle hashes; five production, one test, three delivered controller hashes;
32 worker receipts and 64 logs; private production.patch; nine unchanged inference
engine files; ten native pins; literal native new-body SHA
`78b385b088022a0e3b0abc5968018a26f0590d2b221a99a8f2e40ccfb1aaeead`.
No old driver was executed; no historical worker checkout/build output was read.
Only own checkout/history, named completed /tmp inputs and private outputs were
used. Locked archives were individually checked against own Cargo.lock and copied
to a private Cargo home; no worker Cargo config, whole oversized cache, install,
network, shared config/toolchain change, delegation, push or PR.

Excluded attempts remain recorded: bwrap startup RTM_NEWADDR failure (exact tool
escalation used); duplicate same-checksum archive selector rejection; first build
missing private Git context and invalidated evidence-file inventory guard when
the early checkpoint was committed; second build missing copied tools/common;
pre-launch test-build freeze before libvstd existed; first focused harness run
with duplicate --no-cheating. None supplies a semantic result. The accepted build,
vstd, applications and tests have final guards true. No frontend error, Rust
error, setup failure or harness panic is counted as semantic rejection. There
were no verifier panics or resource failures in the accepted application trials.

Combined objective remains pending: canonical all-u64 / arbitrary initialized
exclusive [u8;10] **6/0 is inherited, not rerun here**. Independently frozen
bitcoin_hashes 0.14 HMAC, arbitrary initialized [u8;128], all lawful key lengths
including 0 and >128, both XOR prefix and unchanged suffix, remains **1/1
UNPROVED**. Same actual checked automation/fresh combined acceptance and the
parent's full 32-target suite remain parent work. Native=0; 44 minimal +3 phase /
101 writer effects OPEN. This frontend review establishes no native/reference
C/F/safety/application theorem. **publication=false; goal_complete=false.**
