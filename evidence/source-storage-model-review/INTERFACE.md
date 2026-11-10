# Independent storage MODEL review — scope milestone

Base checkout: `bc2a0b37`. Review targets: delivered `e77da206`/`ade18e67`
and replay policy `4aefbe9d`/`58a779e2`. Owned changes are confined to this
new evidence directory; no implementation edits, subdelegation, push or PR.

Typed boundary under review:

- `Storage.state H` holds independent local/control/caller/private storage,
  the original `reference`, arbitrary incoming `H`, and a storage-effect trace.
- `Prefix.execute : state H -> event -> result H` exposes Advanced,
  Unsupported, StorageFault and ControlFault. `operational_step` maps those
  results into the existing constructor-erasure outcome carrier; its faults
  do not interpret actual Rust undefined behavior.
- `root_entry` fixes locals0..4 and root bb0[0]. The three covered root
  StorageLive statements advance to bb0[3], with original local1/reference,
  caller cells and incoming H retained. All enabled outcomes must be checked.
- StorageDead progress at later sites is pointwise, not reachable execution.
  Private allocation/release, repeated live/dead and explicit uninitialized
  materialization do not imply referent Drop or general spill interpretation.

Acceptance will require authenticated source/controller/support/tool/dependency
pins before a fresh normal coqc replay, recursive coqchk with no exclusions,
all142 declarations (including generated names) individually closed, and
mathematical V/VO identity with both delivered and parent receipts. Review gates
only from source and pure in-memory parser tests; no filesystem substitution,
race or security experiments. All raw replay outputs are private under /tmp.

Acceptance is MODEL-only. It cannot supply actual Rust source instances,
source safety, no_unwind, permissions, history/hook correspondence or C/F
bindings. Native correspondences remain0; 44 operations+3 phases and101 writer
effects remain open. Canonical6/0 and full HMAC1/1 are inherited, not rerun.
The every-u64/arbitrary initialized10-byte canonical domain and exact frozen
HMAC prefix+unchanged-suffix/all-key-length128-byte domain, normal checking,
source fidelity, same automation and combined independent acceptance remain
required. Publication=false and full-goal completion=false.
