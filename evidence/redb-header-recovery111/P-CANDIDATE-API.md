# Lane-P stable source-module API

Root source: `/private/tmp/redb-header-recovery111-P-cPSXKcfk/recovery_product111.rs`.
Load as a source module; its relative `modules/` directory contains unchanged
contract111/composition110/buffered_flush109/ack_cfg sources. Use exactly the six
historical original `.vir`/`.rlib` provider pairs listed in every invocation.
No newly compiled provider or midpoint-vstd substitution is available or needed.

## Executable decoder

`pub fn decode_expected_header(data: &[u8], expected: &DatabaseHeader, file_len: u64)
 -> Result<(DatabaseHeader, bool), DatabaseError>`

`DatabaseHeader`, `DatabaseError`, and the Result alias are the actual nominal
`recovery_reencoding021` types. The bool is the original finalizer's clean flag;
the bridge does not require or promise that flag to be true in every accepted
layout-reconstruction case. The decoder directly invokes `parse_and_finalize`.

Requirements:

- `data@ == recovery_reencoding021::database_bytes(*expected, true)`;
- `recovery_roundtrip021::header_roundtrip_domain(*expected)`;
- `expected.two_phase_commit`;
- `recovery_roundtrip021::selected_slot_stable(*expected)`;
- `expected_file_length(*expected, file_len)`.

`expected_file_length` is explicit, not a recovery-result assumption:
if not recovery_required, stored layout length is at most file_len; if
recovery_required or stored length differs from file_len, original
`length_accepted(file_len, expected)` holds. Equal normal stored/file lengths
satisfy this immediately. For H's new normal selected slot, corruption=None and
two-phase mode establish selected_slot_stable; physical indices stay 0/1.

Return guarantees: always Ok under the requirements; original
`finalized_from_bytes(data@, file_len, actual, clean)`; exact selected physical
slot, transaction ID, user-root and system-root (`selected_payload_matches`);
all original transaction slots preserved; final layout length=file_len.

## Generic complete-product theorem

`pub proof fn public_ok_recovers_expected(start: product::Product,
 config: product::acknowledgement::Config, actions: Seq<product::Action>,
 expected: DatabaseHeader, file_len: u64)`

Requirements: unchanged product local_mode; product_invariant(start, config);
initial acknowledgement control; complete product run's public status=1;
start.geometry=file_len; start.second_header=database_bytes(expected, true);
`expected_domain(expected, file_len)` (the four header/length requirements above).

Guarantees: unchanged exact first/final successful complete-flush certificates,
ordered barriers/publication/acknowledgement, no blocked frame, drained writes,
durable image length=file_len, final durable prefix=expected original encoding,
original recovery acceptance, and exact selected payload for every original
finalized result on that prefix. Neither generated flush signals nor recovery
success/payload is a premise.

Use H's final serialization as start.second_header and its final actual header
as expected. The supplied decoder slice must equal the derived durable prefix
(or H's exact final serialization); no native fetch or callback frame is
provided by this bridge. Config.new_slot/old_secondary are model PAYLOAD LABELS,
never physical header indices or transaction IDs.

## Dependency handoff received, not rechecked

The parent supplied terminal lane H:
`/private/tmp/redb-header-recovery111-H-20261108T052400Z/staging111.rs`, SHA256
`0d894c65c6315d1c0de84c6f06a797b9b448d55b9539c542d020502345b6bc31`.
Reported H manifest SHA256:
`74f9c69f848a3ed7342bce0a8ebc523e53fbe929576a55b50a89c2b06a044211`.
Reported H HANDOFF SHA256:
`615ba834c3ee57e7584091c49aa8b556e208efb5250f3607eed28819805b2e60`.
No H verification, integration, scope extension, or additional authorization
is inferred. Parent retains its own remaining two applications for composition.

All APIs retain EXPLICIT_INHERITED_STD_BOOL_TCB_NOT_RUST_CORE_PROOF, conditional
and not adopted; native preservation FAILED, native Git UNAVAILABLE, compiler
correspondence UNESTABLISHED, and all eighteen original cells OPEN.
