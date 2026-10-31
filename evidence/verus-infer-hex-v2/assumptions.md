# Scope and trust inventory

The independent requirements are the retained v1 ASCII numeric-range digit/pair
specification and the v2 charter's original UTF-8 byte sequence, selected-pair,
error location/priority and suffix-state semantics. Application contracts are
requirements checked against original bodies. They are not assumed summaries.
Six new application goals connect actual constructor/chunks/decoder calls and an
arbitrary reachable forward-history step. The seventh retained goal is v1's pair
goal. Native execution has no added ASCII, nonempty or successful-input domain.
An even string with non-ASCII bytes constructs successfully and then reports its
first selected invalid byte. Odd **byte** length is rejected first.

The added HexDigits type invariant states that every underlying remaining chunk
has length two and its count fits usize. Its constructor, clone and mutable-borrow
restoration check the invariant. Entry assistance uses Verus's existing checked
use_type_invariant operation. Closed observations expose fields, exact pair
projections, cursor, pending decoded results and reachable forward state. They
are authored logical definitions, not first-party assumed specifications. Native
source contains no added assume, proof block, assertion, loop invariant, proof
function, manual trigger, external body or blanket derive/function exclusion.
Original derives remain; standard derive verification limitations/warnings remain
visible in raw logs and the native selected-scope report.

General callback assistance proposes domain/content/error-field requirements;
ordinary unchanged closure bodies and callers check them. These are not exact
inference certificates. The sole exact inference helper remains hex_chars_to_byte:
body soundness, maximality and exactness are independently mandatory. Its real
static table and initializer retain v1's checked requirements. Publication of all
new assistance profiles requires complete selected-crate verification. Failed,
filtered and no-verify providers export no interface. --no-cheating is required.

New trusted **Rust standard-library** boundaries are explicit in
source/vstd/std_specs/chunks_exact.rs: opaque ChunksExact and TryFromSliceError;
logical chunks/width observations; six assume-specifications for chunks_exact,
next, nth, size_hint, Clone, and copied slice-to-array TryFrom. They model complete
chunks, remainder discard, exact forward suffix transitions, concrete exact hint,
clone preservation and conversion success/content. Concrete ChunksExact next/nth
are no_unwind; generic trait dispatch uses only a resolved core/alloc assume-spec
proxy's explicit refinement. First-party trait impl unwind refinements remain
rejected. This does not verify Rust core's implementation. Standard model triggers
are inventoried as tooling, not authored native proof hints.

The existing Iterator prophetic interface gains a conditional nth law. Every
selected first-party next/nth implementation claiming the laws is checked against
it. The generic size_hint model is only typed: HexDigits' actual size_hint is
checked exact, but the decoder's generic runtime size_hint/ExactSizeIterator::len
exactness is **not** claimed by the application goals. No exact hint guarantee is
imported merely from a first-party trait name. vstd str::len additionally exports
the ordinary representability bound on byte length. Existing vstd arithmetic,
string bytes, options/results, trait/call/derive and solver/compiler trust remain.

String reachability gives selected positions below original byte length; therefore
cursor/position saturation is inactive for selected valid string pairs. Generic
arbitrary cursor contracts preserve actual saturation, separately exercised by a
MAX-position application goal and native boundary tests. Out-of-range nth leaves
the cursor unchanged while exhausting the source. Logical arithmetic for n+1
avoids usize overflow at MAX. Skipped invalid pairs do not raise errors.

Unselected scope includes the public HexSliceToBytesIter forwarding wrapper
(native tests cover it in existing tests), backward/mixed iteration, reads,
encoding, formatting, error presentation/offset helpers, fixed arrays and both
unsafe drains/public decode endpoints. The unchanged 2*N fixed-array expression
still needs its overflow domain N<=usize::MAX/2. This finite iterator result is
not certification of those endpoints, the entire crate or arbitrary Rust.
