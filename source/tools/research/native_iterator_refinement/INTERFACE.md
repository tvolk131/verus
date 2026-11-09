# Source-aware native iterator refinement: first milestone

This workstream does not yet prove the native HMAC pad-XOR theorem. The frozen
128-byte arbitrary-pad/arbitrary-key benchmark and executable loop stay fixed.
No compiler, standard model, application, or sibling module is changed.

## Pinned native path

Rust 1.99.1 `rust-src` was absent and installed using the explicitly authorized
`rustup component add rust-src --toolchain 1.99.1`. The root is
`/home/dev/.rustup/toolchains/1.99.1-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/`.
`source-map.json` binds full files and source excerpts before proof trials.

- `library/core/src/slice/iter.rs:192`: `IterMut` contains `NonNull<T>`, an end
  pointer/length, and `PhantomData<&mut T>`. There is **no owned Rust `&mut [T]`
  field** for the structural resolver to propagate to. `new` is at 219.
- `library/core/src/slice/iter/macros.rs:157`: ordinary `next` increments the
  pointer. At 413, `__iterator_get_unchecked` returns a reference at `ptr + idx`
  without changing that pointer.
- `library/core/src/slice/iter.rs:2987,2997`: both slice iterator types implement
  `TrustedRandomAccess`. Thus the target uses
  `library/core/src/iter/adapters/zip.rs:288`'s specialized implementation:
  `new` stores the minimum length; `next` increments Zip's `index` and calls both
  unchecked getters. It **does not call inner `next`**. `Zip`'s fields are private.

Consequently a checked correspondence proof must account for issued-index
permissions/prophecies; matching a safe split-first model to a guessed
sequential-inner Zip implementation is insufficient.

## Existing interfaces and the missing certificate

`source/rust_verify/src/rust_to_vir_adts.rs:655` translates opaque external
types to datatypes with no fields. Transparent proxy import at 683 reads actual
Rust fields, but rejects private fields at 698. `IterMut`'s real pointer fields
would not themselves provide ownership of the referents anyway. External
function proxies check signatures/bounds; they do not check the linked native
implementation. The comment at `source/vstd/std_specs/slice.rs:496` asks model
changes to be verified in a separate example; it is a maintenance convention,
not a mechanically checked native correspondence mechanism.

`source/vir/src/resolve_axioms.rs:219` propagates resolvedness only through
visible non-spec fields of transparent datatypes without destructors.
`resolution_types.rs:26` treats opaque types conservatively as potentially
unresolved. `resolution_inference.rs:3579` can expose safe resolution for a real
place, but does not turn an arbitrary ghost observation into an owned place.
This rules out fixing the gap by merely asking for more object-resolution facts.

An acceptable source-aware certificate would need:

1. Identity of the pinned monomorphized native constructor/next/getter/drop path,
   including Rust specialization, and checked translation of those bodies.
2. A linear ghost ownership witness for all 128 referents, split into issued
   and retained indices, connected to the actual native pointer/provenance state.
3. Checked constructor, successful-next, None-next and scope-exit obligations;
   every unissued permission must resolve without modification, including the
   empty-key case. Issued permissions retain their writer-derived final values.
4. A checked link from that witness to the **existing** `remaining` and Zip
   accessor observations, plus original-buffer initial/eventual correspondence.
5. A generic certificate checker, rejection of altered source/false ownership
   witnesses, and preservation of the original executable projection. A native
   name/hash allowlist that emits a new axiom would not discharge these duties.

No such certificate importer/checker has been found in the current HIR-to-VIR
frontend. A compiler implementation would require new trusted translation and
soundness work, not just an application proof helper. No such rule is applied.

## Next checked evidence

Freeze an explicit value/prophecy **abstract countermodel**, not a second
custom executable iterator. Check that initial/eventual correspondence,
truncated Zip next/history equations, and resolved opaque-object tokens permit
a changed suffix. Its scope is the enumerated observation equations, not a
claimed interpretation of all vstd/Rust axioms. Separately probe whether ghost
observations can be promoted to owned tracked witnesses; frontend rejection is
classified independently from semantic proof failure.
