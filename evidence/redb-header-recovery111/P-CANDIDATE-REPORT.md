# Round111 lane P: checked product bytes to original recovery payload

## Result and exact consumer surface

Implemented `recovery_product111.rs` as a source-module consumer of the actual
nominal `recovery_reencoding021::DatabaseHeader`, with exact existing
`recovery_roundtrip021` and page/root/hash/hash-dispatch metadata/binary pairs.
The unchanged contract111/composition110/buffered_flush109/ack_cfg source copies
live in `modules/`; all four are hash-identical to the frozen shared sources.
No provider, native application, compiler, runtime, or vstd was built or copied.

`decode_expected_header(data, expected, file_len)` is executable Rust/Verus source
that calls the imported original `parse_and_finalize` on the supplied slice.
Its requirements explicitly retain `data@ == database_bytes(*expected, true)`,
`header_roundtrip_domain`, two-phase mode, `selected_slot_stable`, and the
original normal/recovery-required file-length conditions. The proof constructs
an exact parsed-encoding witness using the original encoder/parser roundtrip
contracts; it proves finalization acceptance rather than assuming recovery or
successful decoding. The result is always Ok under those premises. It preserves
the selected transaction ID and both complete associated optional BtreeHeader
values, including their page/checksum/length fields. All transaction-slot values
are preserved, and the finalized layout length equals the supplied file length.
Layout reconstruction is allowed; whole-header equality is not overclaimed.

`public_ok_recovers_expected` calls unchanged `product::checked_run` and
`product::public_ok_composes`. For arbitrary complete product actions, its
premises retain the local-mode gate, checked initial product invariant/control,
public model Ok, exact second-header encoded origin, expected-header recovery
domain, and model geometry equal to the supplied file length. No desired flush
signals, successful flush snapshots, or recovery payload are premises. The
unchanged complete model_flush product establishes both exact successful
barrier certificates, history ordering, drained pending writes, rejected-frame
absence, and the final durable prefix. The bridge then proves that this prefix
is accepted by original recovery and every actual finalized result has exactly
the expected selected transaction ID and its two associated roots.

The callback gate remains the exact unchanged product `framed` gate. A model
effect that changes image/pending is blocked, not silently admitted. This is a
conditional model composition, not a proof of native callbacks or history.
The source-module API is suitable for a later parent consumer of lane H's
checked original-method staging result; no H result is assumed or imported here.

## Nonvacuity and property-specific controls

`witnesses111.rs` has no-premise executable `concrete_decoder_nonvacuity`, which
constructs the actual nominal header, calls its original `.to_bytes(true)`, and
calls the checked decoder. This is verified executable source, not a native run.
Physical slot 1 holds transaction 11; slot 0 holds transaction 7. User root is
page(region=2,index=3,order=0), checksum 101, length 17. System root is
page(region=4,index=5,order=0), checksum 202, length 19. Both are Some and distinct.
The valid complete file length is 8192 (4096 database-header page plus one 4096
region); the earlier mistaken 4096 witness and its corrections are retained.

`concrete_product_nonvacuity` starts from an actual clean 8192-byte model image
and empty pending queues, uses 46 actual complete product success actions, and
derives public model Ok and accepted recovery from the final durable prefix.
It does not assume the generated signals or public success. The acknowledgement
model's `new_slot=3` and `old_secondary=1` are arbitrary PAYLOAD LABELS. Physical
header selection is 1; label 3 is deliberately not a physical 0/1 index. No
identity between model labels, physical indices, and transaction IDs is assumed.

The final semantic-controls source has three actual-decoder assertions. Check 08
changes only transaction 11 to the wrong old transaction 7; check 10 changes only
the user-root expectation to the distinct system root; check 12 changes only the
system-root expectation to the distinct user root. Each reaches the intended
backend assertion, with 70 verified / 1 error and no frontend or VIR error.
Checks 09/11/13 restore the baseline source byte-for-byte and pass all 71 checked
functions with zero errors. Each exact fresh-absent export path is inspected
after every run; failed negative runs have no export. Positive export metadata
hashes may differ between runs and are recorded rather than presumed identical.

## Accounting, integrity, and retained failures

First own clock is November 8, 2026 05:24:56 UTC; own deadline 05:49:56 UTC,
optional stop 05:46:56 UTC, parent absolute handoff 05:54:18 UTC. One verifier
worker, 13 charged applications including all failures, maximum 90 seconds each,
768 MiB scratch cap, global free disk strictly above 16 GiB. Final accounting and
sampled resource values are in `ACCOUNTING.json`; samples are not continuous peak
measurements. No timeout, cancellation, or extra application is omitted.

Applications 01/03/04/05/06 retain the implementation failures: solver resource
limit in the initial generic proof, historical Map constructor interface, finite
Set constructor interface, concrete witness/domain/array proof obligations, and
concrete canonical-page shift/file-length obligations. All raw stdout/stderr,
argv/environment, source scopes and snapshots, authentication observations,
fresh-export existence/hash results, and charged counts are retained. None of
these failures is counted as a property-specific semantic negative.

All 1,416 frozen records are authenticated at entry/preexecution/exit of every
application and again at final exit. The actual executable midpoint
verus/rust_verify/z3 identities equal the historical nonexecutable originals;
the originals are never chmodded or copied. VERUS_ROOT is the historical
matching recovery021 runtime/vstd, VERUS_Z3_PATH is the executable midpoint z3,
and VARGO_TARGET_DIR is absent. Exact imported provider pairs and direct tool
identities, including the pinned direct Python interpreter, are recorded.
The full crate is verified with `--no-cheating`, no scope filters, no --compile,
one verifier thread, and a fresh requested proof-metadata export per application.
Fresh exports are not paired with newly built native rlibs or promoted as native
library admissions. Hashes establish integrity, not compiler correspondence.

All authored edits use apply_patch. Scratch-only source copies and evidence
snapshots are byte-preserving copies, not modified providers. `final-source.patch`
contains the authored Rust additions; `HANDOFF.json`, `MANIFEST.json`, and
`SEAL.json` bind the final sources, report, controls, accounting, and raw evidence.
The only array extensionality call in the witness uses the already inherited
vstd contract; no new axiom, law, assume, admit, external_body, or gate is added.

## Explicit inherited boundary and open work

EXPLICIT_INHERITED_STD_BOOL_TCB_NOT_RUST_CORE_PROOF remains conditional and is not
adopted. Native-path preservation FAILED. Native Git continuity UNAVAILABLE.
Compiler/source correspondence UNESTABLISHED. All eighteen original property and
history cells remain OPEN. This result does not establish arbitrary-medium
recovery, native fetching, write admission/coalescing, backend crash/byte effects,
actual callback/history frames, allocation, ownership, Drop/lifecycle, or native
acknowledged durability. Encoded origin and length premises remain explicit.

No network, Git writes, parent/frozen mutation, integration, runtime build,
native execution, subdelegation, new trust, cap reset, extension, or automatic
followup occurs. No excluded growth049/zero-growth050/mutation030/padding95/
WritablePage/Drop/lifecycle/diagnostic107-D1 work is resumed or rerouted.
