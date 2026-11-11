# Bounded checked packing deduplication preregistration

Base efd9fb5680ece75f871058677afe10c9ce7e64eb, production21eeabbb.
This commit precedes engine trials. BASELINE.json freezes authenticated completed
bundle bytes and every isolated BV query, per ordinary function. The completed
independent run measured full146/0 (116 BV +3 Eq), packet32/0 (29 BV).
Its AIR inventory predicts 14 redundant full queries and one packet query.
It removes comments/whitespace and occurrence-only qid/skolemid labels solely
for this preliminary comparison. It preserves BV sorts, operators, literals,
variable names and full logical structure. This AIR audit is a prediction, not
the production equality rule: typed SST distinctions must be checked separately.
Completed baseline wall costs, including existing identity hashing, are recorded;
they are historical debug runs, not controlled speedup measurements.

Typed interface: `closed_key(&Exp) -> Option<ClosedKey>`; a key contains the type
of EVERY expression, integer value, exact BinaryOp (including width and flags),
exact VarIdent (including disambiguator), ordered quantifier/binder types/names,
ordered triggers and body. Only Bool, mathematical Int and finite U8/16/32/64,
integer constants, bound variables, generated binary operators and closed
forall/None are supported. Unknown nodes, unbound/mistyped variables, duplicate
binders and exhausted bounded traversal return None. No alpha equivalence,
arithmetic normalization, premise extraction or hashes-as-equality. Only spans
are omitted: they are source diagnostics; these generated formulas have no
semantic span payload. Existing types_equal ignores ImplPaths, so is unsuitable
as a general identity; sst_equal constructs SMT equality, not structural equality.

Selection: a fresh per-prepare HashSet<ClosedKey> retains the FIRST original Arc;
every unsupported proposal remains. Every retained proposal emits the original
empty-requires AssertBitVector immediately followed by Assume of the SAME Arc.
Dropping a duplicate drops both statements. No other assertion changes. No
cross-function/crate/state/cache reuse; all resource bounds and global version
schema39/policy71/cache74 remain. Policy72/cache75 reserved for parent only.

Structural gates: independently constructed identical formula; literal, operator,
expression type, binder name/type/order/identity, trigger change; free variable and
unsupported shape; true+distinct false proposal stays a proof obligation; statement
order/Arc identity; set lifetime local to emission. No unchecked assumptions.

Trials: offline locked Rust1.99.1 jobs2 rustup-first PATH; independently build
compiler/macros/vstd using only copied authenticated completed Cargo inputs. Freeze
final tool bytes before any applications/imports. Same own bundle for every
positive provider/client. Full exact generator/fidelity (8 bodies, pinned SHA),
C/W with actual Eq miss/hit and fresh3 precision checks; unchanged natural packet,
separate visibility-only public adapter/client preserving3 fields; bool2/0,
ordinary1/0,false0/1; full observers5/0,false protector4/1 named goal, constructor
scope control; false isolated BV/stale export and erased-only failure controls.
Run requested focused suites and149 VIR plus matching vstd2250/0 once, broadening
only for evidence. Exact inventory comparisons determine counts; never weaken
controls or use count thresholds as proof. Compare an own same-source tool-pair
with dedup disabled, with independently matching providers/vstd for that compiler;
record retained/removed formula and AIR inventories, typed/source/native domain
identities, wall cost and all excluded trials. Timing includes key construction
and compiler identity hashing; qualify noise and don't claim release performance.

Raw sources/AIR/binaries/logs stay private. Compact hashes/receipts/controllers
only are committed here. Python uses exception gates and -B, valid under -O.
No parent/sibling live state inspection, no security/race experiments, no push/PR.
Native correspondences0;44minimal+3phase/101writer obligations OPEN;
canonical6/0 and frozen HMAC1/1 UNPROVED inherited; necessary shared assistance
on both original full targets unconfirmed. Whole-goal publication=false;
goal_complete=false. Parent independent review required before acceptance.
