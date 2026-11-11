# Genuine NonZero BorTag ordering: frozen baseline, goal open

Base 131ffc3575ab340a4b74292c0d51cd4ee7f6adee; accepted engine 5d79f711.
No engine or global version changes. Schema39 / policy72 / cache75 remain.

The exact pinned Miri declaration has SHA256
0107890f5ffee6b94a4179c0351a4ff121e0e61b9886067e94370a65391e9abd.
It retains all seven original derives and its private genuine NonZero<u64> field.
No input restriction, carrier substitution or runtime adapter is introduced.
The full domain extends through u64::MAX.

Fresh accepted-runtime baseline is in BASELINE.json, with private originals in
/tmp/nonzero-ordering-baseline-qoptrgkf. Original compiler-expanded bodies and
native MIR are hashed, including generated PartialOrd and Ord. PartialOrd calls
Some(BorTag::cmp(self, other)); Ord calls NonZero<u64>::cmp on the two fields.
The actual six comparison call clients fail 2/6; this is not checked derived
ordering. The two verified queries do not certify those ignored runtime derives.

Direct genuine NonZero partial-order observations (< <= > >= and partial_cmp)
verify 5/0 through the inherited standard model. Direct NonZero::cmp fails 0/1
at carrier_ord::total's numeric-order postcondition. The existing standard
NonZero model has no OrdSpecImpl. Authored numeric contracts with derive markers
select original ordering methods but stop at the typed single-scalar-wrapper
gate before proof; macro requests stop earlier at the PartialEq-only gate.
These profile failures are distinct from the direct cmp semantic proof failure.
No failed case publishes its selected export or compiled library.

Existing genuine Eq verifies 6/0 cold and warm for both BorTag and unrelated
CounterKey. Each has one exact helper with requires=true and fresh proved
soundness/maximality/exactness, with real miss then hit. Only Clone, Eq and the
client are checked executable bodies. Other derives remain excluded. This is
regression evidence, not ordering or native success.

Next gate: inspect the actual core Ord semantic instance and whole instantiated
MIR, then specify the ordinary typed body/view correspondence needed to prove
numeric cmp. The existing bool-to-u64 foreign recognizer only covers a single
bool cast; it must not silently admit carrier projections, Ord calls or enums.
No comparison law, view equivalence or new standard specification is assumed.

Raw generated source, MIR, logs and binaries stay private. run.py reproduces a
fresh baseline with rustup-first PATH and Python -B; audit.py validates it under
-O -B using exceptions, not assertions. The first audit expected a retained
derive attribute in compiler-expanded output; rustc correctly consumes it.
That audit-only mismatch was corrected without rerunning proofs. A diagnostic
MIR inspector build failure is retained at /tmp/nonzero-ordering-mir-vcfpd78u;
its initial filename also shadowed Python's inspect during exception reporting.
Neither is semantic evidence.

Existing parent 717 compiler /149 VIR /6 lib /2250 vstd and feature receipts
are inherited acceptance only; not newly rerun here. No production change has
been made that would justify repeating those suites.

TCB remains the accepted compiler/frontend, Rust toolchain, solver and inherited
vstd models. Authored views/contracts supply intended numeric semantics, not a
proof of source correspondence. Closed value remains self.0@. Existing exported
Eq/value compatibility requires ONE checked broadcast lemma and a client import;
this work neither reclassifies it as proof-free nor adds premises.

Ordering, full find_granting and native correspondence remain open. Native=0;
full HMAC=1/1 UNPROVED; all 44 minimal +3 phase /101 writer effects OPEN.
Whole-goal publication=false, goal_complete=false. Independent parent acceptance
is still required.
