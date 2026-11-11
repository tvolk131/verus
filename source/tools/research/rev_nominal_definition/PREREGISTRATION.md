# Experimental Rev nominal definition: preregistration

This is additional semantic-definition research on parent fdd0b0c6. Accepted
production remains 1171bdf7, engine d3b2aa6e, schema39/policy74/cache77.
`INTERFACE.json` is the typed obligation list, not an installed interface.
`publication=false`, `goal_complete=false`, native correspondences=0.

The proposed logical carrier is the actual nominal core Rev ADT, with one private
internal field of its actual instantiated I. C and F are datatype constructor and
selector, not runtime proxy code. For ALL valid values of each supported type:

```
alpha_Rev(Rev {iter:x}) = C(alpha_I(x))
F(C(v)) = v
C(F(r)) = r
existing rev_iter<I>(r) := F(r)
```

The last line is a new compiler definition, not a new axiom or constructor ensure.
All inherited generic spec calls must refer to it. The equality/eta/injectivity,
typed boxing, generic arguments, old/final field reborrows and prophecy operations
must use the same carrier for every value, including parameters, post-mutation
values and aliases. Private Rust fields stay private to source. Logical encoding
visibility must be distinct from source visibility and checked independently.
No production edit precedes this freeze/commit.

The executable target is narrowly read-only Enumerate<slice::Iter<'a,T>>, with
T=u64 and the genuine Item of the authenticated fixture. No IterMut, Zip, custom
iterators, tail frames or native safety premises. Universally quantify over every
valid shared &'a Vec<T>, including empty and arbitrary length, without application
lawfulness, tag-wf or length premises. The unchanged baseline runtime is:

```rust
pub fn baseline<'a, T>(borrows: &'a Vec<T>) {
    for (_, _) in borrows.iter().enumerate().rev() {}
}
```

Fresh concrete actual Item MIR is mandatory; generic/u64 diagnostic MIR is not
its authority. Type/lifetime/trait/erasure checks remain ordinary Rust checks.
Source receiver evaluation/move occurs once. Optimized MIR Copy conveys no source
Copy permission. into_inner is diagnostic only and absent from runtime clients.

The eight inherited probes report available1/0, lawfulness0/1, inner0/1 and literal
empty forloop1/1, each off/on. These are support failures, not semantic controls.
A finite observational SAT counterexample was qualified, not all-vstd unprovability.

Proceed in order: authenticate/copy inputs privately; extract own actual MIR;
check an explicit parametric compatibility model and inspect the native binding
obligation. Only if that obligation can be supplied without an assumption proceed
to production hooks, an internal carrier/definition, independent whole actual
Function/descriptor validators before/after normal WF/autospec/modes, fresh full
NORMAL compiler/macros/vstd and useful loop tests. Check no-helper/no-inference
all-selected output refusal before pruning, semantic fingerprint mismatches,
structural corruptions, healthy false clients, and cold/warm/control/export states.
All failure gates and exact nonclaims are frozen in INTERFACE.json. A transparent
model alone is diagnostic; it must never be installed as a native Rev fact.

If the first source-coherence obligation is unavailable, stop and record its
exact sequent and evidence. Do not add a trusted rule to make that step pass.
Keep hooks/implementation/evidence in separate commits when reached; a stopped
research-only implementation must label its checks as diagnostic. No broad
suite, network, push, PR, publication, worktree creation or delegation.
