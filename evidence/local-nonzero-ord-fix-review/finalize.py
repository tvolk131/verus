"""Generate compact report only after both read-only acceptance modes agree."""
from build import *
r=root();j=json.loads((HERE/'RESULTS.json').read_text())
a=json.loads((r/'acceptance-normal.json').read_text());b=json.loads((r/'acceptance-optimized.json').read_text())
need(a==b and a['accept'] and a['results_sha256']==sha(HERE/'RESULTS.json'),'normal/optimized acceptance agreement')
need(j['status']=='ACCEPT_F1_ACCEPT_F2','complete acceptance required')
h=j['helpers']['bundle-exact-cold'];d=j['dual']
text=f'''# Independent NonZero Ord fixes review

**F1: ACCEPT. F2: ACCEPT.** No remaining blocking finding in the reviewed fixes at
prepared `913044c43a476188537a087618a205e01aafc9eb`, production fix
`297332c265e85bcc0fbcaac253277f307906c3f2`. This accepts the two bounded fixes;
it does not promote provisional policy73 or complete the broader goal.

F1 now removes every distinct explicit/Cargo selection, attempts all paths even
after an error, and reports the first error. Writing precedence is unchanged.
Fresh same-runtime Bool Cargo providers each passed2/0. The next direct Ord
invocation selected explicit+Cargo and reached the intended local-only refusal;
both destinations disappeared. Distinct-path prior artifact SHA-256:
`{d['dual-distinct']['prior_artifact_sha256']}`;
equal-path prior artifact SHA-256:
`{d['dual-same']['prior_artifact_sha256']}`.
These were genuine fresh schema39/policy73 interfaces, never historical positive
VIR. The lib8 filesystem unit passes A/B, B/A, A/A and a first-path directory
error that still clears the second file. All24 existing single-route/disabled/
partial/global/late-erasure invalidation controls also pass. Refusals are frontend
or export-gate failures, not semantic source theorem failures.

F2 now compares all local checked types with complete recursive structural
serialization identity. The whole typed checker contract and ordinary pipeline
review is in SOURCE_REVIEW.md. Independently authored private compiler probes
clone the actual loaded Function at registration and after ordinary WF/autospec/
modes. Each boundary rejects12 embedded-path-only variants: clear/change/duplicate
in each of the two shared parameters, nested body types, and nested contract
types. Retained serialized originals/clones are unequal, but become structurally
equal after masking only datatype witness fields; nominal types, args,
decorations and all other serialized Function data remain intact. The original source
still verifies2/0 in this diagnostic compiler, and no mutated Function reaches
proof. Both boundary original serializations identical:
`{j['structural']['originals_identical_across_boundaries']}`.
This resolves correspondence coverage. No source semantic false-success is
claimed from synthetic type edits. No new law, carrier narrowing or assumption.

Fresh validation used own offline locked Rust1.99.1 jobs2 build, own macros and
full normal matching vstd2250/0; eight tools frozen before application proofs.
Every positive application proof used --no-cheating and full lifetime, trait,
body and erased Rust checks. Test results: focused40/0 (3+5+4+12+3+13), VIR149/0,
compiler library8/0, including real loaded MIR/Function synthetic units. Explicit
RUSTUP_HOME=/home/dev/.rustup was present for those units. Parent broad717,
compiler/Python/canonical/observer suites are not claimed or inspected here.

- Direct actual a.cmp(b), full-domain numeric relation: cold2/0, warm2/0, identical
  invocation, full Function and normal helper-query bytes. Three numeric
  relations2/0; aliased inputs2/0; mixed caller/type-alias contexts6/0.
- False/swapped callers1/1 each, ONLY local_order::total failed, helper healthy.
  Goals off0/1; existing PartialOrd5/0; weak property2/0 is only that weak property.
- Effectful arguments2/0; false effect assertion1/1. Ordinary get/get/u64cmp
  mirror1/0 has identical three typed model targets and no synthetic helper.
  Local Ord auxiliary failure10/1 has no failed ordinary function and helper
  healthy. These are separate semantic/effect/global controls.
- Fresh certified Bool provider2/0, ordinary goals-off import1/0, false import0/1.
  Exact unchanged proofless Item132/0 and packet31/0 export locally. Item fixture
  SHA-256 `1adb8b6c2ee64cc1c6930c9530920105ad26cee8abf3212caff53dda0c36518d`
  was separately user-authorized and authenticated against the completed freeze.
  Ignored-Ord Eq control6/0; original wrapper clients remain2/6, without the helper.

Exact source/core/tool, command, receipt, actual Function and query identities are
in RESULTS.json, SOURCE_AUTHENTICATION.json and TOOL_FREEZE.json. Cold/warm actual
Function log SHA-256:
`{h['actual_Function_sha256']}`;
normal helper query SHA-256:
`{h['normal_query_sha256']}`.
These bind diagnostic bytes, not independent proof objects. The actual helper is
still a normal fresh get/get/u64cmp body proof through BodyBackedForeign and the
existing normal-query health gate; it is not an unchecked std summary. Existing
opaque NonZero get/View and primitive u64 Ord models remain inherited trust.
The bounded translator/checker is compiler TCB, not a kernel or native theorem.

Schema39/policy73/cache76 remain provisional; last accepted parent is policy72.
Genuine wrapper BorTag Ord and ordering imports remain UNPROVED. Native=0,
HMAC1/1 UNPROVED, 44+3/101 OPEN remain inherited statuses, not rerun results.
publication=false; goal_complete=false. No ordering certificate/import exists.

Normal and optimized Python read-only acceptance audits agree via explicit
exceptions/need checks, with final source/fixture/tool guards. Preregistration
commit: d1b92e27. Raw binaries/source/MIR/AIR/logs/Function records remain in
`{r}`. Only this new evidence directory is committed. No production/historical
evidence edits, subagents, shell worktrees, network, push, PR or publication.
An initial accidental broad find was interrupted before any output; no sibling
content was obtained or used. The initial sandbox initialization failure is a
setup exclusion. Subsequent scope was checkout-local plus specifically authorized
completed inputs; no live parent pipelines/artifacts or process table were read.
'''
write(HERE/'VALIDATION.json',dict(status=j['status'],results_sha256=sha(HERE/'RESULTS.json'),matching_vstd=2250,focused=40,vir=149,compiler_library=8,normal_python_audit_passed=True,optimized_python_audit_passed=True,acceptance_stdout_sha256=sha(r/'acceptance-normal.json'),optimized_acceptance_stdout_sha256=sha(r/'acceptance-optimized.json'),source_authentication_sha256=sha(HERE/'SOURCE_AUTHENTICATION.json'),publication=False,goal_complete=False))
(HERE/'REPORT.md').write_text(text)
write(HERE/'SEAL.json',dict(files={p.name:sha(p) for p in sorted(HERE.iterdir()) if p.is_file() and p.name!='SEAL.json'},publication=False,goal_complete=False))
print('compact report and seal complete')
