# Checked executable byte XOR in the existing lrust model

The candidate implements the full byte domain, with no change to the language.
`byte_xor` is a closed function containing a finite 256-by-256 table of integer
literals. `byte_indices = seq 0 256`; `xor_row` and `xor_rows` construct its AST
at Coq definition time. `Z.lxor` labels those fixed literals; it is not a runtime
operator, call summary, semantic primitive or assumed fact. Runtime execution
uses only the existing function application and two `Case` reductions.
The symbolic substitution and list lookup laws avoid enumerating 65,536 proofs.

`byte_xor_pure` proves `PureExec True 3 (xor_call a b) #(Z.lxor a b)` for
**every** `0 <= a,b < 256`. Pure execution includes step availability and
universal determinism: no observations, identical physical state and no forks.
`byte_xor_normalizes` additionally gives exactly three actual primitive steps
for every initial model state, each with that same state, empty observations
and empty spawned-thread list. `byte_xor_range` proves `0 <= Z.lxor a b < 256`.
These results have no Iris heap, credit or state-validity premise. There is no
nondeterminism, recursion through the function itself, divergence trick or
restriction to selected byte values.

`wp_byte_xor` discharges the exact
`ModelBufferTransform.transform.binary_operation_spec Z.lxor byte_xor` interface
with no client resource premise. The pure-step lifting's generated continuation
credits are discarded; they are not an input requirement.

`wp_xor_memory` instantiates the completed physical loop at
`min(length xs,length ys)` for arbitrary byte lists. Its result has the following
independent list semantics:

```
xor_prefix xs ys = map (fun (a,b) => Z.lxor a b) (zip xs ys)
xor_result xs ys = xor_prefix xs ys ++ drop (min(length xs,length ys)) xs
```

The postcondition includes that exact result, original pad length, semantic XOR
prefix, original untouched suffix, byte closure, and the **unchanged complete
key vector**. `wp_xor_memory_128` specializes only the pad length, leaving key
length universally quantified; empty keys and keys longer than 128 are included.
The separate `xor_result_*` lemmas prove the semantic decomposition.

These are internal physical-resource WP theorems: pad/key vector permissions
and one store credit per iteration are explicit. No public raw-heap adequacy
theorem is introduced, and no resource requirement is appended to the original
Rust application. The operation's finite normalization and the memory loop's WP
are separate theorems. This work does not assert operational adequacy or a
termination theorem for the whole physical loop from an arbitrary raw heap.

## Validation and trust

The typed pretrial milestone is commit `e45d4671`. Trial015 first completed normal
compilation, all 26 closed assumption reports and full recursive kernel checking.
Trial016 repeats the same candidate with the final controller: it also checks
the generated proof-term export module and freezes Coq runtime stub libraries.
Exact results and hashes are in `STATUS.json`, `EVIDENCE.json`, and
`TRUST_INVENTORY.json`; the final pretrial binding is `trial017-freeze.json`.
Trial017 repeats trial016 after whitespace-only controller cleanup; the candidate
is byte-for-byte identical across all three successful trials.

The controller requires both the pinned baseline-ready record and its successful
durable build receipt, including successful version/build/kernel command stages.
The foundation is unmodified VerusBelt `115a618dd35ff8337112cf2081a7f5391d3973d5`
with Coq 8.20.0. It authenticates all 523 retained foundation source inputs, the
284-object inventory, loadpath file and pinned prover/checker/runner/opam tools.
It authenticates the completed trial007 loop receipt against the committed
ModelBufferTransform STATUS hash **in the retained guard snapshot**, then checks
every retained loop source/object hash against that authenticated receipt.
The loop source in this checkout must match its accepted private compiled source.

The source/controller/shared supports and parser/acceptance code are frozen.
The external-set hash binds Coq standard compiled libraries, Coq core plugins,
OCaml runtime stubs and all retained GMP package/manifest/installed files, plus
the completed loop artifacts. Before/after guards cover these and the baseline.
Inherited Coq/OCaml/load-library overrides are removed; the pinned runner adds
its authenticated GMP environment and opam switch. All Coq compilation happens
in NEW private `/tmp/verus-byte-xor-trialNNN/proofs` directories, so `.vo`, `.glob`,
`.aux`, `.vos`, `.vok`, and tactic caches stay outside the repository. No dependency
installation, engine change or shared-baseline write was performed.

There are 26 candidate constants, no new constructors, no Program declarations
or obligations. The recursive checker verifies the actual candidate declaration
list against the frozen inventory. Every candidate's `Print Assumptions` is
closed. Two generated inventory/export modules contain 52 trivial marker proofs
each, all recursively checked in the final run. Fully explicit `Print` output is
hashed separately for each candidate constant; compiled object and source hashes
are retained too. Recursive dependency checking reports only the foundation's
inherited functional extensionality and `Eq_rect_eq` axioms; no new assumption,
unsafe cast, unchecked computation or trusted contract is added.

## Development trial classification

All attempts have a pretrial source/control freeze and a private durable receipt.
Failed candidate copies and logs remain under their corresponding trial directory.
These are proof-development failures, **not semantic negatives or native evidence**.

| Trials | Classification |
| --- | --- |
| 001 | Invalid tactic syntax (`simpl only`). |
| 002 | String-list versus binder-list typing. |
| 003 | Explicitly interrupted runaway map-composition rewrite in the wrong direction. |
| 004 | Substitution rewrite target had already been reduced. |
| 005 | Unresolved placeholder in an explicit `change` expression. |
| 006 | Premature PureExec instance inference. |
| 007 | Ambiguous language versus context-language `prim_step` projection. |
| 008 | Missing explicit XOR argument before arithmetic side-condition solving. |
| 009 | Wrong list-length lemma name. |
| 010 | Missing import for pure WP lifting. |
| 011 | Generic `expr` name shadowing caused by import order. |
| 012 | Explicit versus named implicit arguments of the lifting lemma. |
| 013 | Integer scope on the proofmode later count. |
| 014 | Used ordinary `apply` where the proofmode needed `iApply`. |
| 015 | Normal compile, 26 closed assumption reports, explicit proof-term export and recursive kernel success. |
| 016 | Same-candidate replay, with export-helper kernel coverage and runtime-stub freeze. |
| 017 | Final same-candidate replay after whitespace-only controller cleanup. |

The actual language term did not gain an instruction or assumed summary during
these repairs. The normalization theorem was added at trial007 and checked at
trial008; the generic full-byte operation statement and loop specification were
not weakened.

## Remaining native boundary

This is executable **MODEL** code. It does not show that the original Rust body,
its MIR XOR instruction, actual entry/source state, provenance, ancestor loans,
phase interpretation, C/F observations, iterator operations or loop correspond
to this table or to the physical model loop. It does not alter any original Rust
body, domain, predicates, safety/debug checks, contracts or application requires.
Actual native correspondences remain **0**, the 44 minimal operations plus three
phase obligations and all 101 effects remain open, the full frozen HMAC
prefix-and-suffix result remains **1/1**, and canonical verification remains
**6/0**. Publication is false and the parent goal stays active. Model progress
is not completion of that goal.
