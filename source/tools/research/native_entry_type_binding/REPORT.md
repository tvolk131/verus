# Concrete byte-reference compiler binding

The additive carrier-layout epoch is recorded separately in
[carrier-results.json](carrier-results.json) and
[CARRIER_SOURCE_CUT.md](CARRIER_SOURCE_CUT.md). Its9 guarded command outcomes
pass: the actual array-reference parameters are Scalar/8-byte/alignment8, and
the key-reference parameter is ScalarPair/16-byte/alignment8. Pointee size and
carrier size are distinct. This extends the earlier type-only checkpoint,
independently accepted by `e03ce8c2`; the additive epoch is independently accepted
by `a4d43ec9` at compiler-layout scope only.
The earlier receipt/results/freezes remain preserved unchanged.

The exact local compiler checks passed. The instantiated type/layout condition
left open by the [entry-source review](../../../../evidence/published-entry-source-review/REPORT.md)
is now supported by direct rustc queries and ordinary trait checking. This is
**compiler evidence only**, not a native theorem or a permission interpretation.

The pinned compiler is Rust1.99.1, commit
`58cf8f9c35b355199727627d07ae34a4e2846d67`. The driver reads both unchanged inputs:
the accepted canonical `candidate-008.rs` through normal macro erasure, and the
already frozen native HMAC writer fixture. It queries their actual parameter
types after analysis, rather than inferring them from source spelling.

| Actual parameter subject | Unpin | UnsafeUnpin | Freeze | Static byte size | Alignment |
| --- | --- | --- | --- | --- | --- |
| Canonical buffer `[u8; U64_LEN]` | true | true | true | 10 | 1 |
| HMAC pad `[u8;128]` | true | true | true | 128 | 1 |
| HMAC key `[u8]` | true | true | true | unsized | 1 |

The target has64-bit pointers. Array references occupy8 bytes; the key slice
reference occupies16. The slice's minimum layout size0 is **not** its runtime
length. No key-length bound or pointer/history/contents premise was introduced.

An ordinary, separately compiled generic function calls a trait-requiring
function for `[u8;N]` for arbitrary const N, without an input trait where-clause.
The unsized `[u8]` requirement also typechecks. The query inventory additionally
covers u8, zero/10/128 arrays, PhantomPinned and UnsafeCell. Three independent
ordinary negative compilations reject PhantomPinned:Unpin,
PhantomPinned:UnsafeUnpin and UnsafeCell<u8>:Freeze. Each exits1 with exactly
E0277 as its error-code class; none is counted as an application semantic test.

## Consequence and remaining proof

In the already pinned Miri Stacked Borrows `NewPermission::from_ref_ty`, these
concrete pointees satisfy the conjunction selecting **Unique / Write / strong
protector** for mutable FnEntry. The TwoPhase branch precedes that conjunction
and instead selects **SharedReadWrite / no access / no protector**. This is a
source inspection combined with compiler answers, not a verified translation of
that Rust function. It supplies no successful grant, live allocation, initial
byte permission, ancestor/protector relation, lawful-input coverage, source
safety, cleanup simulation, argument-copy installation, or original C/F binding.
The historical model and actual Rust/Miri semantic differences remain open.

`RUSTC_BOOTSTRAP=1` exposes the unstable rustc-private and marker-trait APIs and
the candidate's existing macro feature. It does not disable ordinary checking.
No application verifier run, native execution, Coq theorem or production fact
is claimed. Canonical6/0 and full HMAC1/1 remain inherited. All44+3/101 native
obligations remain open; native=0, publication=false, goal_complete=false.

## Receipts and replay

[results.json](results.json) binds the accepted optimized-Python trial003, all9
command outcomes, complete query data, pretrial freeze and excluded setup trials.
The final source/tool/binary guards passed. The336-file input map includes the
compiler, driver/runtime libraries, erasure dependencies, actual source inputs,
controller/support and pinned Miri policy source. These installed Rust artifacts
remain an inherited compiler trust boundary; they were not rebuilt from source.

```sh
python3 -B -O source/tools/research/native_entry_type_binding/check.py \
  --freeze source/tools/research/native_entry_type_binding/trial004-freeze.json \
  --output /tmp/native-entry-type-binding-FRESH
```

That command describes replay in the integration checkout whose absolute paths
the historical freeze records. For an isolated checkout, rebind only repository
paths to identical committed bytes in that checkout and the four erasure-library
paths to the completed byte-identical private bundle recorded in results.json;
freeze that explicit mapping before a new trial. Never read an old live checkout
through a historical path. The copied bundle was identity-checked, not separately
used for another proof run. Fresh independent combined acceptance remains open.

Trial001 stopped before any compiler command because its freeze was still being
written. Trial002 compiled and queried successfully, then its controller rejected
the symbolic constant spelling U64_LEN. Trial003 uses structural TyKind byte-array
identity and the actual layout, retaining the original spelling. Neither earlier
trial contributes accepted semantic evidence; receipts and chronological sources
remain preserved.
