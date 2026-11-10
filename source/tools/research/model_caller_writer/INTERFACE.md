# Model writer within an existing caller context

Prove the concrete shared-key XOR procedure under an existing typeG, ordinary
E/L/I and typed mutable-pad/shared-key inputs. Do not initialize a fresh global
heap/TypeG or end the caller's parent lifetime. Create/end only the shorter
child required for the computation, then restore the existing parent with its
new current bytes and its original future unchanged. Preserve arbitrary ancestor
Frame and the original E/L/I. No supplied physical PointsTo or exclusive key
vector; derive access from the existing semantic reference resources.

Use the checked shared reader and loop, generic shorter_run_resources and actual
unique-body accessor. A shared reader session over the same ordinary L splits
it into R/G: keep R available for callback reads while G guards the context for
mutable access, close the same child body, then join R/G before child ending.
Do not demand a second independent copy of L or assume parent future=current.
Explicit aliveness and internal credit indices are model typing premises; no
new original Rust application requirement or source input interpretation follows.

Keep all pad/key contents and finite lengths, including128/empty/>128. Prove
the exact min-length XOR prefix, untouched suffix and typed parent-current
relation. The returned parent may still be used by its caller. This is a typed
MODEL procedure law, not a final physical heap theorem or actual Rust/MIR proof.
Actual incoming Rust context/provenance/ancestor-history and C/F interpretation
remain unproved; native0,44+3/101open,pubfalse.

Only this new directory is owned. Early typed interface, pretrial freezes,
normal compilation, complete assumptions/obligations inventory and recursive
kernel checking. Read-only pinned dependencies and explicitly delivered
committed sources only. No parent/sibling checkout reads or edits, new axioms,
primitives, unsafe modes, compiler/vstd/shared changes, subdelegates or push/PR.

## Exact early typed milestone

`candidates/interface.v:caller_xor_contract` fixes the expression, resources,
result vector and full postcondition before proof expansion. The theorem will
quantify over all `n k E L I tid iκs κ κkey p q x key m d g Frame` and assume
only these internal MODEL typing conditions:

- `lctx_lft_alive E L κ`, and aliveness of `ty_lft (&uniq{κ} (bytes_ty n))`;
- `lctx_lft_alive E L κkey`;
- `uniq_bor_loc m = (p, repeat [] n)` and `uniq_bor_current m = x`.

The single ordinary `llctx_interp L` supplies both pad and key access. The
credit index is `(g+1) + (1 + min(n,k)*(g+2))`; this is an internal symbolic
logic index, not a bounded source execution budget or a Rust entry premise.
The body is the existing concrete `zip_transform byte_xor (min n k) p q`.
The only lifetime effects around it are `Newlft; ReBor; ...; Endlft; Skip`.
The shared key view remains reusable, including by existing shared aliases.
No disjoint-key vector, heap ownership, range or output-summary premise occurs
in the public contract. Byte initialization is derived from semantic types.
The result is a vector of the same length, and the parent observation preserves
its exact original future while changing current to the exact XOR result.
