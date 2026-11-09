# Actual original-crate offline preflight115

## Evidence that changes the next action

All132 recovered redb files authenticate against the recorded SHA256, length
and Git blob content hashes for pin c82ab865bb362bc4af067d332adc44978ff64813.
The original Cargo.toml, build.rs, source modules and workspace configuration
remain byte-identical in both the recovered input and fresh staged copy.

The unchanged original workspace parses with offline locked no-deps metadata,
but full offline resolution fails because the unrelated redb-python workspace
member requires unavailable pyo3. That nonzero result and its diagnostic remain
retained. It is not a theorem failure, a reason to enable network, or proof that
the actual default redb library cannot be analyzed offline.

A separate root diagnostic envelope depending on the unchanged redb package by
path resolves successfully offline. It does not rewrite redb's manifest or
source. Cargo generates a new envelope lockfile, and subsequent offline locked
metadata succeeds. The actual redb package selects exactly default and std;
the only external resolved package is installed libc0.2.191. The other two
packages are redb4.4.0 and the root diagnostic envelope. Optional logging,
chrono/uuid and preview/no_std modes are not claimed covered.

This supplies a concrete next route to compiler diagnostics over the complete
original default-std library, rather than another namespace-changing projection
of nine methods. Root Cargo metadata/resolution is different from canonical
whole-workspace resolution, which remains blocked on pyo3. No dependency is
vendored, overridden, downloaded, copied or silently supplied.

## Next executable target

After the sole compiler-heavy worker hands off, a fresh source-bound diagnostic
driver can analyze this exact redb library using the successful path-dependency
envelope and recorded lockfile. It must capture original
TransactionalMemory::write_header's Builder and elaborated-MIR observations
before unsupported resource types can be mistaken for verified VIR admission.
Same-source function identity and actual anonymous WritablePage temporary must
be evidenced, not replaced by a caller-supplied byte buffer or owned-u64 fixture.

The diagnostic must be explicitly separate from proof admission, retain raw
typed local/phase/drop observations, and produce no Verus interface/export or
native-proof success claim. If a dedicated callback is needed, it must be
default-off, provenance-bearing and fail-closed as a proof source. Merely using
normal Rust to observe MIR does not validate Arc/Mutex/cache/glue semantics or
create a native resource producer. Existing S0 and publication gates remain.

## Limits and honesty

These were only noncompiling Cargo metadata/lock-resolution commands. No redb
application, source build script, verifier, solver or compiler build was run.
Successful metadata does not guarantee compilation, linker/SDK closure,
resource correspondence or final-cell success. New lockfile and packaging
configuration are explicit; original source-path preservation FAILED, Git
continuity UNAVAILABLE and compiler/source correspondence UNESTABLISHED remain.
All18 remain OPEN, with no new trust, source integration or native proof credit.

EXECUTION-NOTES.md discloses the exploratory unavailable tomllib parser and
pre-authentication discovery; raw managed commands have exact launch environments,
true completed statuses and both untruncated streams. Commands used pinned
tool executables directly, not a rustup shim. The installed libc source is read
and inventoried in place, never copied. No old allowance was restarted.
