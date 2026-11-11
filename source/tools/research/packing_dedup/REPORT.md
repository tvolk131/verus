# Checked packing dedup experiment

Measured optimization only; independent parent review is still required before
acceptance. Base efd9fb5680ece75f871058677afe10c9ce7e64eb / production21eeabbb.
Preregistration **8c2445a0** freezes baseline inventories and the typed interface
before engine trials; implementation **a7ca94cf** activates local exact selection.
No push, PR, subagents or global version edits. Schema39/policy71/cache74 remain;
policy72/cache75 are reserved for parent reconciliation after review.

| Fresh same-source ablation | Verified/errors | Isolated BV | Removed exact duplicates |
|---|---:|---:|---:|
| full | 146/0 → 132/0 | 116 → 102 | 14 |
| packet | 32/0 → 31/0 | 29 → 28 | 1 |

Full retains 27 ordinary checks and all 3 fresh Eq precision checks; packet retains
all 3 ordinary checks. Both cold/warm epochs give these totals. Natural packet
source is unchanged; a separate adapter changes only packet and CHANNEL_MASK
visibility and also proves 31/0.
The ordinary imported packet client proves all 3 original field properties, 1/0,
without infer-goals. Counts follow exact query inventories, not count thresholds.

| Full ordinary check owner | Original BV | Kept | Removed |
|---|---:|---:|---:|
| PERM_MASK | 14 | 13 | 1 |
| PROTECTED_MASK | 14 | 12 | 2 |
| TAG_MASK | 14 | 14 | 0 |
| Item::new | 23 | 19 | 4 |
| Item::perm | 23 | 19 | 4 |
| Item::protected | 14 | 12 | 2 |
| Item::tag | 14 | 13 | 1 |

Packet CHANNEL_MASK keeps 10/10; packet itself keeps 18/19. No proof is shared
across functions. The source diff leaves all discovery, arithmetic construction,
constant folding, resource bounds and original statements unchanged. It adds a
restricted typed closed key and a fresh per-function set, retaining the first
original Arc. Types at every node, exact operators/flags/widths, literal values,
VarIdent disambiguators, ordered binders/triggers and structure enter full Eq.
Only diagnostic spans are omitted. Unsupported/free/mistyped shapes fail closed
by retaining the query. There is no normalization, alpha equivalence or new trust.
Hash collisions still require structural Eq. Each retained original empty-requires
BV proof immediately precedes Assume of the identical Arc. Duplicate selection
removes that redundant pair together. Any retained false query still rejects.

RESULTS.json inventories every original/kept query by ordinary owner and exact
formula/raw AIR hashes, with first-occurrence mappings. Fresh disabled selection
reproduces the completed historical formula inventories. In the real ablation,
all original nonidentical queries survive in order. Inventory-only AIR comparison
ignores comments/whitespace and generated occurrence qid/skolemid labels; production
SST equality ignores only spans and retains ordered triggers as well. All 38 full
typed functions (19 Exec + 19 Spec), bodies/domains/contracts/visibility and 47 native
classifications match between fresh tool pairs. Complete typed-function hashes
also match after rebinding only the exact fixture filename and the generated
Eq candidate-ID attribute to its separately recorded freshly checked identity;
all raw hashes and both candidate IDs remain recorded. No type, body, operator,
binder, contract or other attribute is normalized. Packet typed/native inventories
also match. The complete generator SHA is
`1adb8b6c2ee64cc1c6930c9530920105ad26cee8abf3212caff53dda0c36518d`; fidelity pins all 8 original runtime bodies, debug
assertions and private genuine NonZero carriers. There are no application source
changes, proofs, snapshots or strengthened premises.

Full cold/warm uses identical frozen inputs and identical AIR. Eq candidate
`54531253ebdb19fe8b1ab422c349a422606a1efadb3aad3203b21e9d9c4c17d9` has a real miss/hit; soundness, maximality and
exactness are required and freshly proved on both runs over the full domain.
The actual foreign helper remains a successful normal query. Its query and foreign
certificate identities match cold/warm. Raw API hashes are recorded independently;
any conservative diagnostic relocation remains visible rather than bypassing an
identity check. Ordinary full observers successfully import both exports; imported
exec/closed-spec bodies stay hidden. Constructor contracts still expose only wf
and lower tag, with no input protector/permission equality.

Fresh compiler/macros and vstd were built offline/locked, Rust 1.99.1, jobs 2,
rustup-first PATH, with copied authenticated completed Cargo/solver inputs. Final
eight-tool byte hashes are frozen before applications. Executing rust_verify is
`b50c4f03ce261f739d526ac09315f2e80b93ae8e92fd168ee4b92e60afac8128`. Every positive provider and client uses this same
own bundle; no historical .vir is a positive input. The separately built disabled
bundle differs only at the selection condition in source, plus its own build paths
and resulting compiler-bound identities; its providers never feed the candidate.
Both matching vstd builds prove 2250/0. Source/tool/fixture guards and receipt hashes
are checked again at final audit; production checked_foreign and publication code
are unchanged.

| Fresh semantic/control validation | Outcome |
|---|---:|
| Matching candidate / ablation vstd | 2250/0 each |
| Requested focused compiler suites | 40/0 |
| VIR | 149/0 |
| Compiler library (3 new structural, 2 API, 1 existing tuple) | 6/0 |
| Symbolic bool C/W and provider | 2/0 |
| Ordinary bool client / false client / infer-goals off | 1/0;0/1;0/1 |
| Ordinary full observers cold and warm | 5/0 each |
| False protector observer, named observer_false::protector | 4/1 |
| Constructor protector-input equality limit | 0/1 |
| Ordinary packet client, all 3 fields | 1/0 |
| Auxiliary-only false BV, explicit / Cargo | 10/1;10/1 |
| Erased-only Rust type failure, explicit / Cargo | 2/0 then E0308 each |

The focused suites are local_checked_packing 4, local_checked_foreign 5,
certified_foreign_bool 3, infer_goals 12, infer_nonzero_derive 3 and
constructor_function_values 13. Structural units independently construct equal
formulas at distinct spans, distinguish semantic/type/binder/trigger changes,
retain free/unsupported/distinct false proposals, and inspect empty-requires,
original-Arc proof-before-use and fresh function scope. All 3 new units and both
API units passed in the initial run too. The existing tuple unit initially lacked
its hardcoded release-path vstd fixture; its failed receipt is retained and excluded.
A corrected own matching-bundle alias yields unmodified 6/0. See EXCLUSIONS.md.

Both auxiliary-only controls retain a healthy ordinary contract and genuinely
checked helper with failed_functions=[] yet reject the false isolated query and
remove the selected stale .vir. Their counts are derived from complete retained
query inventories, without a fixed successful-query threshold. The additional
Function-Recommends diagnostic query after failure is inventoried separately
from genuine verification totals. Both erased-only
failures remove stale exports after their 2 successful proofs and E0308. Additional
ordinary bool re-export/transitive/private-helper controls are retained in receipts.
No controller was weakened to force totals. Final deterministic audit passes under
normal Python -B and optimized -O -B; gates use exceptions, not Python assert.

| Wall cost (debug, concurrent) | Cold | Warm |
|---|---:|---:|
| full, disabled | 57.82s | 58.51s |
| full, enabled | 74.26s | 67.72s |
| packet, disabled | 53.97s | 53.54s |
| packet, enabled | 53.71s | 55.20s |

Candidate build 246.97s, candidate matching vstd
160.21s; ablation build
279.65s, ablation vstd
140.18s. Runner time includes child runtime
and post-run source/tool hash guards, excluding pre-run freeze work. Compiler
identity hashing includes an unstripped 796,612,520-byte candidate executing binary.
Selection/key/hash overhead is included; it is not separately isolated. Concurrent
own builds/tests/trials and debug hashing dominate these noisy samples. Full is
slower in this run; packet differences are small. This demonstrates redundant query
removal, **not wall-time speedup** or release performance. Historical baseline
wall costs are retained in BASELINE.json and are not used as a performance ablation.

Raw generated sources/AIR/logs/binaries remain private at PRIVATE_ROOT and
ABLATION_ROOT. Compact receipts, hashes, formula/typed/native inventories and
controllers are committed here. See REPRO.md and ASSUMPTIONS.md for replay and
unchanged compiler/core/model/BV/Z3/imported-artifact/vstd trust. Certificates are
proof-produced metadata, not independent proof objects or native refinement.

Native correspondences 0;44 minimal + 3 phase / 101 writer obligations remain OPEN.
Canonical 6/0 and full frozen HMAC1/1 UNPROVED are inherited, not rerun. Necessary
shared assistance on both original full targets remains unconfirmed. Whole-goal
publication=false;goal_complete=false. Independent parent review required.
