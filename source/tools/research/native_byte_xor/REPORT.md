# Checked byte XOR and conditional cell update

This module adds an independent eight-bit Boolean/word interpretation and
checked MODEL execution results. Native correspondences remain **0** and
publication remains **false**. Selecting the frozen native instruction is not
an independently checked Rust-to-model simulation.

`bits` decodes little-endian bits using odd/div2. `word_xor` applies Boolean
`xorb` to pairs of bits. `word_value` sums positional weights using only addition
and multiplication. None of those definitions, or `scalar_xor8`, calls `Z.lxor`.
The proof first relates bitwise Boolean words to the library bit specifications,
then proves positional decoding exact within the unsigned width bound.
`scalar_xor8_lxor` proves agreement with literal `Z.lxor` for **every** pair of
bytes. `scalar_xor8_range` independently proves byte range (even when inputs are
arbitrary integers). No bounded sample or renamed numeric operation supplies
this fact.

`scalar_xor8_pure` and `scalar_xor8_normalizes` relate the result to the existing,
unchanged `ModelByteXor.byte_xor` call in exactly **three** pure primitive steps,
with unchanged physical state and no observations/forks. This claim concerns
only the scalar rvalue computation.

`cell_assignment p b` actually executes the existing `Read ScOrd p`, a concrete
call of `ModelByteXor.byte_xor` with the loaded byte and scalar `b`, then the
existing `Write ScOrd` to that **same p**. `wp_cell_assignment` supplies an
arbitrary frame `R` unchanged, and returns ownership of the updated cell and its
byte-range fact. Its internal precondition explicitly includes **old-cell
ownership and one store credit**; a pointer or copied register cannot produce
that ownership. These are conditional MODEL WP resources, not new original
application requirements.

The optional ordinary-data `prepare_assignment` adapter takes scalar local 19
from a register file, returns the concrete cell expression with that scalar,
and removes 19 from the returned register file. Checked lemmas prove 19 is
unavailable, every other register is unchanged, and missing 19 rejects.
`wp_prepared_assignment` packages those properties with the cell WP. This is
proof-level register bookkeeping, not a new heap-language primitive or an
execution rule for Rust MIR. Pointer local 17, its native interpretation and
incoming resources must still be supplied by a future correspondence proof.

## Source coverage and limits

The untrusted Python selector authenticates the committed SOURCE/source-results
pins, successful outer and inherited extraction receipts, raw JSON hash, and
canonical JSON hash before selecting enum payloads. Raw and canonical hashes
differ because their formats differ. It selects exactly ROOT bb7:s6:
`(*_17) = BitXor(copy (*_17), move _19)`, with local 17 `&mut u8`, operands and
destination `u8`, and no projection on scalar local 19. Unknown or extra payloads
reject. Nine controls change structured data only in memory: BitOr, wrong left,
wrong right, wrong destination, wrong width, Copy for Move, Move for Copy,
unknown field, and wrong pointer width. These are selector controls, not native
semantic negative proofs.

One scalar instruction payload is selected out of the 408-operation extracted
graph; **zero native correspondences are closed**. The actual incoming Rust
physical context, native reference load/store validity, provenance, retags,
phase transitions, register-state simulation and original C/F observations are
unproved. The entire assignment is not asserted pure. The full unchanged writer
and global goal remain unproved; frozen unsigned-varint/HMAC bodies and domains
are untouched. Inherited counters remain 44 minimal + 3 phase / 101 effects open,
canonical 6/0, and full frozen HMAC prefix AND suffix 1/1.

## Validation and artifacts

The early milestone is commit `c3799d9d`. The proof source first passed in
trial007; trial008 repeats it with stronger retained-freeze/helper inventory
checks. Trials001–006 are ordinary proof-development failures, retained
privately and classified separately from semantic evidence. Each trial has a
pre-execution committed-scope freeze and a new private output directory.

The final evidence and replay command are in EVIDENCE.json and REPRO.md. Normal
Coq 8.20.0 compilation and recursive `coqchk -o` cover the candidate and both
inventory modules, with no dependency exclusion. The inventory contains **26
candidate constants** (8 definitions/fixpoints and 18 lemmas), zero new datatype
constructors/eliminators/Program obligations, and **104** checked marker helpers
(52 in each inventory module). Every candidate's `Print Assumptions` is closed;
full recursive context reports only the two pre-existing baseline logical axioms
listed in TRUST_INVENTORY.json. No new axiom, contract, primitive or unsafe mode
is introduced.

The controller authenticates both foundation readiness and successful durable
build, 523 copied sources, 284 baseline objects, tool/loadpath hashes, the pinned
GMP manifest, and the completed 26-/8-constant model dependencies. It retains
JSON/freeze snapshots and copied proof sources and checks input/snapshot/object
stability before setting proof success. Raw logs, proof objects and auxiliary
files stay in private `/tmp` outputs. No shared baseline is written; no installs,
subagents, pushes, PRs or filesystem-control experiments are used.
