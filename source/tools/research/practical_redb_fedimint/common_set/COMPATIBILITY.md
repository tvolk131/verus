# Parent S1 compatibility probes

This checks the delivered S1 source at f22e069b/root7384108e with its pinned immutable debug runtime. It is additional bounded evidence, not independent implementation acceptance, an application gain or a new feature layer.

Two previously valid proof programs pass with goals disabled AND enabled, unchanged one-proof-body scopes and zero inferred helpers:

1. A checked spec observation at(xs,i):Set<int> recommends valid Seq indexing. A caller requires !p and uses the observation only in the guarded postcondition p ==> at(xs,0)==b. Both modes pass1/0, with no new recommendation diagnostic. S1 emits2 ordinary calls over2 terms.
2. A proof over an empty Seq<Set<int>> uses an index only under a bounded quantifier, plus two free Set parameters. Both modes pass1/0. S1's2 calls use the free parameter observations; the bound name cannot escape the quantifier, and full mode/WF checking passes.

Spec recommendations are soft diagnostic conditions, not hard logical undefinedness; see the existing [spec signature reference](../../../../docs/guide/src/reference-spec-signature.md). S1 still proposes only true checked Set facts and adds no runtime access, assumption or library precondition bypass. These two programs do NOT establish universal selector compatibility. Other names/types/visibility/budget cases remain for independent review.

Exact source files and existing-utility replay: /tmp/practical-s1-parent-compat-v2/{guarded.rs,bound.rs,check.py,receipt.json}. Receipt SHA256: 8cda4737134052e6bba42535cb4c430f2e0382af7229f8ae87dcf94c5aece3ed. Four final expected outcomes complete under --no-cheating; full local scope, body identities, empty inferred helpers, runtime hashes and input stability are checked. Use a fresh output/cache for replay. Debug-runtime warning is inherited in both modes; no speedup/cache comparison is claimed.

Two initial frontend-only setup starts are retained under /tmp/practical-s1-parent-compat/ and -v1: the first used an unrecognized attribute spelling, the second an invalid private-open combination. Both failed before testing S1. The supported spec(checked) private declaration is used in final sources; these setup failures are not semantic evidence or selector findings.

Parent token comparison also confirms the frozen r2-reduced.rs equals the original full manual fixture with ONLY the necessary extensional assertion and redundant range broadcast deleted. Frozen f2-reduced.rs equals the original immutable component plus disclosed checked FromSpec adapter, with ONLY the three unsolved-slot comments removed. All runtime/contract/spec/helper tokens otherwise match; no authored replacement Set calls are hidden in either reduced fixture. This supports the precise claimed author-work deletion, separately from the new compiler checks.

The NEW independent S1 review and S2/combined acceptance are still pending. Unsupported F0/R1/F1 stay counted; HMAC/native remains deferred and unachieved.

## Parent replay scope tightening after independent acceptance

The accepted independent review checked full Rust function identities in addition to counts. The existing `replay_s1.py` now carries that gate into every ordinary positive and negative scenario: exact source and selected body inventory, distinct `BtreeBitmap::set`/`U64GroupedBitmap::set` identities, exec/proof modes for successful runs, zero falsely certified bodies for negatives, exact failed-function/SMT identities and exact Set-pair function scopes. Inferred-helper precision checks remain unchanged. The policy option defaults to77 and also permits the preregistered combined78; it does not accept arbitrary trailer versions.

Validation rechecks the authenticated independent replay's23 ordinary outcomes (the2 intentional version/frontend refusals retain their separate gate) under normal and optimized Python. This is data validation, not new compiler evidence. Five adversarial unit tests per mode exercise omitted/extra/duplicated/misclassified bodies, source/selection/count changes, duplicate method basenames, wrong dependency failures and invalid pair scopes. The existing replay remains the entry point; no new execution/evidence framework is added. Private validation: `/tmp/practical-s1-replay-scope-validation-33cf6e88/{normal,optimized}.json`. Future matching combined compiler runs must pass these stronger checks with their actual fixture inventories.

Setup note: a repository-instruction discovery command briefly enumerated AGENTS filenames too broadly; it read no sibling file contents. The correction read only the exact ancestor paths before editing. No enumerated sibling was a build or proof input.
