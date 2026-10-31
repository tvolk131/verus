# Hex-conservative native forward iterator acceptance

Accepted: the real internal string constructor used by decode_to_vec,
ChunksExact-backed HexDigitsIter and generic HexToBytesIter next/nth compose into
a complete forward string-iterator proof. The [charter](charter.md) fixes the
semantics; [scope and trust](assumptions.md) lists authored requirements and
standard-library assumptions.

Native implementation: `cc0bf36056a190bd7da4cbf226e778580e8bb472`.
Engine implementation: `1e0cf81d20d0dcec0f137edc9b95a8d9c2d98d60`.
Native v1 baseline: `5ccfbad247d5a332408929173e4072fbf06d8bd1`.
Engine baseline: `b42a50e3e96e32b9c76b25dd51caaacd5a120c37`.

Six new connected application goals prove construction on arbitrary Rust strings,
selected output arithmetic, original-input invalid byte payload/location and high
byte priority, suffix state and remaining pair count for arbitrary reachable
forward histories, skipping, exhaustion and relevant saturation boundaries.
The retained v1 pair goal/specification and its exact inferred helper remain
mandatory. No ASCII restriction is imposed. Length and position count UTF-8
bytes. An out-of-range nth consumes the source but leaves the cursor unchanged;
skipped invalid pairs produce no error. Empty input and usize::MAX skips are
covered. String-reachable selected byte positions cannot overflow; arbitrary
cursor states preserve the actual saturating formula.

- Fresh restored proof: **30 verified results, zero errors**; **25 named executable
  bodies**, the real static initializer, and auxiliary/certificate obligations.
- Application goals: **six new, one retained**. Vstd replay: **2,215 tooling
  results**, not application theorems. The pair helper retains all three mandatory
  soundness/maximality/exactness certificates.
- **18 deliberate semantic mutations rejected**, with healthy pair certificates
  and no published interfaces. These are injected controls, not upstream defects.
- Without-assistance, no-verify and filtered controls reject; restored source
  hashes equal the fresh-run inputs. Publication remains fail-closed.
- **291 inference regressions pass**, including unrelated cross-crate triplet
  chunks and conditional/saturating tuple error callbacks. Const/static 57 pass
  (two pre-existing ignored); unwind 17 and type-invariant 90 pass.
- Native tests pass with Rust **1.99.1**: default/all/no-default/alloc-only.
  Baseline and restored MSRV **1.74.0** tests pass. Added state-machine tests cover
  mixed case, invalid/skipped pairs, UTF-8, odd/empty input and saturation.
- Parser audit preserves all **218 original signatures and executable bodies**.
  Production algorithms, original table/initializer, derives and configuration
  are intact. Changes are selection/required contracts, an impl split, closed
  observations, checked structural invariant, semantic goals/tests and docs.
- Immutable hex v1/Aleph v3/v4 evidence, archived inputs, temporary accepted Aleph
  source and fixed engine artifacts match prior hashes.

The runtime HexDigits size_hint is checked exact. Generic HexToBytes runtime
size_hint/ExactSizeIterator::len exactness is not proved here; logical remaining
counts are proved. The public HexSliceToBytesIter wrapper is natively tested but
not selected. Backward/mixed iteration, reads, formatting, encoding, fixed arrays
and public unsafe drains remain outside this acceptance. Fixed-array 2*N still
requires its overflow domain. This is a finite native iterator result, not a
public decode_to_vec theorem or blanket certification.

New engine support proposes slice-conversion callback contracts and uses the
existing checked type-invariant entry operator. Original closure bodies and
borrow restoration verify ordinarily. Conditional tuple error constructors use
checked generated relations. New trusted standard-library models explicitly
cover core ChunksExact and copied slice-to-array conversion, plus a str length
bound. Concrete standard no-unwind refinements do not permit unchecked
first-party trait laws. [The assumption inventory](assumptions.md) details this
trust boundary. No native assume, external body, source proof block, authored
assertion/loop invariant/manual trigger or blanket exclusion was added.

Replay from the engine root with the matching policy **52**, cache **58** and
Rust 1.99.1 compiler/vstd:

```sh
python3 source/tools/check_hex_iterator.py \
  --native-root /Users/tommyvolk/Documents/GitHub/hex-conservative \
  --output /private/tmp/hex-iterator-replay-new
```

The controller temporarily mutates native source and restores it in finally.
It requires native write permission, uses fresh per-run export/cache paths,
--no-cheating and explicit offline/locked toolchains. The source archive is the
accepted committed native checkout plus its ignored matching Cargo.lock; the
patch is against upstream 129db6a. Source/artifact hashes bind the build despite
its retained b42a50e.dirty banner. `acceptance-final` is the accepting series;
the earlier `acceptance` stopped at an ambiguous controller match and restored
source. Development and both series' raw logs are archived, with a hash index.

See [closure](closure-summary.json), [controls](controls-summary.json),
[source audit](source-audit.json), [engine receipt](engine-source-receipt.json),
[preservation](preservation.json) and [raw index](raw-evidence-index.json).
