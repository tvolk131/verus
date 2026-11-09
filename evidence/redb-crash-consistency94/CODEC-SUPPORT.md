# Narrow native codec dependency

The pinned native `header.rs::get_u32`, `TransactionHeader::from_bytes`,
`TransactionHeader::to_bytes`, and `btree_base.rs::BtreeHeader` use direct
`u32`/`u128` little-endian operations. Existing vstd defines the corresponding
mathematical byte specifications, checked roundtrip lemmas, and trusted
slice/vector wrappers, but lacks the four direct standard-library bindings.
The direct u64 bindings already follow the intended pattern in vstd/bytes.rs.

The proposed dependency repair adds exactly `u32::from_le_bytes`,
`u32::to_le_bytes`, `u128::from_le_bytes`, and `u128::to_le_bytes`, bound to the
EXISTING mathematical specifications, with native fixed array sizes and
`no_unwind`. It does not add a checksum axiom, trust native decoders or validators,
declare hashes injective, or alter the native source. These are individually
inventoried trusted standard-library boundaries; they are not checked Rust-core
implementations, nor a new Infer feature. Independent trust review is required.

Qualification must retain an old-runtime frontend witness, rebuild/verify the
matched vstd privately, prove all-input word and byte roundtrips and exact native
integer-read bodies, reject wrong-endian/changed-byte and short-slice controls,
and exercise a fresh ordinary export/import. Header/slot/tree validation and
crash correspondence remain OPEN after this support step.

## Subsequent leaf-count dependency

The exact native `LeafAccessor::new` also directly decodes its u16 pair count.
A separate preserved frontend witness (`source-runs/attempt-027`) confirms that
this standard-library call is missing, not a leaf proof failure. The same narrow
pattern is extended to `u16::from_le_bytes` and `u16::to_le_bytes`, with their
PREEXISTING specifications/roundtrip lemma and native two-byte arrays. This
adds two separately inventoried std boundaries, not a leaf-page validity axiom.
The original four-binding checkpoint and runtime remain unchanged. The u16
extension gets a separate matched vstd/runtime and remains unreviewed until an
independent boundary review explicitly includes it.

## Checked public expansion interface

External callers cannot reveal the existing closed u16/u32 byte specifications.
Four public checked lemmas expose their existing exact masks/shifts or bytewise
decoding equations: `lemma_u16_to_le_bytes_expansion`,
`lemma_u16_from_le_bytes_expansion`, `lemma_u32_to_le_bytes_expansion`, and
`lemma_u32_from_le_bytes_expansion`. The from-byte lemmas require the exact
two/four-byte width. These are proved inside the original defining module;
they add no trusted specification and do not make the closed definitions open.

Private codec-runtime-005 checks the complete matched vstd at 2225/0. Its
compiler binary is unchanged. Four all-input ordinary executable clients of
the new interface verify, as do two genuinely imported literal/extrema clients
(ten queries). Wrong-endian decode, reversed encode and missing-width controls
fail four intended backend obligations. An earlier literal-client draft failed
because bitvector checks lacked explicit scalar constant facts; it is retained
as an unsuccessful draft, not semantic-control evidence.

The six trusted direct std bindings and these four checked lemmas are distinct
inventory items. The original independent four-binding review does not approve
the later u16 bindings or checked expansion interface. All eighteen final
property/history cells remain open.
