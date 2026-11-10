# Conditional pointer-distance and size-hint fragment

Delivered only in this new directory. Interface `efea4e77`; implementation
`3713bff6`. Final guarded trial011 succeeded. Native correspondence remains0;
44 minimal operations +3 phases and101 full effects remain open. Publication=false,
goal incomplete. Canonical all-u64/arbitrary initialized exclusive[10] remains6/0;
full frozen bitcoin_hashes0.14 pad-XOR with arbitrary initialized[128], every valid
key length, prefix AND untouched suffix remains1/1. Those inherited outcomes
were not rerun. Same actual checked automation on both full properties, root
integration and fresh independent review remain required.

## Checked results

`candidates/distance.v` imports the completed NativeMirLrust fragment/simulation
library and reuses its window, pointer, bounds and actual Offset laws. It proves
finite64-bit address order, nonwrapping unsigned subtraction, signed byte-distance
bounds, same-origin one-past formation, exact unsigned division by1, and equality
of the result to length. The operational lemmas prove actual existing lrust
head reductions for Offset and scalar subtraction with the same state, no events
and no forked threads. They add no native instruction or Rust intrinsic axiom.

`span` is a CONDITIONAL proof interface. The allocation case permits arbitrary
cursor/containing allocation, including strict interior pad[128]; it creates no
heap authority. The equal-endpoint case needs no allocation for an empty dangling
key. Empty q=p is permitted. No fresh/zero buffer, key<=128, disjoint-allocation,
new application permission, whole-heap initialization or future-defined-run
premise is introduced. Nonempty spans preserve one containing origin and bound
all offsets by its extent and isizeMAX. Formation does not assert one-past
readability; the inherited array-end non-dereference theorem remains available.

`candidates/branches.v` proves the explicit u8 scalar/control chain for BOTH
UbChecks settings, excludes ZST/address-order/size-assert failures, computes the
returned `(n,Some(n))` and `.0` projection, and connects both COMPUTED hints to
specialized Zip index0/len=min(128,n) with unchanged inner endpoint descriptors.
The empty-key and n>128 cases are checked lemmas. No panic implementation is
invented: failure exclusion is conditional on the span and authored fragment.
The original min/Zip cleanup/drop/resume edges remain retained and unsupported
as source simulations, even though the scalar fragment returns normally.

Normally checked rejection lemmas distinguish a wrong numeric distance,
reversed nonempty operands, distinct dangling origins, width32 overflow,
isize overflow and the wrong u8 CTFE branch. These are propositions proved
in Coq, not claimed negative compiler runs. Nine structured producer mutations
also reject wrong operand order, origin field, pointer width, branch target,
CTFE result, address-order comparator, Zip length, min comparator and exceptional
edge. They retain identity/debug strings and bypass identity mismatch gates, so
hash mismatch cannot account for those rejections. The producer is untrusted;
its structural eligibility is not a native source-proof success claim.

## Source and semantic dependencies

[SEMANTICS.md](SEMANTICS.md) gives the exact pinned compiler lowering and core
contracts, actual typed operand chain, and source-to-fragment limitation.
`source-connected.json` retains compact coverage for instances11/15/16/17/
21/22/23/26/27, all their exceptional edges, and checks both size-hint operand
chains. It is derived from the audit's named completed32-instance/408-operation
graph. `pins.json` authenticates its source/extraction receipts and raw input,
actual compiler/core artifacts and installed core sources, plus compiler source
at rustc commit58cf8f9c35b355199727627d07ae34a4e2846d67. The lowering is
ptrtoint(pointer width), unsigned sub with no underflow, exact unsigned divide
by the u8 size1. Inspection justifies the arithmetic chosen; it is not a checked
compiler-preservation theorem. No SAFETY comment is used as a proof premise.

Source-validity -> span is deliberately separate from arithmetic. Root / the
constructor-erasure lane must derive the actual field/cast operand relation for
every lawful original entry/history, including empty dangling keys, and prove
source validity of those steps. It must then relate actual intrinsic, comparisons
and control execution to this checked scalar fragment and compose calls/phases,
exceptions and original-reference observations. No new assumption or opaque
source-erasure/frame/resolve law is exported to fill those gaps. The original
raw initializer/caller9/full ordinary25 remain model prerequisites; this lane
does not promote them to Rust source correspondence. No full-writer theorem or
native-status change follows from these conditional lemmas.

## Validation and replay

Trial `/tmp/native-pointer-distance-efea4e77-trial011/receipt.json` is successful;
its hash, pretrial freeze identity, object/log hashes, exact77-entry per-declaration
assumptions inventory, command exits/times and controls are in `results.json`.
Normal coqc compiled distance/branches and the generated assumptions module;
recursive coqchk checked all three objects and all dependencies without exclusions
or unsafe flags. All77 declarations are closed, recursive axioms=[], and new
Program obligations=0. Constructors and generated elimination principles are
included. Coq compilation took1.73s combined, assumptions4.95s, kernel27.66s.

The controller authenticates baseline-ready.json AND successful durable-build-2
receipt, unmodified VerusBelt115a618d's523 sources/284 objects, tool/loadpaths,
GMP manifest332763a92d52a35f1623c7383970e6cd290b5010c426ca5bb15b21efc7f0bb37,
and the completed native_mir_lrust trial018 source/product receipt. Inputs are
retained under ArtifactGuard through success. Candidate sources are copied to
a fresh private trial, every completed object is guarded through later commands,
and all final sources/products/inputs are checked after recursive kernel checking
before success. Python runs with -B -O; runtime assert is not an acceptance gate.

Ten development trials are retained privately with pretrial freezes, copied
sources, outputs and failure receipts; `development.json` indexes hashes. They
are proof/controller development failures, not semantic negatives. Trial010's
normal compilation and kernel check passed but its controller rejected Coq's
empty-axiom heading. The local controller now recognizes that exact sentinel
and still rejects every unsafe/unrecognized summary; trial011 is the accepted
complete replay. No shared tools or policy were edited.

From the repository root, with a fresh output directory:

```sh
python3 -B -O source/tools/research/native_pointer_distance/check.py \
  --freeze source/tools/research/native_pointer_distance/final-freeze.json \
  --output /tmp/fresh-native-pointer-distance-replay
```

Cherry-pick the interface, implementation and final evidence commits in order.
All edits are confined to this new directory; no shared build/install, compiler,
vstd, global policy, application, parent/sibling checkout or unfinished output
was changed/read. No subdelegates, push, PR, publication or acceptance performed.
