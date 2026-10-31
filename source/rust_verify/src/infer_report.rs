//! Read-only presentation of generation observations and candidate-bound evidence.
//! This JSON is a report, not an importable certificate or an acceptance input.
use super::Verifier;
use serde_json::{Value, json};
use std::collections::HashMap;
use vir::ast::{ExprX, Fun, FunX, Function, Mode};
use vir::ast_util::{fun_as_friendly_rust_name as name, typ_to_diagnostic_str};
use vir::infer_proof::Outcome;

/// Presentation-only snapshot of compiled local Rust functions. It is never
/// serialized into a proof interface or consulted by verification/publication.
#[derive(Clone)]
pub(super) struct NativeFunctionScope {
    name: Option<Fun>,
    rust_name: String,
    source: String,
    selected: bool,
    external_body: bool,
    external_specification: bool,
    derived: bool,
}

#[allow(deprecated)]
pub(super) fn collect_native_scope<'tcx>(
    tcx: rustc_middle::ty::TyCtxt<'tcx>,
    items: &crate::external::CrateItems,
    verus_items: &crate::verus_items::VerusItems,
) -> Vec<NativeFunctionScope> {
    let mut functions = vec![];
    for item in &items.items {
        let id = item.id.owner_id().to_def_id();
        if !matches!(
            tcx.def_kind(id),
            rustc_hir::def::DefKind::Fn | rustc_hir::def::DefKind::AssocFn
        ) {
            continue;
        }
        let (selected, external_body, external_specification) = match item.verif {
            crate::external::VerifOrExternal::VerusAware {
                external_body,
                external_fn_specification,
                ..
            } => (true, external_body, external_fn_specification),
            crate::external::VerifOrExternal::External { .. } => (false, false, false),
        };
        functions.push(NativeFunctionScope {
            name: crate::rust_to_vir_base::def_id_to_vir_path_option(tcx, Some(verus_items), id)
                .map(|path| std::sync::Arc::new(FunX { path })),
            rust_name: tcx.def_path_str(id),
            source: tcx.sess.source_map().span_to_diagnostic_string(tcx.def_span(id)),
            selected,
            external_body,
            external_specification,
            derived: crate::automatic_derive::is_automatically_derived(tcx.get_all_attrs(id))
                || crate::automatic_derive::is_automatically_derived(
                    tcx.get_all_attrs(tcx.parent(id)),
                ),
        });
    }
    functions.sort_by(|a, b| a.rust_name.cmp(&b.rust_name));
    functions
}

fn check(outcome: Option<Outcome>, required: bool, missing_reason: &str) -> Value {
    let (status, reason) = match outcome {
        Some(Outcome::Valid) => ("proved", None),
        Some(Outcome::Invalid) => ("not_proved", Some("solver_did_not_establish_obligation")),
        Some(Outcome::Incomplete) => ("incomplete", Some("resource_limit")),
        Some(Outcome::Skipped) => ("not_established", Some("query_skipped")),
        None => ("not_established", Some(missing_reason)),
    };
    json!({"status": status, "required": required, "reason": reason})
}

impl Verifier {
    fn verification_scope_json(
        &self,
        run_succeeded: bool,
        available: &HashMap<Fun, Function>,
    ) -> Value {
        // A success under filters or --no-verify is not evidence for the entire
        // selected scope. Keep reporting conservative on all failure paths.
        let full_success = run_succeeded
            && !self.args.no_verify
            && !self.args.verify_root
            && self.args.verify_function.is_none()
            && self.args.verify_module.is_empty()
            && self.args.verify_only_module.is_empty();
        let mut checked_exec = 0usize;
        let mut checked_proof = 0usize;
        let proxies: HashMap<_, _> = available
            .values()
            .filter_map(|f| f.x.proxy.as_ref().map(|p| (p.x.clone(), f)))
            .collect();
        let functions: Vec<_> = self.native_function_scope.iter().flatten().map(|native| {
            let f = native.name.as_ref().and_then(|name| available.get(name)
                .or_else(||proxies.get(&name.path).copied()));
            let status = if !native.selected { "outside_selected_scope" }
            else if native.external_specification { "external_specification" }
            else if native.external_body || f.is_some_and(|f|f.x.attrs.is_external_body) {
                "unverified_body_with_contract"
            } else if let Some(f) = f {
                if f.x.mode == Mode::Spec { "specification_definition_or_declaration" }
                else if f.x.body.is_none() { "interface_without_checked_body" }
                else if !full_success { "selected_body_not_certified_by_successful_full_run" }
                else if f.x.mode == Mode::Proof { checked_proof += 1; "checked_proof_body" }
                else { checked_exec += 1; "checked_exec_body" }
            } else { "selected_item_without_translated_body" };
            json!({
                "function": f.map(|f|name(&f.x.name)).unwrap_or_else(||native.rust_name.clone()),
                "rust_function":native.rust_name,"source":native.source,
                "automatically_derived":native.derived,"selected":native.selected,
                "body_status":status,
                "inference_candidate":f.is_some_and(|f|f.x.attrs.infer),
                "ordinary_contract":f.is_some_and(|f|!f.x.attrs.infer && (!f.x.require.is_empty() || !f.x.ensure.0.is_empty())),
            })
        }).collect();
        json!({
            "coverage":"selected local Verus items; not all Rust code",
            "native_collection_completed":self.native_function_scope.is_some(),
            "complete_selected_scope_verified":full_success,
            "checked_exec_bodies":checked_exec,"checked_proof_bodies":checked_proof,
            "native_functions":functions,
            "limitations":["Only compiled cfg-enabled named functions/methods are listed; closures belong to their enclosing body and generated methods may appear.",
                "Outside scope includes unannotated, explicitly excluded and automatically ignored implementations.",
                "A checked body is not a claim that unverified callers satisfy its preconditions.",
                "This report does not certify a requested application property or replace an authored scope/trust inventory."],
        })
    }

    pub fn inference_report_json(&self, run_succeeded: bool, total_micros: u128) -> Value {
        let generation = &self.inference_generation;
        let available: HashMap<Fun, Function> = self
            .vir_crate
            .iter()
            .flat_map(|k| k.functions.iter())
            .chain(generation.candidates.iter().map(|c| &c.function))
            .map(|f| (f.x.name.clone(), f.clone()))
            .collect();
        let mut candidates: Vec<_> = generation.candidates.iter().collect();
        candidates.sort_by_key(|c| name(&c.function.x.name));
        let mut required_failure = false;
        let mut required_incomplete = false;
        let functions: Vec<_> = candidates.iter().map(|candidate| {
            let f = &candidate.function;
            let allow_approx = f.x.attrs.infer_allow_approx;
            let evidence = self.inference_proofs.outcomes(f);
            let observed = evidence.as_ref().ok().copied().unwrap_or([None; 3]);
            let accepted = self.inference_proofs.status(f)
                .is_ok_and(|p| p.satisfies(allow_approx));
            for (i, o) in observed.iter().enumerate() {
                if i == 0 || !allow_approx {
                    required_failure |= matches!(o, Some(Outcome::Invalid));
                    required_incomplete |= matches!(o, Some(Outcome::Incomplete));
                }
            }
            let unavailable = if evidence.is_err() {
                "no_matching_valid_evidence"
            } else {
                "check_not_run"
            };
            let approximate_dependency = vir::infer::uses_approximate_callee(f, &available);
            let state_summary = vir::infer::is_state_summary(f);
            let unsupported_state_precision = state_summary && !vir::infer::is_record_summary(f);
            let maximality_missing = if unsupported_state_precision { "state_precision_not_supported" }
            else if approximate_dependency {
                "sound_mode_dependency"
            } else { unavailable };
            let exactness_missing = if unsupported_state_precision { "state_precision_not_supported" }
            else if allow_approx && matches!(
                f.x.ensure.0.first().map(|e| &e.x),
                Some(ExprX::Const(vir::ast::Constant::Bool(true)))
            ) { "unconstrained_result" } else { unavailable };
            let dependencies: Vec<_> = candidate.referenced_functions.iter()
                .filter(|callee| available.get(*callee).is_some_and(|f| f.x.attrs.infer))
                .map(|callee| {
                let f = &available[callee];
                json!({"function": name(callee), "candidate_id": f.x.attrs.infer_candidate_id,
                    "mode": if f.x.attrs.infer_allow_approx { "sound" } else { "exact" }})
            }).collect();
            let (requires, ensures) = vir::infer::format_contract_parts(f);
            let solver_micros: u128 = self.func_times.values()
                .filter_map(|times| times.get(&f.x.name))
                .map(|stats| stats.smt_time.as_micros()).sum();
            json!({
                "function": name(&f.x.name),
                "source": f.span.as_string,
                "candidate_id": f.x.attrs.infer_candidate_id,
                "mode": if allow_approx { "sound" } else { "exact" },
                "summary_kind": if vir::infer::is_owned_summary(f) { "owned_value" } else if vir::infer::is_record_summary(f) { "record_state" } else if candidate.record_template { "record_view" } else if state_summary { "collection_state" } else { "value" },
                "parameters": f.x.params.iter().map(|p| json!({
                    "name": p.x.name.0, "type": typ_to_diagnostic_str(&p.x.typ)
                })).collect::<Vec<_>>(),
                "result_type": typ_to_diagnostic_str(&f.x.inner_ret.x.typ),
                "contract": {"requires": requires, "ensures": ensures},
                "loop_proofs": vir::infer::format_loop_proofs(f),
                "proofs": {
                    "soundness": check(observed[0], true, unavailable),
                    "maximality": check(observed[1], !allow_approx, maximality_missing),
                    "exactness": check(observed[2], !allow_approx, exactness_missing),
                },
                "required_checks_passed": accepted,
                "available_to_same_crate_callers": self.inference_ready && accepted,
                "exported": self.inference_exported && accepted && f.x.visibility.restricted_to.is_none(),
                "dependencies": dependencies,
                "metrics": {
                    "cache": if candidate.cache_hit { "hit" } else { "miss" },
                    "cache_policy": if vir::infer::is_record_summary(f) || candidate.record_template { "regenerated_state" } else if vir::infer::is_collection_summary(f) { "regenerated_template" } else { "candidate_cache" },
                    "generation_micros": candidate.elapsed_micros,
                    "solver_micros": solver_micros,
                    "requires_nodes": candidate.requires_nodes,
                    "ensures_nodes": candidate.ensures_nodes,
                },
            })
        }).collect();
        let status = if let Some(failure) = &generation.failure {
            // These are generation diagnostic categories, never proof outcomes.
            if failure.starts_with("Verus Infer unsupported:") {
                "unsupported"
            } else if failure.starts_with("Verus Infer incomplete:") {
                "analysis_incomplete"
            } else {
                "generation_failed"
            }
        } else if !run_succeeded {
            if required_incomplete {
                "required_proof_incomplete"
            } else if required_failure {
                "required_proof_failed"
            } else if self
                .func_fails
                .iter()
                .any(|f| !available.get(f).is_some_and(|f| f.x.attrs.infer))
            {
                "caller_proof_failed"
            } else {
                "verification_failed"
            }
        } else if generation.requested == 0 {
            "no_inference"
        } else {
            "verified"
        };
        let mut failed_functions: Vec<_> = self.func_fails.iter().map(name).collect();
        failed_functions.sort();
        json!({
            "schema_version": 1,
            "verification_scope":self.verification_scope_json(run_succeeded, &available),
            "status": status,
            "crate_success": run_succeeded,
            "requested_functions": generation.requested,
            "generated_functions": functions.len(),
            "generation_failure": generation.failure,
            "range_loop_assistance": generation.range_loops.iter().map(|(f,s)|json!({"function":name(f),"source":s.as_string,"checked_by":"ordinary invariant, safety and original iterator termination obligations"})).collect::<Vec<_>>(),
            "pointwise_assistance": generation.pointwise_facts.iter().map(|(f,n)|json!({"function":name(f),"facts":n,"checked_by":"ordinary checked assertions; no assumed invariant or clone law"})).collect::<Vec<_>>(),
            "mutable_selector_assistance": generation.mutable_selectors.iter().map(|(f,n)|json!({"function":name(f),"postconditions":n,"checked_by":"ordinary original-body verification; source contracts preserved"})).collect::<Vec<_>>(),
            "growth_loop_assistance": generation.growth_loops.iter().map(|(f,s)|json!({"function":name(f),"source":s.as_string,"checked_by":"ordinary invariant, safety and termination obligations"})).collect::<Vec<_>>(),
            "local_fold_assistance": generation.local_folds.iter().map(|(f,s)|json!({"function":name(f),"source":s.as_string,"checked_by":"ordinary full-body invariant, safety and original iterator termination obligations"})).collect::<Vec<_>>(),
            "factory_assistance": generation.factory_callbacks.iter().map(|(function,count)|json!({
                "function":name(function),"strategy":"total_factory_callbacks","callbacks":count,
            })).collect::<Vec<_>>(),
            "error_callback_assistance": generation.error_callbacks.iter().map(|(function,count)|json!({
                "function":name(function),"strategy":"constructor_and_callee_postconditions","callbacks":count,
                "checked_by":"ordinary closure and original-body verification",
            })).collect::<Vec<_>>(),
            "array_callback_assistance": generation.array_callbacks.iter().map(|(function,count)|json!({
                "function":name(function),"callbacks":count,"checked_by":"ordinary unchanged slice conversion callback; generated domain and element equations",
            })).collect::<Vec<_>>(),
            "type_invariant_entry_assistance": generation.type_invariant_entries.iter().map(|(function,count)|json!({
                "function":name(function),"parameters":count,"checked_by":"existing use_type_invariant operator; ordinary construction and restoration checks retained",
            })).collect::<Vec<_>>(),
            "goal_assistance": generation.goal_lemmas.iter().map(|(function, lemmas)| json!({
                "function": name(function), "strategy": "counting_broadcast",
                "lemmas": lemmas.iter().map(name).collect::<Vec<_>>(),
            })).chain(generation.view_bound_lemmas.iter().map(|(function, lemmas)| json!({
                "function": name(function), "strategy": "sequence_view_bound",
                "lemmas": lemmas.iter().map(name).collect::<Vec<_>>(),
            }))).chain(generation.fold_lemmas.iter().map(|(function, lemmas)| json!({
                "function": name(function), "strategy": "sequence_fold_observation",
                "lemmas": lemmas.iter().map(name).collect::<Vec<_>>(),
            }))).collect::<Vec<_>>(),
            "failed_functions": failed_functions,
            "functions": functions,
            "arithmetic": "mathematical integers; explicit machine casts; rust_div/rust_rem truncate toward zero",
            "trust": "ordinary Verus backend and imported specifications; report is not a proof certificate",
            "imported_inference_dependencies": self.imported_inference_dependencies,
            "trusted_models": self.inference_trusted_models.iter().map(|(f, fingerprint)| {
                json!({"function": name(f), "fingerprint": fingerprint})
            }).collect::<Vec<_>>(),
            "interface_written": self.inference_exported,
            "metrics": {"generation_micros": generation.elapsed_micros, "total_micros": total_micros},
        })
    }
}
