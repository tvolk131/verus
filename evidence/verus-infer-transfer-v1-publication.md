# Publication of the frozen unrelated-application baseline

Published against `verus-infer`, not `main`. This is the third PR in the current
series: publication of existing evidence, not another application experiment or
an engine extension. The [original report](verus-infer-transfer-v1/README.md)
remains the source of the frozen experimental claims.

## Result and scope

The target is unsigned-varint **0.8.0**'s original `encode!` macro and `u64` body,
with every `u64` and every arbitrary initialized, exclusively borrowed `[u8; 10]`.
The required property is canonical unsigned base-128 encoding. Neither the
executable body nor the required domain was simplified after failure.

All **16 application attempts were frontend-rejected**, with **zero application
proofs, generated candidates or exports**. The first eight attempts also exposed
an adapter omission: the unchanged `U64_LEN` constant was outside verification
scope. After the constant-only enclosure correction, eight further attempts
isolated the unsupported macro-produced `for` loop. Initial and corrected
fixtures, summaries and raw logs are all preserved. These are frontend timings,
not proof-performance or assistance-quality measurements.

Native debug/release each passed 3 tests covering **173,503 input/buffer pairs**.
The independently built matching engine and vstd passed their build/sanity
checks, including cold/warm sanity with fresh required proofs. These are tooling
checks and finite native tests, **not an application proof success**, an upstream
defect, or evidence that the application property failed. No semantic mutation
control was run because no positive application proof existed. The suggested
macro-loop adapter remains a future extension, not part of this publication.

## Exact provenance and immutability

- Original local evidence commit:
  `3147acb47fcee7b9c359e2a5367325e34d88d4bd`, from
  `t3/verus-infer-transfer-experiment-1`.
- Its parent and engine baseline:
  `45f93e51f4d0c0f1b71ef645932d54a7862ec983`.
- Publication base: `9c3b610b0a92581109ab433742a689db05f2fa63` on `verus-infer`.
  `git diff` between those engine/base commits is empty for `source`,
  `dependencies`, `tools` and `rust-toolchain.toml`. This PR adds only evidence,
  its publication link, and Python audit/tests; verifier, macros and vstd are
  unchanged.

All **227 original Git blobs** were restored and compared against the original
commit. The original `evidence-index.json` remains byte-for-byte unchanged:
SHA-256 `f18a65b05415b158dfe60e984baa958231d906e4ef802821d73ed875cab08c28`.
Its **226 entries / 603,898 bytes** exclude the index itself. No historical
selection, fixture, source, audit, fingerprint, failed replay, successful replay,
raw log or report was regenerated. In particular, the original report's
"no push/PR" statement describes the earlier local capture; it is intentionally
not rewritten to describe this later publication.

The original `audit.py` is a capture-time tool: it relies on original absolute
artifact paths, uses assertions, and rewrites `audit.json` and the index. **Do
not run it to validate the published capture.** Use the separate read-only checker
below instead. No original capture script was imported or executed in place to
regenerate evidence.

## Offline checks and CI

From the repository root, with Python 3.9 or newer and only its standard library:

```sh
python3 -B source/tools/check_infer_transfer_evidence.py \
  --replay-root evidence/verus-infer-transfer-v1-publication-replay
python3 -B -m unittest discover -s source/tools -p 'test_infer_transfer_evidence.py'
```

The checker pins the historical index itself, checks all indexed hashes/sizes and
the exact file inventory, rejects symlinks, validates all **62 historical command
receipts** and stream hashes (32 primary, 16 corrected replay, 14 failed replay),
compares upstream snapshot files with the bundled archive without extracting it,
and cross-checks trial summaries against raw frontend reports, native results and
fresh mandatory sanity proofs. The failed replay's two missing launch receipts
are historical limitations: their diagnostics and failing replay summary remain
preserved, not reclassified as successes. The checker does not execute captured
commands, access the network or inspect current binaries. Failures raise explicit
errors and return a nonzero exit even under `python3 -O`.

Tests cover immutable/read-only success, changed/missing/extra files, symlinks,
an attacker-rehashed index, receipt inconsistencies, incomplete commands, source
preservation, a false proof-success claim, optional sanity obligations, optimized
Python, and the separate publication replay receipts/artifact claims. The existing
`infer-tooling` job in `.github/workflows/ci.yml` discovers them using
`python3 -B -m unittest discover -s source/tools -p 'test_infer*.py'`; no workflow or
third-party Python dependencies were added.

Publication validation passed both normal and optimized-Python offline checks,
all **92** tests in that existing discovery command (including the 14 new transfer
tests), and `git diff --check` restricted to authored documentation/Python files.
The full staged whitespace check reports existing blank lines at EOF in captured
raw stdout/stderr, including the new replay, and upstream `src/encode.rs`; those
bytes are intentionally not normalized. The eight original engine/vstd artifacts and
rustc, rustc_driver, Cargo and Z3 were also rehashed after replay and still matched
the frozen fingerprints. No repository-wide engine build was needed for this
evidence-only publication.

This is a hash/source/log consistency audit, not a proof certificate, an
authenticated upstream supply-chain audit, or a source-to-binary attestation.
Checking captured binary fingerprints does not authenticate a newly built engine.

## Separate publication-time replay

On October 31, 2026, a new replay started at **21:36:19 UTC** using the original
read-only engine artifacts and solver still present in
`/Users/tommyvolk/.t3/worktrees/verus/t3-10dbd998`. The command was:

```sh
python3 -B evidence/verus-infer-transfer-v1/replay.py \
  --engine-root /Users/tommyvolk/.t3/worktrees/verus/t3-10dbd998 \
  --output /private/tmp/verus-infer-transfer-publication-20261031-5399f1e6
```

The original replay script was read from this publication checkout; it wrote new
sources, caches, exports, native binaries and logs only under the new `/private/tmp`
output. No other checkout or original artifact was modified, no worktree was
created, and no engine was rebuilt. All **16 replay commands validated**: six
native build/test commands exited 0, eight corrected-constant application runs
exited 1 with the same macro-loop frontend blocker, and two sanity runs exited 0
with the required proofs. The raw commands, outputs and receipt are archived
separately in [the publication replay](verus-infer-transfer-v1-publication-replay/).
This does not add eight new attempts to the frozen 16-attempt baseline.

The replay measured hashes of the actual reused binaries: all eight engine/vstd
artifact hashes and the solver hash matched the frozen receipts. This is measured
reuse of the original build, **not** a claim that an independent current rebuild
would have those hashes. The new replay's paths, timestamps, timings and raw-output
hashes are distinct evidence and were not substituted into the frozen index.
The optional `--replay-root` offline check validates its 16 command receipts,
32 stream hashes, expected statuses and recorded artifact identities against the
historical successful replay; it does not rerun or authenticate binaries.

For another host, use a **new** output directory. Replay requires a compatible
macOS/aarch64 environment, Rust 1.99.1, the pinned source/dependency cache, matching
engine/macros/vstd and Z3 4.16.0. The original reproduction recipe describes a
fresh offline build and retained vstd lockfile. Pass an unchanged engine checkout
with `--engine-root`: the original replay's source guard includes all of
`source/tools`, so this PR's additional audit scripts make the publication
checkout itself unsuitable as that engine root, even though the verifier is
unchanged. With a separately rebuilt engine,
explicit `--allow-rebuilt-artifacts` records hash differences as distinct evidence;
never rewrite the frozen fingerprints or assume source equivalence proves build
provenance. Ordinary offline receipt checks do not require those tools or paths.
Single-repetition process timings do not establish general performance results.

## License and publication audit

The bundled unsigned-varint archive SHA-256 is
`eb066959b24b5196ae73cb057f45598450d2c5f71460e98c49b738086eff9c06`.
The package metadata records revision
`10d55e5dbdd7f61b6495bd12a49d06df82e08612`. The MIT license and Parity Technologies
copyright notice remain in `verus-infer-transfer-v1/upstream/LICENSE`; all seven
snapshot members, including license/metadata and complete encode/decode/lib
sources, match their archive members byte-for-byte. The archive and cached
registry checksum agreement is local provenance, not independent network or Git
revision authentication.

Publication audit scanned all 280 changed files (including 276 frozen/replay
files) and all 22 regular archive members for GitHub/API tokens, AWS access keys, private-key blocks, JWTs,
credential assignments and authorization headers; no matches were found.
Captured environment fields are a tooling/path whitelist rather than a full
environment dump. Personal absolute paths, package metadata and public hashes
are deliberately retained for provenance. This bounded review cannot guarantee
absence of every possible secret encoding; no credential or private environment
file is intentionally published.

Publication checker, tests and note: AI-assisted implementation with OpenAI
Codex:gpt-6.1-sol. The original report retains its own assistance disclosure.
