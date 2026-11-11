# Source review of 297332c2 at prepared 913044c4

F1: `import_export.rs:361` now independently collects the explicit PathBuf then
the Cargo PathBuf unless equal. NotFound remains success. Every iteration runs
after any earlier error; the first error is retained and returned after the loop.
Writing still uses the unchanged explicit-first `export_path`. Both inferred and
local-only invalidators share this implementation, including early Cargo cleanup.
Path identity is lexical PathBuf equality; canonicalization is unnecessary for
ordinary selected file paths, and absent files are harmless. No race, substitution,
permission attack, or security claim is part of this review. A selected directory
can remain because remove_file fails, but verification fails and other selected
files are still attempted. The filesystem unit exercises explicit/Cargo A/B and
B/A, equal A/A, and first-path directory failure with second-path removal.

F2: all prior local `types_equal` uses become `same_serial`, including u64 model
selection, parameter/return, body block, local pattern and binding, temporary and
initializer, read-place/output, tail, and every typed contract component. The
existing helper uses serde_json value equality with fail-closed serialization.
TypX and ImplPath derive Serialize without witness-field skip annotations.
Serialization recursively includes datatype/dyn witnesses, type arguments,
reference/allocator decorations and resolved FnDef information. The supported
MIR type alphabet is concrete u64/NonZero<u64>/Ordering and shared refs, so this
stricter comparison does not require arbitrary semantic type normalization.
Global `vir::ast_util::types_equal` remains unchanged for its ordinary uses.

The checker still re-inspects complete real MIR: exact semantic core Ord/get/u64
instances/signatures, whole CFG/block/local counts, Copy/no-drop, live/dead/read/
borrow/transmute sequence, primitive call operands/successor/unwind and mentions.
Both getter formal/result maps are injective, type checked and compared with the
whole original getter's uninterpreted transmute. No transmute representation axiom
is introduced. Fresh copies come from genuine shared inputs, which may alias.

Actual Function readback covers all FunctionX fields: module/name, visibility and
body visibility, opaque/static/exec/proxy, type parameters/bounds, both returns,
parameter modes/mutability, empty requires and other non-supported contract fields,
exact one ensures, attributes/item kind, dependency list and hidden fields. Nine
binders are injective; six declarations and all initializer places, patterns,
copy/mode/mutability fields, read IDs, read kinds/types and local names are checked.
The only permitted calls are exact typed get/get/u64cmp targets, including resolved
implementation/type/impl-path/attributes, with no post_args or call bodies. Whole
block counts exclude additional effects or unvisited statements. The full numeric
ensures parses typed spec_get values in original order, result identity, unsigned
Lt/Gt tests, and all Less/Greater/Equal alternatives. No domain precondition exists.

Registration validates before BodyBackedForeign insertion. Query::_compute_checked_function
revalidates after external-trait demotion, normal WF, autospec and modes. Existing
read-kind finalization records separate mode/erasure information; it does not
legitimately remove this concrete type's dependency witnesses. AutospecUsage::Final
and fixed resolved targets avoid later autospec replacement in this helper. The
fresh supported-source controls and actual second-boundary diagnostic test check
this operationally. Future transformations changing witnesses must either retain
this identity or establish a newly reviewed correspondence; rejecting such an
unsupported transformation is fail-closed, not permission to omit witnesses.

Source redirection still binds the actual HIR caller/expression, adjusted shared
arguments and concrete instance, and preserves ordinary argument translation and
effects. Model binding still uses imported opaque get/spec_get/View and actual u64
OrdSpecImpl. The existing u64 model's numeric cmp_spec and true obeys_cmp_spec are
inherited from unchanged vstd. The constructed get/get/u64cmp Function has a real
ordinary body, not external_body or an unchecked std summary. Existing normal
helper-query success gates remain in verifier.rs before successful completion;
query logs are observations, not substitute proof objects. No StdModel admission,
NonZero Ord representation law, carrier restriction, primitiveCmp encoding,
assumption, vstd/core/model edit or imported ordering certificate was added.

The local-only barrier still refuses any selected explicit or Cargo output before
pruning. Existing Bool certificates and goals-off imports use their ordinary
pipeline. Wrapper BorTag Ord and ordering imports remain unproved. This review
checks the bounded compiler-TCB correspondence; it does not prove the verifier
kernel, native behavior, or a broader wrapper/lookup/borrow-frame theorem.
