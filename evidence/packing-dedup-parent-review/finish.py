"""Summarize only audited results and seal compact evidence; no raw artifacts."""
from review import *
need(sys.flags.dont_write_bytecode,'python -B')
j=json.loads((HERE/'RESULTS.json').read_text());need(j['findings']==[],'review findings must be reported explicitly')
a=json.loads((HERE/'AUTHENTICATION.json').read_text());c=j['comparisons']
report=f'''# Independent review: typed packing deduplication and policy72/cache75

No actionable findings at prepared commit `{j['prepared_commit']}`. Independent
review passes for this bounded cleanup and compatibility scope. This is not the
root's separately run full36/broad-regression acceptance or whole-goal completion.
No production edits, delegation, push or PR were performed.

The restricted key is injective on its supported closed typed syntax modulo
only diagnostic spans. Every node type, operator payload, integer value, full
variable identity, ordered binder, ordered trigger and body is represented.
Unsupported/free shapes retain checks. Selection keeps the first original
empty-requires check/assume Arc pair in the same ordinary function. A distinct
false proposal remains; malformed syntax cannot be repaired by losing a later
identical copy because its original copy still undergoes ordinary validation.
See SOURCE_REVIEW.md for the field-by-field argument and publication review.

## Authentication and unchanged inventories

Independently authenticated preregistration8c2445a0 before implementationa7ca94cf,
identical preregistered baseline bytes, {a['source_bindings']} historical shipping
source bindings through own Git, {a['sealed_files']} sealed evidence files,
{a['receipts']} receipts, {a['raw_logs_and_freezes']} raw logs/freezes,
{a['fixtures']} fixtures and two complete eight-tool bundles. Completed source-only
ablation differs solely by `false &&` at the selection condition. No historical
certificate supplies a fresh positive import. Root completed auth-v3 SHA256 is
`{a['root_authentication_sha256']}`.

| Exact formula inventory | Historical/disabled | Fresh enabled | Removed |
|---|---:|---:|---:|
| Frozen full generate('full') | 146/0, 116 BV | 132/0, 102 BV | 14 |
| Unchanged natural packet | 32/0, 29 BV | 31/0, 28 BV | 1 |

All 27 ordinary full queries and all three fresh Eq precision proofs remain;
packet retains all three ordinary queries. Each original nonidentical query
survives in its original owner/order/multiplicity. Full typed inventory matches
all {len(c['full']['typed']['functions'])} functions (19 Exec/19 Spec) and
{len(c['full']['typed']['native'])} native classifications; packet typed/native
inventory matches too. Bodies, full domains, contracts, visibility and authored
semantics agree with authenticated worker and ablation. AIR inventory comparison
ignores comments/whitespace and qid/skolemid occurrence labels only; it is not the
production equality predicate. All retained/removed formula hashes are recorded.

Frozen full SHA256:
`{j['full_source_sha256']}`.
Fidelity preserves all eight original runtime bodies/debug assertions/private
genuine NonZero carriers. No application proof calls, snapshots, assertions,
new client premises or narrowed domains were added. Natural packet is checked
separately; its adapter changes only packet and CHANNEL_MASK visibility.

Full C/W uses identical invocation, source, tools and AIR, real Eq miss/hit,
and freshly proved required soundness/maximality/exactness over requires=true on
both runs. The actual foreign helper is a successful ordinary query with matching
helper/certificate identities. Raw API summary hashes remain recorded. Complete
C/W typed logs differ only at the exact known Eq ensure diagnostic-span token.

## Fresh independent validation

| Check | Result |
|---|---|
| Matching vstd | 2250/0 |
| VIR | 149 passed, zero failed/ignored |
| All compiler library units | 6 passed, including 3 packing + 2 API + original tuple |
| Focused compiler suites | 40 passed, zero failed/ignored |
| Ordinary full C/W observers | 5/0 each |
| False protector | 4/1, named observer_false::protector |
| Constructor protector/permission input equality limits | 0/1 each |
| Ordinary packet client, all three original fields | 1/0 |
| Bool provider/client/false/off | 2/0; 1/0; 0/1; 0/1 |
| Ordinary/transitive/private-helper import controls | pass |
| Auxiliary-only false BV, explicit and representative Cargo | 10/1 each, failed_functions=[]; selected stale exports absent |
| Erased-only type error, explicit and representative Cargo | 2/0 then E0308 each; selected stale exports absent |
| Old policy71 ordinary import | rejected by policy72; no consumer export |
| v74-tagged candidate entry at current address | cache miss, v75 regeneration, all three fresh full-domain Eq proofs |
| Independent authentication and final audit | pass with Python -B and -O -B |

Focused suites: local_checked_packing4, local_checked_foreign5,
certified_foreign_bool3, infer_goals12, infer_nonzero_derive3 and
constructor_function_values13. Ordinary consumers do not use infer-goals.
Constructor exposes word wf/lower tag, not equality to the input protector or
permission. Auxiliary inventories preserve the successful helper and ordinary
contract while exactly one isolated false query rejects globally. The additional
diagnostic Function-Recommends query is separately inventoried; successful-query
totals are observations, not acceptance thresholds. Global ordinary/inference/
auxiliary/deferred errors and final erased Rust compilation still gate publication.
Required actual same-compiler foreign certificates survive private pruning and
are enforced on ordinary/transitive imports; focused tests reject missing/stale/
incompatible evidence and disabled-check imports.

## Reproduction, identities, cost and trust

Fresh Rust1.99.1, rustup-first PATH, offline/locked Cargo, jobs2, authenticated
copied archives/index/git/solver inputs. Own compiler/macros/vstd are built from
identical prepared shipping source. Final compiler bytes were fixed before vstd
and before every provider/consumer proof. Only debug sections were removed from
the own copied unoptimized rust_verify; byte hashes before/after are recorded.
Every positive uses that same immutable bundle, including test harness copies.
The original tuple unit uses a matching own release-path vstd alias, unmodified.

Executing compiler SHA256: `{j['tools']['rust_verify']}`.
Packing module SHA256:
`d384bdd6091b89a83612529e6dc9bfba1573dd7c35018941f6befacb3b536f72`.
Own tested source-map SHA256: `{j['source_manifest_sha256']}`.
Worker canonical final source-map SHA256:
`d9aae6ee3a866da653fd19672df4975dc5262535dd71a8772c839f27416504f8`
(SHA256 of sorted-key json.dumps(FINAL_TOOLS.source), not the pretty snapshot).
All eight hashes, installed Rust identities, invocations, helper/certificate/AIR
hashes, fixture hashes and {len(j['receipts'])} fresh receipts appear in RESULTS.json.
Exact commands and access boundaries are in REPRO.md; all setup/excluded events
are in EXCLUSIONS.md. Raw sources, logs, AIR and binaries remain private.

Own build cost {j['receipts']['build']['seconds']:.2f}s; matching vstd
{j['receipts']['vstd']['seconds']:.2f}s. All individual observed costs are retained.
No wall-time improvement is demonstrated. Worker noisy concurrent debug full
runs were slower with dedup (cold57.82->74.26s, warm58.51->67.72s). Own stripped-debug
runs are not a matched benchmark against those unstripped runs. Query reduction
alone supports no speedup or release-performance claim.

Trust remains the compiler/backend/BV solver/Z3/model/vstd/imported-artifact
boundary. Certificates are proof-produced metadata, not independent proof objects
or native refinement. No foreign helper/library/model trust was added. No fresh
ablation build is claimed here: exact completed ablation authentication plus
structural source proof and fresh retained typed/formula inventory are the control.

Native correspondences0; all44 minimal+3 phase/101 writer effects remain OPEN.
Canonical6/0 and frozen full HMAC1/1 UNPROVED are inherited, not rerun. No necessary
actual shared step for BOTH original full targets is established. Whole-goal
publication=false; goal_complete=false. Root's matching full36,717 compiler,
Python148 each mode, canonical26/observer23 pipeline is independently owned and
is not claimed here.

One evidence-only local commit records this review; its SHA is given in the final
review handoff (self-reference is intentionally outside the sealed tree).
'''
(HERE/'REPORT.md').write_text(report)
files={p.name:sha(p) for p in sorted(HERE.iterdir()) if p.is_file() and p.name!='SEAL.json'}
write(HERE/'SEAL.json',dict(prepared_commit=j['prepared_commit'],files=files,findings=[],native_correspondences=0,publication=False,goal_complete=False))
print('sealed',len(files),'evidence files')
