# Checked abstract countermodel; native bridge remains unproved

No full native HMAC pad-XOR theorem was achieved. No production change, trusted
model extension, assumed native law, executable rewrite, push, or PR is included.
The frozen original loop and property/domain were not edited. The earlier
concrete-model theorem (11/0) is inherited research, not a native refinement and
not a new result of this workstream.

## New checked result

`candidates/observation-countermodel-v2/probe.rs` proves
`checked_trace_counterexample` for **every** `before: Seq<u8>` of length 128 and
every key sequence shorter than 128, including empty keys. It checks 7 queries
with zero errors under ordinary `--no-cheating` checking. Six named local proof
bodies are checked; there are no executable bodies or inferred helpers.
The input restriction here identifies counterexamples to logical entailment;
it does not replace or restrict the native application's full-domain theorem.

The constructed future has the correct XOR prefix and changes exactly the
first unused byte by XOR 1. Referent records have an explicit index identity,
initial value, and eventual value. The checked conjunction establishes:

- every initial/eventual slice-to-referent equation at all 128 positions;
- the exact XOR relation on every consumed byte;
- arbitrary-length successful Zip steps with logical tail identity preserved,
  exact remaining length decrease, and the returned head's identity/value;
- the ordinary sequence snapshot/history prefix equations;
- a terminal None step with unchanged empty truncated pair sequence;
- resolution of the terminal opaque Zip object, projection to its first inner
  object, and current/eventual equality of all remaining inner referents;
- failure of the **unchanged full** `pad_xor` predicate.

The last two resolution conditions deliberately model even the **proposed**
IterMut remaining-frame and Zip resolution-projection laws. They hold at the
terminal state because its inner observation is empty. Prior running states
are unresolved. The generic next contract allows this abstract hidden-inner
transition: it constrains only the zipped sequence. Thus those two proposed
laws alone would still lack a conservation/transition obligation for the
unreturned tail. This is new evidence beyond the previously failed native
resolution-law probes.

This is a checked countermodel of **the explicit value-level observation
equations in this fixture**, not an executable iterator or a model of every
axiom in vstd/Rust. `resolved` is an interpreted Boolean token, not an assertion
of native `has_resolved`. It demonstrates insufficiency of the enumerated
equations, not a metatheoretic independence proof for the complete verifier.
In particular it is not a Rust counterexample: the native implementation does
not forget the tail. No production theorem imports these abstract definitions.

`observation-countermodel-v1` checks the component lemmas (6/0); v2 adds the
single conjunctive trace theorem (7/0). `false-abstract-frame-v1` keeps the same
healthy component proofs and asks for the false full abstract frame; it reports
6 verified/1 error, only the intended postcondition failure. Every candidate
was separately hash-frozen before its first compiler invocation.

## Actual pinned Rust route

`source-map.json` binds Rust 1.99.1 core source and the frontend/resolver/model
inputs. `additional-source-evidence.json` binds pointer-permission and frontend
scope evidence. The source root is recorded in the map. The requested
`source/rust_verify/src/resolve_axioms.rs` is located in this checkout at
`source/vir/src/resolve_axioms.rs`.

The real path is `slice/mod.rs:1059` (`iter_mut`) to
`slice/iter.rs:219` (`IterMut::new`), `iter/traits/iterator.rs:629` (`zip`),
`slice/iter.rs:19` (immutable slice IntoIterator), then
`iter/adapters/zip.rs:293,299` (specialized Zip new/next). Both slice iterators
implement `TrustedRandomAccess` at `slice/iter.rs:2987,2997`. The specialized
next increments Zip's own index and uses `slice/iter/macros.rs:413`'s unchecked
getters. Those getters do not advance the inner pointer. The terminal branch
returns None without touching either inner iterator. An attempted safe model
that simply calls both inner next functions is not the exact selected native
implementation. Source inspection identifies obligations; it does not discharge
them or prove correspondence.

Native IterMut at `slice/iter.rs:192` holds `NonNull<T>`, a pointer/length,
and `PhantomData<&mut T>`, not the owned slice field of the checked safe example.
All fields of native Zip are private (`iter/adapters/zip.rs:13`). Merely letting
the resolver inspect private pointer fields would still not produce referent
permissions. `resolve_axioms.rs:72` treats raw pointer resolution as trivial;
`resolve_datatype_axiom:219` propagates only through actual visible non-spec
fields, skips destructor types, and supplies no pointer-to-owned-referent law.

## Checked mechanism probes and frontend evidence

`tracked-observation-v1` tries to initialize a tracked witness from native
`it.remaining()`. Full-checking flags are enabled, but the frontend rejects it
with `expression has mode spec, expected mode proof`. A duplicable observation
is not a linear ownership witness. `transparent-native-v1` asks the existing
external-type proxy machinery to expose the real IterMut fields. It is rejected
with `private fields not supported for transparent datatypes`. Both are
**frontend rejections**, not SMT semantic negatives or checked native bodies.

`rust_to_vir_adts.rs:655` maps an external-body native proxy to a datatype with
no fields and transparency Never. At 683 the transparent path reads the actual
native ADT, then rejects private fields. There is no declaration syntax here
that checks a substitute struct's ownership against the native representation.
`rust_to_vir_func.rs:1000,1050,1142` resolves a specified native function and
checks signatures/predicates, not its body. HIR translation iterates local
crate items (`rust_to_vir.rs:613`); adding rust-src does not import foreign
native bodies for proof. The separately checked example mentioned in
`std_specs/slice.rs:496` is a maintenance convention, not a checked relation to
the opaque native type.

The existing raw-pointer model has tracked `PointsTo` permissions. Its
`ptr_mut_ref` wrapper (`vstd/raw_ptr.rs:636`) requires a matching initialized
permission and connects borrowed current/eventual values to that permission.
The native getter supplies no such ghost argument; native `iter_mut` currently
returns no permission carrier. Inventing it from the pointer/observation would
be the missing ownership bridge, not a consequence of that existing wrapper.
No pointer axiom or external wrapper was added or changed.

## Smallest remaining model/trust change

At the logical API level the missing information is conservation of **all
unissued mutable referents**, not just the truncated Zip remaining sequence.
A u8/128-specific certificate is sufficient, but must connect the original
array borrow to linear permissions and account for specialized indexed yields.
Its constructor must retain all 128 slots, successful steps transfer exactly
one slot to the returned reference, None must preserve the retained tail, and
scope exit must discharge every unissued permission unchanged. The inherited
initial/eventual observations must be proved to match those permissions.
The empty-key case forces this ownership/frame requirement even without a
single successful write. Arbitrary key length must remain supported.

There are two distinct next-work choices, neither applied here:

1. Implement checked native-body/certificate translation for this pinned
   monomorphized pointer/specialization path, with source identity, ownership,
   prophecy, transition, drop and observational-correspondence checks. This
   extends the compiler's trusted translation and needs an explicit soundness
   argument and false-certificate controls. Existing structural resolution
   alone cannot check it. The original source must remain the executable input.
2. Extend the inherited trusted standard model with a retained-tail ownership
   and resolution contract, justified externally against Rust. Even if the safe
   concrete model proves such a contract, attaching it to native IterMut/Zip
   without the checked correspondence above is a **trust change**, not a
   checked bridge or completed goal under the present boundary.

A native-name/hash allowlist emitting an axiom, adding only the two previously
requested resolution laws, changing the writer, or adding an application
precondition does not solve the stated task. This report makes no claim that
all possible checked refinements are impossible; the current frontend supplies
no checked correspondence route, and none was implemented in this workstream.

## Reproduction and audit

The copied accepted compiler/macros/vstd/Z3 bundle is private and immutable;
`bundle-identity.json` records matching hashes and distinct inodes. It is runtime
0c00b6ec, Rust 1.99.1, Z3 4.16.0 (schema38/policy67/cache70 inherited, unchanged).
No compiler/vstd rebuild is needed because no production source changed.

```sh
python3 -B -O source/tools/research/native_iterator_refinement/replay.py \
  --verus /matching/private/bundle/verus --solver /matching/private/bundle/z3 \
  --output /private/fresh-native-refinement \
  --case observation-countermodel-v1 --case observation-countermodel-v2 \
  --case false-abstract-frame-v1 --case tracked-observation-v1 \
  --case transparent-native-v1
```

Repeat with a fresh output and `--goals` to check the same outcomes with
`--infer-goals`. These are ordinary checked math/probe results, not assistance
or cache acceptance. Each case explicitly expects zero inferred helper
identities and still invokes the shared mandatory-obligation checks. Ordinary
positive/semantic-negative cases use `validate_goal_run` with named local
exec/proof inventories, full scope and failure-kind checks. Frontend results
use a separate marked validator; safety/resource/mixed failures cannot count
as semantic evidence. None occurred in the accepted trials. There are no
poisoned-helper claims for an empty helper set.

`results.json` records accepted replay receipts, code/tool hashes, and excluded
setup evidence. Raw commands, JSON, stdout/stderr and binaries remain private
under `/tmp/native-iterator-refinement-20261109-fa70fa8e/`. Controllers use the
shared `infer_experiment`/`infer_eval_support` guards and durable failure
receipts, including under Python -O. The initial freeze-script import generated
two local Python cache files outside this workstream; those generated files
were removed and both controllers now disable bytecode generation. No final
file outside the owned research directory changed.
