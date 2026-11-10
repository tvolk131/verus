# Borrow-semantics inspection: reuse rules, not an assumed source certificate

The inspected artifacts supply concrete retag operations and useful lemmas.
They do not supply the missing Rust1.99.1 constructor/source certificate or
original-reference C/F interpretation. No dependency was built or imported;
the existing checked foundation and native/publication status are unchanged.
[Pins](pins.json) bind two source checkouts and four document snapshots.

## Stacked Borrows

The [published artifact](https://plv.mpi-sws.org/rustbelt/stacked-borrows/)
links the inspected repository at commit
`d1471b10fe4cb4f506371b2e3acdd927d0a0c076`. Its
[retag definition](https://gitlab.mpi-sws.org/FP/stacked-borrows/-/blob/d1471b10fe4cb4f506371b2e3acdd927d0a0c076/theories/lang/bor_semantics.v)
computes grant/protector checks, while the expression-level retag leaves the
data heap unchanged. The two layers must both hold: heap stuttering alone
does not establish that the grant succeeds. The retag lemmas include preservation
of active protectors, locality and properties of resulting stack heads.

This version uses untagged raw pointers and old Coq/Iris/Paco/Equations
dependencies. It is not a direct model of the pinned modern compiler. The
artifact's optimization simulation also needs source definedness; it is not
the missing proof that this source body is safe. Porting an individual pure
lemma requires a justified source-operation relation and compatible tag rules.

## Tree Borrows / Simuliris

The [authors' source link](https://plf.inf.ethz.ch/research/pldi25-tree-borrows.html)
leads to Simuliris. Inspected commit:
`e3abfcdc301f92250d4227fc03c00463bf57b982`.
`tree_borrows/bor_semantics.v` separates child creation, freshness, call validity
and the read performed by retagging. These are explicit operational premises.

The decisive limitation is in
[simulation/slsls.v](https://gitlab.mpi-sws.org/iris/simuliris/-/blob/e3abfcdc301f92250d4227fc03c00463bf57b982/theories/simulation/slsls.v):
the simulation receives source-pool safety. The
[retag lifting rule](https://gitlab.mpi-sws.org/iris/simuliris/-/blob/e3abfcdc301f92250d4227fc03c00463bf57b982/theories/tree_borrows/step_laws/steps_retag.v)
uses that premise to obtain source-step conditions. Reusing this rule cannot
establish our missing source safety without circularity. This is a scope
distinction, not a defect in optimization verification. The checkout also
requires newer Rocq/Iris dependencies than the pinned proof environment.

## Other source scope

Louwrink's [SBSL thesis](https://eprints.illc.uva.nl/id/eprint/1790/)
describes a sound program logic for a simplified Stacked Borrows language.
Section4.7 reports experiments connecting lifetime reborrowing and retagging;
it leaves further RustBelt integration open. The author's fork link from the
public GitLab listing redirects to sign-in. Its code was not inspected, and no
authentication was attempted. This is not evidence of a ready Rust1.99.1
MIR/source-to-Verus C/F theorem.

The [official Miri documentation](https://github.com/rust-lang/miri) and
[UCG Tree Borrows notes](https://github.com/rust-lang/unsafe-code-guidelines/blob/master/wip/tree-borrows.md)
also qualify their experimental/non-normative scope. The retained moving-head
document snapshots are inspection evidence, not dependencies bound to the
frozen compiler. A successful Miri experiment would not replace universal
source safety or observation proofs.

## Consequence for this branch

Continue the concrete storage-prefix work and implement the missing bounded
reference transitions explicitly. A future reborrow layer may reuse protector,
locality and grant-progress arguments, after matching the actual source
semantics and incoming-reference validity. Do not initialize an empty history,
assume a grant succeeds, use source safety as the conclusion's premise, or
identify ghost unpacked pointers with runtime addresses.

The constructor and exit records remain conditional interfaces. This inspection
does not create a source instance, validate all lawful incoming histories,
prove pending-reference observation or enable a native frame rule. It avoids
an unsupported replacement of the safety obligation with an optimizer theorem
and avoids an unnecessary second proof-toolchain build.

No Coq proof claim is made here. Native0, 44 minimal+3 phase/101 effects open,
full HMAC1/1 unproved, publication=false and goal incomplete remain unchanged.
