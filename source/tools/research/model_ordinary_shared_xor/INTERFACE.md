# Checked ordinary shared reads and XOR loop

Establish a genuine ordinary-memory counterpart of the checked SC reader/loop.
Keep the same loop algorithm and parameters; the only runtime expression change
is ScOrd to Na1Ord for reads/writes. This is a new MODEL ordering variant, never
a rewrite of the frozen native body or a native correspondence claim.

Derive a scoped key_reader_na from the existing shared view, ordinary E/L and
llft context. Reuse shared_key_prepare, then actual lifetime/physical guards and
wp_read_na_guarded_cells. Do not infer a Na read law merely from the SC reader's
black-box WP. Preparation costs g+1; a sufficient per-read budget is 3*(g+1).
Return the same linear reader token, persistent Cap, Frame and R/G restoration.
Actual ordinary reads have two runtime phases and allow arbitrary reader counts.

Prove the unchanged loop algorithm with this Na reader and ordinary pad stores.
The existing wp_write_na needs no extra input store credit with full cell
ownership; use a sufficient count*3*(g+1) read budget and account for all other
costs explicitly. Preserve exact min-length XOR prefix, untouched pad suffix,
byte range, arbitrary Frame, shared aliases and linear R. Cover arbitrary
pad/key lengths including 128-byte pads and empty/>128 keys, with no first-cell
access in the zero-count case. Persistent Cap is a separate base-case input.

For parent integration, a checked adapter may close the SC session from
ModelPairReader using its R/G closure, then open a fresh Na session at d=1,g=0.
If reusing its already prepared Cap avoids preparation, prove that fact; a raw
view still needs the advertised preparation budget. Preserve pad/Lpad, outside frame and
same Outer. Return genuine new session tokens; the closed SC reader must not
retain a usable old token. Extra proof credits are internally allocated by
public model adequacy, never appended to Rust application premises.

Only this new directory is owned. Exact pretrial freezes, source/object/receipt
pins, normal compilation, full declaration/obligation inventories and recursive
kernel checking. No baseline, compiler, vstd or sibling edits, new axioms,
primitives, unsafe modes, parent/sibling checkout reads or source/domain changes.
Partial-correctness MODEL facts only; native0, 44+3/101 open, publication false.
