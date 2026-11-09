# Fresh bounded native redb leaf admission bridge

Status: checked conditional bridge, pending fresh independent review. **All 18 final redb cells remain OPEN.** This is not acceptance, trust/runtime adoption, an exported provider, or parent integration.

## Source and exact native preservation

Native pin: `c82ab865bb362bc4af067d332adc44978ff64813`. The recovered `btree_base.rs` SHA256 is `c3a1f5ca0cefdde0bc7d1b40536bc4f2d68a8bce852115433621a8b37fb9ec60`. `source-context/native-btree_base.rs` preserves the authenticated source context without modifying its original.

`candidate.rs` contains BOTH complete native helpers from original lines 1017–1031 and 1034–1048. Their entire bodies are byte-identical to the native originals, stronger than lexical preservation. The only native-helper changes are visibility (`pub(super)` to `pub`), named Boolean results, and checked contracts outside the bodies. No ghost code was needed inside either helper. Independent balanced-body extraction and lexical tokenization in `audit.py` checked every retained application source and the final candidate; see `native-preservation-audit.json`.

- Split body: 20 lexical tokens, body SHA256 `cf6946b84bd412511558ef2db756303422614a22c79c5b3cdf52eb7df9d2e720`.
- Fit body: 33 lexical tokens, body SHA256 `3a2f3f9ee610030c725c88401c08152bc7b8b2b2c28f783b327e2b651ef95807`.
- Fit calls imported `builder_step::RawLeafBuilder::required_bytes`, then compares capacity, then conditionally evaluates the unchanged `u16::try_from(num_pairs).is_ok()`.
- Split evaluates the entire `!leaf_fits_one_page(...)` call BEFORE its right-hand `num_pairs > 1` operand. Neither short-circuit order was swapped.

With ONLY imported `M + payload <= usize::MAX`, generic checked contracts establish fit iff `M + payload <= page_size && n <= 65535`, and split iff `n > 1 && !fit`. The split postcondition also establishes non-split implies fit when `n > 1`. No count/nonempty/capacity/u32 restriction was added to these scalar domains; zero, 65535, 65536, and the oversized singleton remain admissible scalar inputs under the arithmetic bound.

## Authored caller, not an original native caller

`fit_then_lookup` is explicitly AUTHORED. Its generic contract retains nonempty/equal key/value counts; exact reserved-key and payload sums; machine and u32 used-end bounds; matching optional widths; imported ordered four-byte numeric keys; and a four-byte query. It does NOT assume imported `request`, the count cap, or page-capacity fit.

The client calls copied native fit on the ORIGINAL page length. Failure returns `(false, None)` with no builder call. Success derives the EXISTING imported `request` in a checked proof block, then calls imported `lookup_client::build_then_lookup` with the ORIGINAL mutable-page borrow, not a replacement buffer. Its postcondition specifies the exact admission predicate, rejection None, accepted Some iff matching input query bytes exist, returned query bytes, and the originally associated value bytes.

`caller_witness` checks concrete LE keys 1 and 256 against present 256 / absent 2 queries, including exact returned key/value bytes. `concrete_caller_cases` calls that gate witness in all four independently fixed/variable key/value-width modes, with exact-fit capacities 12/20/20/28, spare capacity 32, and rejection capacity 3. Values are empty, as allowed by the prescribed witness design; the generic client contract covers arbitrary accounted associated values. These are verified source witnesses, NOT native application executions.

## Raw verification and semantic controls

Every application includes both original helper bodies and the authored gate in the full selected scope, without function filtering. Raw `stdout.raw`, `stderr.raw`, parsed verifier JSON, complete `input.rs`, result metadata, exact argv, complete environment, source hashes and dependency inventory binding are retained per attempt.

| Attempt | Purpose | Result / credit |
| --- | --- | --- |
| 001 | Generic helpers/gate + scalar positives | 4 verified / 0 errors; 4 checked executable bodies |
| 002 | Initial complete caller witnesses | FRONTEND failure: ghost comparison was in executable context; zero positive/negative credit |
| 003 | Move ghost comparison into proof block | One caller ordering assertion failed; ordinary correction only |
| 004 | Explicit quantified order/numeric facts | Two numeric ghost assertions failed in one function; ordinary correction only |
| 005 | Checked concrete bit-vector simplifications | FULL POSITIVE: 8 verified / 0 errors; 6 checked executable bodies |
| 006 | Incorrect exact-fit result | 7 verified / 1 error, ONLY intended backend assertion at line 83 |
| 007 | Incorrect 65536 count-cap result | 7 verified / 1 error, ONLY intended backend assertion at line 87 |
| 008 | Incorrect oversized-singleton converse | 7 verified / 1 error, ONLY intended backend assertion at line 90 |
| 009 | Byte-identical restored complete positive | FULL POSITIVE: 8 verified / 0 errors; 6 checked executable bodies |

The three negative sources differ from complete positive 005 by precisely ONE ghost assertion each; executable calls, identical arguments, valid preconditions, contracts and imports are unchanged. The exact-fit negative asserts false for `fit(2,8,Some(4),Some(0),12)`. The count negative asserts true for the 65536 call alongside the unchanged valid 65535 positive, both zero-width/payload0/page4. The singleton negative asserts `!split ==> fit` on count1/payload5/widths4,1/page8, whose positive has both split and fit false. `negative-*.diff`, raw diagnostics and `RESULTS.json` establish localization to `scalar_boundaries`; no frontend/import/precondition/panic/timeout/unrelated failure is counted as a semantic control.

Final source SHA256: `dbde2c19907bb6e579b593f8a0bc148f0e9325a8fcfc012999a5eed1895445ee`. Attempt005, attempt009 and `candidate.rs` are byte-identical. The verifier's 8 obligation successes include two checked bit-vector assertions; they are NOT eight named executable bodies. JSON reports 6 checked executable bodies and 0 checked proof-function bodies. The generated reveal marker has no translated body and is not counted as one; all three required executable functions are explicitly certified in the successful complete scopes.

## Authentication, environment and resource accounting

Entry, EVERY preexecution, and exit authenticate all 25 parent-authenticated assessment inputs, the specified design/report hashes, all 9 matched runtime files, 141 matched private-vstd source files (including conversion/result specifications), all three exact VIR/rlib import pairs and dependency directories, and 3340 historical sysroot files. There are 3517 unique expected bound paths. Full inventories and raw phase receipts are retained.

The historical sysroot expected hashes come from `preservation-baseline.json`, as used by the existing historical `run.py`. That additional read-only evidence input is separately content-hashed at entry/exit; it is NOT falsely represented as one of the parent's 25 authenticated assessment inputs. No current-parent source driver, operational91 driver or sealed runner was executed.

Only the specified private midpoint runtime and 1.99.1 aarch64 macOS sysroot were used. Logical flags and full environment derive from attempt011. The ONLY environment changes relocate writable HOME/TMPDIR/Cargo/Rustup/target/cache directories into each new scratch attempt; exact old and new bindings are recorded. All applications are verification-only, edition2024, release `debug-assertions=no`, `--no-cheating`, `--num-threads 1`, `--output-json`, `--rlimit 60`, timeout60, with NO `--compile` or `--export`. No runtime, toolchain or cache was copied. No compiler/vstd/Cargo/harness build or native application execution occurred.

9 of 16 applications were used: 3 clean positives, 3 intended negatives, and 3 ordinary correction attempts. There were 0 spawn failures, 0 timeouts, 0 authentication failures, 0 runner setup failures and 0 invocation/result metadata-correction failures. One missing-path reconnaissance read (`runner.py`) was corrected to `run.py`; both optional absent AGENTS probes and overbroad diagnostic output truncations are disclosed in `RESULTS.json`. None receives proof/control credit. The initial setup was NOT assumed feasible: the actual imports, conversion/result bindings, generic functions and complete caller witnesses were executed through the verifier.

Maximum verifier application duration: 0.816 seconds; total verifier subprocess duration: 5.422 seconds. Resource guards run before/after authentication and execution. Sampled minimum free space through result finalization is 55,520,100,352 bytes (>51 GiB), well above 16 GiB. The seal records final elapsed wall time and final scratch size, within 15 minutes and 128 MiB. There is one verifier application at a time and one solver worker. All work is in this NEW scratch directory; no network, Git writes, parent/frozen edits or subdelegation occurred. No previous ledger or refused workstream was resumed; leaf100 was untouched.

## Inherited assumptions and explicit limitations

This remains CONDITIONAL on the existing imported contracts, the six inherited little-endian boundaries, std conversion/result specifications, Rust/Verus/solver TCB, comparison-only trait treatment, authored builder loop/transfer adapter, and the private midpoint mathematical boundary. None is newly proved, adopted, or strengthened here. No new axiom, assume/admit, external body, trusted binding or engine/runtime change was added.

Recovered source hashes/pin provide qualified source correspondence only: original native paths, Git continuity, AST/nominal identity and whole-crate correctness are NOT restored or proved. No claim is made about the actual native allocator, accumulated-byte accounting, `LeafBuilder::build`, original caller execution, native ownership correspondence, `Drop`, destructor completion/unwind, panic paths, borrow-release glue, I/O, persistence, recovery, commit or transaction histories. Local checked preconditions and inherited verifier models do NOT close these native obligations.

No proof export was requested or attempted. Its unrequested absence is NOT refusal evidence. The directory is a content-hashed evidence handoff, not a standalone runtime distribution or a proof certificate; replay would still require the pinned external read-only runtime, toolchain and import artifacts. The seal is a hash inventory, not a signature, acceptance decision or new trust. Fresh independent review remains separate. All 18 cells stay OPEN.
