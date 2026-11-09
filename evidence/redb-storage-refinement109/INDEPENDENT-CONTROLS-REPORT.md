# Independent round109 finite controls: implementation handoff

**Candidate evidence only. No acceptance, adoption, native producer or integration.**
All eighteen native final cells remain OPEN. No parallel proof worker's output
was read, awaited or polled; no subdelegation occurred.

## Implemented and exercised

- Python stdlib-only independent storage/outcome implementation, 21 unittest
  methods, 12 authored golden outcomes with ordered/forbidden events and source
  anchors, and a machine-readable complete enumerated case inventory.
- 844 storage parameterizations / 7,172 event transitions / 750 distinct terminal
  states. Both two-stripe orders, both within-stripe orders, every failing write
  index, partial old/intended/unknown write-error effects and persistent effects.
- 195 commit/opaque parameterizations / 5,190 event transitions / 103 distinct
  terminal outcomes: 176 Err, one Ok, 18 explicit OPEN_CALLBACK_EFFECT endpoints.
- 12,644 event-prefix crash observations: 5,385 process-restart and 7,259 power-loss.
  These are finite raw-image observations, **not recovery results** or native
  executions. Duplicate parameterizations and unique terminal counts are separate.
- Six deliberately wrong semantic programs have corrected counterparts: premature
  Ack, omitted prepare barrier, lost coalesced latest header, draining failing
  stripe, Err-implies-rollback, and early-secondary-is-fresh. Seven source mutants
  mismatch the guards; originals/corrected fragments pass. Native sources are
  never rewritten. Byte identity is authenticated separately from lexical rules.
- All execution attempts, including expected nonzero controls, retain real child
  statuses, full stdout/stderr, argv, environment, timing, interpreter/input hashes.
  Source control exit 3 = MISMATCH, 4 = REFUSAL, 5 = lexical PARSE_FAILURE; only 3
  counts as source-mutant rejection. Crashes/timeouts are not rejection success.

## Reviewer compatibility checks

1. **Buffered candidate:** distinguish pending / visible / durable. Two full header
   writes at offset zero replace the pending value until issuance. Do not equate
   enqueue with native WritablePage admission; Drop/loan/glue binding is OPEN.
2. **Failure prefix:** a failing stripe retains all entries, including its earlier
   successfully issued writes. Earlier stripes may already have drained. A write
   Err can affect bytes and persistence; an Err is not an atomic no-op. The trace
   must not proceed to sync after flush_write_buffer returns Err.
3. **Barrier semantics:** sync Err does not undo visible or durable data. Successful
   sync semantics are a reference-model law only. A proof must expose any native
   primitive obligation instead of importing this finite oracle as trusted truth.
4. **Public candidate:** prepared successful barrier before selector publication;
   final required successful barrier before public Ok. Derive events from checked
   control rather than imposing the target trace as a theorem premise. Header-hold
   and admission errors must be present, not just sync errors.
5. **Actual continuation:** `transactions.rs:2174` can return Err after the manager
   has durably published user data. At `2180` the epilogue has six selected fallible
   stages, including non-durable commit and registration (`2244`, `2252`). No
   unconditional public-Err-implies-rollback lemma is compatible with these controls.
6. **Stale secondary:** initial secondary transaction 1 is stipulated valid/stale;
   primary transaction 2 is old. Prepared transaction 3 only exists after preparing.
   Do not identify every valid secondary at every early crash with transaction 3.
7. **Opaque callbacks:** preparation quick-repair loops and post-publication tracker,
   free and savepoint callbacks have nontrivial effects. The OPEN endpoints refuse
   to invent their behavior. The successful/no-extra-effects traces are conditional
   finite witnesses, not proofs that callbacks cannot write, panic or allocate.
8. **Scope gates:** fixed geometry, single process, Immediate/two-phase only. Shared
   direct header writes and the third post-barrier publication write, post-barrier
   shrink/resize, non-durable modes, poisoned abort, full operation histories,
   resource/panic/abort/lifecycle and crash recovery remain OPEN. See the selected
   question-mark inventory and `BRANCH_COVERAGE.md` before claiming compatibility.

## Artifact guide and reproduction

`oracle.py`, `source_guards.py`, `test_controls.py`, `controls_cli.py` are actual
implementations; `expected_outcomes.json` is authored independent golden data.
`case_inventory.json`, `crash_inventory.json`, `semantic_mutants.json`,
`source_mutants.json`, `source_anchors.json`, `source_characterization.json`, and
`coverage.json` retain concrete events, images, results and source locations.
Each `attempt-NNN` contains command/receipt JSON and unedited output streams.

From this scratch directory, replay using:

```
/usr/bin/env -i PATH=/usr/bin:/bin HOME="$PWD" TMPDIR="$PWD" LC_ALL=C TZ=UTC PYTHONDONTWRITEBYTECODE=1 PYTHONHASHSEED=0 /usr/bin/python3 -B run_evidence.py
```

A replay appends a new attempt and invalidates the old seal; run `seal.py` again
afterwards. It invokes only Python, unittest and authored scripts. No Cargo,
compiler, verifier, solver, Rust/native application, network or Git invocation.

## Authentication and limits

The requested PLAN and INPUTS pins matched before content use. Native root AGENTS
was read and scoped; its Cargo-oriented instructions do not authorize forbidden
builds and no native file was edited. Selected native sources and interpreter
matched INPUTS. All five disclosed historical document drifts are retained as
drifts in authentication records, not silently repaired. Historical documents
were hashed, not used as semantic premises. Supplemental selected native/AGENTS
hashes were in INPUTS; authored execution inputs were hashed before each attempt.
The Python launcher, actual interpreter path and apply_patch shim are recorded.
Final entry/preexecution/exit equality checks do not imply an atomic read snapshot.

Bootstrap discovery used inherited-shell read-only commands; its first combined
display was truncated and the plan/source were read again in bounded ranges.
The replayable proof-control executions, unlike exploratory shell display, have
explicit minimal environments and full retained streams. No exploratory failure
occurred. Expected nonzero mutant/refusal/parser control statuses are intentional
and separately recorded, not relabeled verifier failures.

First resource/time observation: 2026-11-08T03:42:34Z; own 13-minute optional cutoff
03:55:34Z and 16-minute hard bound 03:58:34Z precede the round's deadlines. Resource
samples are not continuous monitoring or a hard quota: all observed global-free
samples exceed 16 GiB; scratch is below 64 MiB, with final values in FINAL_STATUS.
After optional work stops, only validation, authentication and retention run.

Native path preservation FAILED_INHERITED; Git continuity UNAVAILABLE_INHERITED;
compiler/source correspondence UNESTABLISHED. Hashes do not repair these facts.
No original final-cell gate, S0/full-scope/Infer/import-export barrier, trust,
admission, export or source integration is changed. Excluded growth049,
zero-growth050, mutation030, refused-padding95 and refused drop-pipeline work
was not resumed or rerouted. Parent retention/publication remains its own task.
