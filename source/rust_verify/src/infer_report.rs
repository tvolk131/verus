//! Read-only presentation of generation observations and candidate-bound evidence.
//! This JSON is a report, not an importable certificate or an acceptance input.
use super::Verifier;
use serde_json::{Value, json};
use std::collections::HashMap;
use vir::ast::{ExprX, Fun, Function};
use vir::ast_util::{fun_as_friendly_rust_name as name, typ_to_diagnostic_str};
use vir::infer_proof::Outcome;

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
            "status": status,
            "crate_success": run_succeeded,
            "requested_functions": generation.requested,
            "generated_functions": functions.len(),
            "generation_failure": generation.failure,
            "range_loop_assistance": generation.range_loops.iter().map(|(f,s)|json!({"function":name(f),"source":s.as_string,"checked_by":"ordinary invariant, safety and original iterator termination obligations"})).collect::<Vec<_>>(),
            "pointwise_assistance": generation.pointwise_facts.iter().map(|(f,n)|json!({"function":name(f),"facts":n,"checked_by":"ordinary checked assertions; no assumed invariant or clone law"})).collect::<Vec<_>>(),
            "growth_loop_assistance": generation.growth_loops.iter().map(|(f,s)|json!({"function":name(f),"source":s.as_string,"checked_by":"ordinary invariant, safety and termination obligations"})).collect::<Vec<_>>(),
            "factory_assistance": generation.factory_callbacks.iter().map(|(function,count)|json!({
                "function":name(function),"strategy":"total_factory_callbacks","callbacks":count,
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
