# Accepted-engine native proof boundary

Status: full native resize NOT PROVED. This is a frontend blocker, not a
counterexample in redb and not a failed bitmap invariant.

The fresh accepted-engine diagnostic retains this native body:

```rust
fn required_words(elements: u32) -> usize {
    let words = elements.div_ceil(64);
    words as usize
}
```

The compiler rejects the resolved core unsigned `div_ceil` method before
backend proof checking. An explicit checked contract does not fix translation.
The entire nine-body resize chain therefore remains unavailable on the accepted
engine. Replacing the method, assuming its result, or presenting a rewritten
ceil helper as native proof is not an acceptable workaround.

## Fresh useful result

Seven unchanged native bodies (`bits_in_range`, `data_index_of`, `select_mask`,
`len`, `get`, `set`, `clear`) plus an authored read/update composition check.
Native tuple translation is supported with explicit checked contracts; tuple
inference is not required for this reference-proof route. The access composition
reports 11 successful verification results, covering eight executable bodies
(seven native plus the composition client) and one proof body. The checked
bit-vector lemma, contracts, snapshots, and quantified proof are author work,
not new Infer automation. False postconditions fail in the backend.

## Trust decision before any implementation

The older strict83 freeze explicitly authorized a new unsigned `div_ceil`
standard-library correspondence. Its experimental implementation was never
accepted into the current engine. Reusing that old authorization would still
add a trusted correspondence relative to publication86; it is not already
provided by the current vstd, nor a proof of Rust core's method implementation.

This renewed combined proposal excludes new trusted assumptions. Consequently
no application `assume_specification`, external-body wrapper, or disguised
compiler primitive model is installed here. The original strict83 authorization
and its trust expansion remain visible in the historical scope; they are not
silently grandfathered into this renewed campaign.

To proceed past this blocker, either explicitly authorize that precise std
correspondence exception again, or establish checked translation of the actual
core method body. A quotient/remainder theorem alone does not establish native
method correspondence. Any exception needs a separately recorded decision,
per-width/type/identity audit, nonzero-divisor safety, extremes and negative
controls, and new independent review. Even then, full resize induction and
storage/padding composition would remain to be proved.

No Infer production change is justified under the presently frozen trust
boundary. Consolidation and seven-body component evidence remain useful, but
neither discharges the full native proof or closes the old goal.
