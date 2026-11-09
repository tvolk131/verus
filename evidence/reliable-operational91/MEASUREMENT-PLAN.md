# Campaign91 measurement plan

Freeze the measured work before interpreting timings. Consolidation already
records one fresh VIR unit/doc regression, two Python modes and two 29-case
Fedimint matrices. These are observations, not repeated benchmark estimates.
Historical workflow90 compiler/vstd/regression costs remain historical.

The next source-build and matched-vstd run are fresh acceptance work, measured
once and explicitly distinguished from clean-cache builds. Reusing a private
Cargo development cache is permitted, never relabelled as a cold build.

For repeated workflow costs, perform three fresh-output stateful replays with
the exact same final qualified runtime and copied qualified test executables.
Keep fixture/oracle/source/native bindings, commands and selected proof scope
identical. Record staging, validation/preflight, eight native application runs,
focused regression methods and total wall time. Each replay must execute the
verifier again; no previously successful report may stand in for execution.
Record the first invocation separately because filesystem/process caches may
differ. Do not claim three runs characterize all hardware or all Infer use.

Also measure source/artifact hashing and staging independently with the same
artifact set and three repetitions where those components are suspected to
dominate. Require unchanged artifact hashes and unchanged outcomes across any
before/after speed comparison. Report component fractions and ranges rather
than adding overlapping process/suite times together.

Only implement an additional tooling optimization if this evidence identifies
a reproducible, material development-loop bottleneck. At most two are allowed.
Do not spend a full compiler rebuild solely to obtain a number; record missing
cold measurements honestly. A well-supported no-optimization decision passes
the measurement phase, but has no speedup credit.
