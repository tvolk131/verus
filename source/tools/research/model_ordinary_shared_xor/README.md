# Ordinary shared reader and XOR loop (MODEL)

`candidates/reader_na.v` derives indexed `Read Na1Ord` directly from the existing
shared view, ordinary lifetime contexts, and actual indexed physical guards.
Both the ordinary-context and scoped-session rules apply
`wp_read_na_guarded_cells` at depth `g+1` with `3*(g+1)` credits. They do not use
the SC reader WP to prove a Na read law. Each read returns its exact linear R,
persistent Cap (same n/κ/q/x/d/g/tid), and arbitrary Frame. Session closure joins
R/G back into the original ordinary L. Reader and Cap are persistent aliases;
the token is linear. No exclusive key vector is extracted.

`shared_key_reader_na_session_prepared` constructs a fresh session from ordinary
L and an existing Cap with zero additional preparation credits. The raw-view
wrapper `shared_key_reader_na_session` calls the existing `shared_key_prepare`
with `g+1` credits. Preparation is once per raw session, not once per index.

`candidates/xor_na.v` retains the frozen `zip_transform` expression tree and
parameters. Only the two read orders and single write order change. The replay
compares the exact bodies after those three substitutions, ignoring whitespace.
The XOR operator itself and all old Rust/SC sources remain frozen. The loop
uses the Na reader for key cells and `wp_read_na` / `wp_write_na` for owned pad
cells. A sufficient budget is `count*3*(g+1)`; the pad read and store need no
extra input credit. The generic contract retains the original key base with
advancing `start`, exact `transform_prefix`, untouched suffix, length, byte
range, Cap, R and Frame. Separate Cap input covers count=0. The explicit zero
control needs no Reader, makes no cell lookup/access, and allows p=q. Min and
128-byte specializations cover every finite key length, including 0 and >128.

`candidates/adapter.v` consumes the SC session's R/G and closure to recover L,
then opens a fresh ordinary session from the same prepared Cap. It returns no
old R. A retained persistent SC reader therefore has no old token provided by
this adapter. The generic adapter frames any assertion. The fixed
ModelPairReader callback wrapper explicitly frames PadRef, Lpad, invariant
context, exact outside Frame and all remaining credits. It uses the existing
TypeG and exactly the same Outer. No extra preparation budget is needed for
this already prepared output. The public observed setup wrapper internally
allocates `2+k` credits through model adequacy, just as the frozen pair reader;
there is no additional client credit premise.

The initialized writer and final observer composition are deliberately left to
the integrating root. The public adapter is an interface to that composition,
not a claim that the complete writer/observer is checked here.

## Validation and replay

Run `python3 -B source/tools/research/model_ordinary_shared_xor/develop.py` for a
fresh private pretrial freeze and full replay. For the committed final snapshot,
use the command in `results.json`. Raw compiler, assumption, proof-term and
recursive kernel outputs stay in private `/tmp` trials. Compact result and
attempt inventories retain their hashes; unsuccessful trials are development
history only. Every trial uses normal coqc, then complete declaration and
assumption inventories and recursive coqchk without admissions or exclusions.
No shared library or baseline is rebuilt. Tools, sources, objects, loadpaths,
receipts and GMP environment are pinned and guarded through the final checks.
Historical paths inside imported receipts are identity labels, not additional
checkout access permissions.

## Limits

Partial-correctness MODEL facts only. Ordinary accesses have two runtime phases,
with intermediate state changes; there is no unrestricted concurrent SC/Na
behavioral-equivalence claim. The imported isolated-step and framed same-cell
facts do not establish native correspondence. No new axiom, primitive, trusted
summary, unsafe mode, original application premise, or source-domain restriction
is introduced. No native Rust/MIR/provenance/ancestor-history/retag/C/F result is
claimed. Native correspondences remain 0, minimal/phase simulations 44+3 open,
writer effects 101 open, publication false and goal incomplete. Canonical6/0 and
HMAC1/1 are inherited frozen results, not rerun. No subdelegation or independent
review was performed.
