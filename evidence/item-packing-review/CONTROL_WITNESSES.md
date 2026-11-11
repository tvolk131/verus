# Fresh semantic controls and conservative omissions

All finite-width witnesses below apply unchanged to W=8,16,32,64. Applications use the actual bool conversion helper in the crate, normal source checks, --no-cheating and --compile. Rejections count only when verifier JSON reports full-scope proof errors and stderr locates a postcondition or shift-safety failure, never a parser/type/setup error.

* False zero-shift relation `r != x`: x=0 gives r=0.
* Weakened low range low<8 instead of low<4, low projection r&3==low: low=4,high=0 yields0!=4.
* False contiguous-field strict bound r<3 for (x&12)>>2: x=12 yields3.
* False flag relation to !b: low=0,b=false yields no flag bit, contrary to !b=true.
* Shift-safety x<<W: x=1 has an invalid shift amount. This is an ordinary safety obligation, not a semantic postcondition counterexample and not a frontend rejection.
* Signed x<4 without nonnegativity, masked result==x: x=-1 yields3!=-1.
* Noncontiguous mask20, claimed extracted <=3: x=16 yields4.
* Reversed mask12&x, false strict extracted <3: x=12 yields3.
* Nested constant (3<<2)|16, claimed extracted <=3: x=16 yields4.
* XOR overlap, low<8,high<2, claimed r&7==low: low=4,high=1 yields0. This combines overlap with the false low-projection contract. It does not claim XOR is wrong for disjoint fields: on correctly disjoint ranges XOR and OR can coincide.

Full frozen Item mutations are separate: inverted original returned relation, removed original constructor tag bound, changed original permission shift by one, and inverted protected before its genuine conversion. No authored view or contract is repaired to accommodate a mutation. Removing the tag bound permits a value above the original low-tag mask and causes truncation; shifting the permission field changes its projection; inverting the bool changes the actual returned protector bit.

Two mathematically valid unchanged probes initially failed78/2 at W8: bare `x<<(W-1)` equal to x*2^(W-1), x<2; and `(12&x)>>2 <=3`. These failures are retained as conservative omissions. The first lacks the OR expression used as the split theorem's trigger; the second reverses the mask operand, which the recognizer does not commute. Supported zero-shift, nested-mask reader and three-field raw-upper probes are run separately with their original assertions. No assertion is weakened, and these valid failures are never labeled false controls.
