# Source audit of the two corrections

Reviewed frozen source at 9761b5f55405fdcb2901f6599b52973cb1e31c43.
Production changes are only ef662ee3 and c4df45e8. BINDING.json contains the
exact source bytes/identities. This audit does not inspect D2.

## D1 metadata traversal

`source/vir/src/internal_repr.rs:240` still validates actual descriptor shapes
before its owned-family guard. No internal descriptor means immediate return;
Rev-only crates still return before the new general traversal. The two factories,
closed u64 instantiation check, identities and shape/privacy checks are unchanged.
The correction replaces just the Function/datatype loops with `visit_krate`,
then explicitly visits both opaque fields. The opaque destructuring has no `..`.

Audited every KrateX field (`ast.rs:2013`) against `AstVisitor::visit_krate`
(`ast_visitor.rs:1432`) and the container definitions at `ast.rs:1877–1993`:

| Inventory | Type-bearing fields and traversal |
| --- | --- |
| functions | FunctionKind trait args; typ_bounds; params; outer_ret; inner_ret; require/ensure/returns/decrease/decrease_when/fndef_axioms/mask_spec/atomic_update/unwind_spec/body expressions; both return_opaque_type_eqs types. All visited at ast_visitor.rs:1134,1173. typ_params are identifiers, not Typ. FunctionAttrs has no Typ field. |
| datatypes | typ_bounds; all variant field types; sized_constraint. All visited at :1303. typ_params are (Ident, AcceptRecursiveType), not instantiated types. Internal descriptor has identities/paths/visibility, no embedded Typ. |
| traits | typ_bounds and assoc_typs_bounds, both visited at :1342. assoc_typs are identifiers, methods are Fun paths, typ_params are identifier/positivity pairs. dyn_compatible and external_trait_extension contain strings/paths, no Typ. |
| trait_impls | typ_bounds and trait_typ_args, both visited at :1403. typ_params are identifiers. trait_typ_arg_impls are ImplPaths. |
| assoc_type_impls | typ_bounds, trait_typ_args and typ RHS, visited at :1375. typ_params are identifiers. impl_paths are ImplPaths. |
| opaque_types | typ_params really are Typs; typ_bounds are GenericBounds. General visitor clones this inventory unchanged. New explicit loop at internal_repr.rs:269 visits BOTH. def_fun and name are paths. |
| reveal_groups | Fun/path/visibility/member paths only (ast.rs:1810). No Typ hidden in the skipped group inventory. |
| modules | path and optional Fun reveals only. |
| external_fns/external_types/path_as_rust_names/arch | Fun/path/string/architecture information only. |

ImplPath (`ast.rs:988`) has only TraitImplPath(Path) and FnDefImplPath(Fun),
so cloning dictionary paths does not hide type arguments. GenericBoundX visits
Trait arguments, TypEquality arguments AND RHS, and BOTH ConstTyp operands
(`ast_visitor.rs:1055`). Typ recursion covers opaque args, function/closure
inputs/results, datatypes/dyn/primitive arguments, allocator decoration types,
boxed/projection/pointee/mut-ref types (`:973`). WalkTypVisitorEnv recursively
checks those types before the callback (`:1512`). Expression, place, pattern,
call target, binder and statement visitors visit their types and subterms.
`validate_type_use` itself recursively validates actual descriptor-bearing
Datatype arguments. No further omission was found for finalized VIR in scope.
This is source inspection, not a new exhaustive synthetic runtime fixture.

One deliberate visitor exception is transient ExprX::Header: its inner header
is not traversed (`ast_visitor.rs:501`); ExprX::AirStmt holds already lowered
AIR rather than VIR nominal types. Header is explicitly temporary during
rust_to_vir (`ast.rs:1201`), extracted by rust_to_vir_func.rs:2433, and rejected
by normal WF (`well_formed.rs:773`). This acceptance does not certify arbitrary
malformed/pre-extraction AST as valid backend input. No skipped valid Krate
metadata type field was identified.

The new regression (`infer_checks/tests/internal_representation.rs:486`)
constructs all nine rows below for C<u32> and C<u64>. It calls the direct guard
AND metadata_backend_accepts for every row/width, without short-circuiting.
`metadata_backend_accepts` constructs GlobalCtx from the descriptor-only base,
then calls actual Ctx::new on the metadata-bearing crate (:451). Actual
`Ctx::new` invokes the guard as its FIRST statement (`context.rs:813`).

1. Trait implementation arguments.
2. Trait implementation bounds.
3. Associated implementation RHS.
4. Associated implementation arguments.
5. Associated implementation bounds.
6. Trait declaration bounds.
7. Trait associated-type bounds.
8. Opaque parameters.
9. Opaque bounds.

Expected bad u32 rejection and good u64 acceptance are asserted independently
at both entries: 18 fixture crates, 36 results. Accepting the good variant at
Ctx entry helps rule out an unrelated blanket error as the negative oracle.
Fixtures intentionally reference synthetic metadata and build GlobalCtx from
base; they do NOT establish full source validity, whole crate WF, a source
admission bypass, method authority, or any native theorem. Runtime evidence is
replay of this unchanged compiled test in the full 172-test VIR executable.
No independently compiled additional synthetic cases were introduced.

## F1 crate owner coverage

`checked_owned_cell_admission.rs:639–694` still walks the item AND all ancestors,
reads both external/verifier flags, and rejects the two skip markers, four
frontend directives and all external/trusted exclusions BEFORE evaluating the
new crate-owner condition. Only positive `verify || verus_macro || sets_mode`
accumulation is gated by `d != hir::CRATE_OWNER_ID.def_id` (:682). The initial
`checked = default_verify`, caller's actual `args.no_external_by_default`
(:797,:849), and every real item/module context remain intact.

This matches `external.rs:get_crate_items:138`: the default starts from the
actual no-external-by-default option; root contents are visited directly via
visit_mod, rather than sending the crate owner through attribute-processing
visit_general. Real item/module attrs establish Verify in visit_general (:239).
The fix does not claim this attribute approximation equals the eventual
translated/checked Function inventory. That reconciliation remains a future
activation gate, including selected outputs and generated/frontend special items.

No default-off production path changes: verifier.rs:3524 invokes the whole
experimental check only under the explicit flag, before imports/pruning/cache
reuse; normal checks continue otherwise. checked_owned_cell::check_request
still unconditionally returns the later inactive error after typed admission.
The correction creates no descriptor, installer, field permission, source name
privilege, native call rewrite, helper fact or erasure exemption. Crate markers
confer no authority. The test additions contain exactly 28 compiler calls:
three root positive flags * two modes * (plain + item + module + true default)
= 24, plus two root exclusions * two modes = 4.

Independent CLI fixtures in cli_witnesses.py intentionally exercise ordinary
feature-off checking as the positive coverage oracle, and specific feature-on
refusal as the inactive admission oracle. A feature-on later refusal is never
called a passed body proof. Valid false ordinary contracts must fail on
postconditions, not frontend syntax or WF failures. Root exclusions are checked
conservatively even though ordinary translation ignores root opt-in attributes.

## Preserved boundaries

The two diffs change no descriptor shape/identity/factory, wire format, source
privacy, global version/default or trusted law. DatatypeTransparency::Internal
remains #[serde(skip)] (`ast.rs:1856`). Loaded-native replay tests both wire
refusals, eight corrupted shapes, six bad argument shapes, source invisibility,
58 MIR/signature refusals (3*(16+2)+4) and 45 correspondence controls
(3*(12+1+2)). It still asserts THREE full normal Function WF opaque-field
failures (`checked_owned_cell_lowering.rs:669–694`). These are actual refusals,
not native body proofs or full current/future/contract correspondence.
The eight old internal-representation/Rev regression tests all execute as part
of the full VIR inventory, alongside the new metadata test. The original F1
two-marker/four-directive groups execute unchanged in the seven admission tests.
