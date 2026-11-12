# Bounded frozen source audit

Root ce0f00897bec7f8ea143cd42409563440d9c8784. No production edits.

* checked_owned_cell_admission.rs:639 walks all local opt_parent links. Both
  earlier skip flags refuse before the new four. checked never short-circuits
  exclusion checks. Fn/Const call checked_item before insertion (792/844).
  Whole-crate definitions include unused nested items. Admitted local bodies are
  traversed; remaining body owners/nested bodies refuse. Header closures require
  actual builtin identity/crate/ABI/shape, and predicates/captures are traversed.
* rust_to_vir.rs:67/70 genuinely omit const_body/reveal_fn items. At94/97/179,
  size_of_global, item_broadcast_use and structural_const_wrapper const branches
  return without ordinary ConstInfo/Function insertion. rust_to_vir_func.rs:
  2008-2036 emits a reveal group and returns before FunctionInfo insertion.
  All four new flags are excluded on items and every local ancestor, including
  conservative malformed placements. Malformed fixtures are not proof evidence.
* rust_to_vir_func.rs:1700-1730 dispatches encoded_const/encoded_static to
  check_item_const_or_static. At3466 it records ConstInfo; finish emits an ordinary
  Function with a body unless external_body. Corrected runtime encoded_const
  controls verify valid ordinary bodies. This is not an omitted-body branch.
* The final VERUS_SPEC Ok(None) at2342 follows FunctionInfo insertion at2321.
  Legitimate specification methods are outside admitted item grammar; a free
  function using that reserved prefix errors at2059. No missing-body claim is
  inferred merely from a caller receiving None.
* has_body/finish_function body removal uses external_body,
  external_fn_specification or no_method_body. External/trusted item/ancestor
  markers refuse. no_method_body is invalid for admitted free functions; its
  call is also outside allowed header/call grammar. Traits, impls/associated
  items, statics, foreign items, ADTs and nonlocal callbacks are excluded.
  GlobalAsm is ignored by ordinary translation but not an admitted Fn/Const;
  type aliases introduce no body. Encoded static still records ConstInfo.
* Closed source/type/effect checks retain source projection, shared/raw escape,
  drop, imported/abstract calls, capture, quantifier and real recursion refusals.
  The unchanged focused suite independently exercises these and option/output
  invalidation guards. Source annotations/macros confer no native authority.
  Private nonserializable AdmittedUnit carries no field/proof authority. Its
  current request consumer unconditionally refuses before imports/pruning/cache.

Additional actual mismatch: checked_item:679 counts crate-owner positive
verify/verus_macro/sets_mode attrs. external.rs:get_crate_items:138 initializes
Verify only for no-external-by-default and visits root contents without processing
crate-owner attrs. A valid root-only verify unit witness is admitted experimentally
but skipped by ordinary verification on old AND fresh combined compilers, both
modes. REPORT.md and RESULTS.json bind the eight runs and smallest proposed fix.

Retain all ancestor exclusions but do not treat crate-owner opt-in attrs as
positive body verification context. Preserve real default-verify, item/module
and nested contexts; add both-mode negatives and positive controls. No production
fix here. Actual admitted-to-translated/checked-Function reconciliation remains a
FUTURE activation gate; an attribute list is not full body proof coverage.
