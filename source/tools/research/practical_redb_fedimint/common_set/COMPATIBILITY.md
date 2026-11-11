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
