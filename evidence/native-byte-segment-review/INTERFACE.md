# Exact MODEL byte/cell and register-segment review

Review base: `91eee431526e9006e2a58c46c8c73fd73a238f60`.
Implementations: register segment `703f9544`; byte XOR `240eb3dc`.
Owned writes: only `evidence/native-byte-segment-review/`. All generated proof
sources, objects, command logs and retained snapshots go to new private `/tmp`
directories. No implementation edits, parent/sibling checkout reads, live
parent-byte replay inspection, shared builds, substitutions or race experiments.

Typed review result: either accept the exact conditional MODEL statements with
explicit limitations, or return actionable findings with exact source lines and
reproduction evidence. Receipt success alone is insufficient: authenticate
worker receipt/source/object/freeze identities, all 30 register declarations
(including datatype constructors and generated eliminators), all 26 byte
constants, every assumption, helper and obligation inventory, readiness AND the
successful baseline receipt, 523 sources/284 objects, tool/loadpath/GMP pins and
completed dependencies. Fresh normal compilation and full recursive `coqchk -o`
without exclusions are mandatory for both modules. Freeze inputs before trials.

Inspect Boolean-word XOR vs literal Z.lxor, three-step scalar purity vs the
effectful whole assignment, actual read/call/same-cell write, explicit old-cell
ownership and one consumed store credit, arbitrary frame preservation, ordinary
data move19 bookkeeping, descriptor/register dependency and finite Offset
premises vs unbounded target arithmetic. Examine adequacy scope without adding
an adequacy or native simulation claim. A selector is untrusted source identity
only, never semantic correspondence or physical authority.

No new axioms, admissions, primitives, contracts, unsafe checking, original-body
or domain changes. No delegation, push or PR. Development/setup failures are
recorded separately from semantic findings. Only completed explicitly pinned
read-only `/tmp` evidence and this checkout/git history may be read; historical
checkout paths in receipts are labels and must be mapped to this checkout.

Acceptance cannot strengthen the inherited trust state: actual native
correspondences ZERO; 44 minimal + 3 phase / 101 writer effects open; full frozen
HMAC prefix AND suffix 1/1; canonical 6/0 (shared observer causally redundant
there); publication=false; goal incomplete. These are inherited markers, not
newly proved results. Return a committed final report, exact hashes and replay.
