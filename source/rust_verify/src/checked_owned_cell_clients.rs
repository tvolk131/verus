//! Private actual-source frontend probe. No source installer/call substitution.
use super::*;
use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    rc::Rc,
    sync::Arc,
    time::Instant,
};
use vir::ast::*;

#[path = "checked_owned_cell_client_correspondence.rs"]
mod correspondence;
struct Original {
    normal: crate::verifier::VerifierCallbacksEraseMacro,
    frozen: bool,
    raw: bool,
    reached: bool,
}
impl rustc_driver::Callbacks for Original {
    fn config(&mut self, c: &mut rustc_interface::interface::Config) {
        self.normal.config(c); // Exact normal providers; no no-lifetime branch.
    }
    fn after_expansion<'tcx>(
        &mut self,
        _: &rustc_interface::interface::Compiler,
        tcx: TyCtxt<'tcx>,
    ) -> rustc_driver::Compilation {
        let spans = crate::spans::SpanContextX::new(
            tcx,
            tcx.stable_crate_id(rustc_hir::def_id::LOCAL_CRATE),
            tcx.sess.source_map(),
            HashMap::new(),
            None,
        );
        let erased = Rc::new(RefCell::new(crate::context::ErasureInfo {
            local_hir_vir_ids: vec![],
            binder_hir_vir_ids: vec![],
            ctor_hir_vir_ids: vec![],
            resolved_calls: vec![],
            resolved_pats: vec![],
            direct_var_modes: vec![],
            ignored_functions: vec![],
            bodies: vec![],
            shadow_check: vec![],
            extra_erase_ast_ids: vec![],
            local_invariant_bodies: vec![],
        }));
        let mut args = crate::config::ArgsX::new();
        args.no_cheating = true;
        args.no_external_by_default = self.raw;
        args.vstd = crate::config::Vstd::Imported;
        assert!(
            !args.no_lifetime
                && !args.no_trait_conflicts
                && !args.no_erasure_check
                && !args.no_verify
        );
        let args = Arc::new(args);
        let ctxt = crate::context::ContextX::new(
            args.clone(),
            tcx,
            erased.clone(),
            spans,
            Arc::new(crate::verus_items::from_diagnostic_items(tcx)),
            false,
            crate::rust_to_vir_base::mk_crate_id(tcx, rustc_hir::def_id::LOCAL_CRATE),
        );
        let items = crate::external::get_crate_items(&ctxt)
            .unwrap_or_else(|e| panic!("original crate items: {e:?}"));
        crate::erase::setup_verus_aware_ids(&items);
        rustc_hir_analysis_verus::check_crate(tcx);
        assert!(tcx.dcx().has_errors().is_none(), "normal original source type analysis");
        let cell = native_cell(tcx).unwrap();
        let admitted =
            admission::admit(tcx, cell, &args).expect("complete original typed-unit admission");
        let (admitted_functions, owners, headers) = admitted.original_inventory();
        let expected = if self.frozen { 3 } else { 1 };
        assert_eq!(admitted_functions.len(), expected);
        let actual_owners: HashSet<_> = admitted_functions
            .iter()
            .map(|id| id.expect_local())
            .chain(headers.iter().copied())
            .collect();
        assert_eq!(&actual_owners, owners, "every owner is a Function or canonical header closure");
        if self.frozen {
            // Prove freshly against this actual source TyCtxt BEFORE any possible
            // summary consumption. This probe never installs/uses those summaries.
            lowering::test_actual_proposals(tcx);
        }
        let root = std::path::PathBuf::from(std::env::var_os("VERUS_ROOT").unwrap());
        let mut input =
            std::io::BufReader::new(std::fs::File::open(root.join("vstd.vir")).unwrap());
        let imported: Krate =
            bincode_next::serde::decode_from_std_read(&mut input, bincode_next::config::legacy())
                .unwrap();
        let pre = crate::rust_to_vir::crate_to_vir(ctxt, &vec![imported.clone()], &items)
            .unwrap_or_else(|e| panic!("actual original pre-crate refused: {e:?}"));
        let mut translated_ids = HashSet::new();
        for info in &pre.infos {
            if let crate::rust_to_vir_func::FunctionOrConstInfo::Function(f) = &**info {
                translated_ids.insert(f.def_id);
            } else {
                panic!("unreconciled original item kind");
            }
        }
        assert_eq!(
            &translated_ids, admitted_functions,
            "all admitted Functions have original frontend entries, zero helpers included"
        );
        let mut functions = vec![];
        for info in &pre.infos {
            let before = functions.len();
            crate::rust_to_vir_func::finish_item(&pre.ctxt, info, &mut functions).unwrap_or_else(
                |e| panic!("actual original full Function lowering refused: {e:?}"),
            );
            let crate::rust_to_vir_func::FunctionOrConstInfo::Function(source) = &**info else {
                unreachable!()
            };
            assert_eq!(functions.len(), before + 1, "one full Function per actual owner");
            assert_eq!(functions[before].x.name, source.function.x.name);
            correspondence::check(
                tcx,
                cell,
                source.def_id.expect_local(),
                &functions[before],
                admitted.original_headers(),
            )
            .expect("independent whole typed runtime-body/call/place correspondence");
            // Negative actual-Function controls never create or query a twin.
            if self.frozen {
                let f = &functions[before];
                let mut bad_type = f.x.clone();
                Arc::make_mut(&mut Arc::make_mut(&mut bad_type.params)[0]).x.typ =
                    Arc::new(TypX::Bool);
                assert!(
                    correspondence::check(
                        tcx,
                        cell,
                        source.def_id.expect_local(),
                        &f.new_x(bad_type),
                        admitted.original_headers()
                    )
                    .is_err()
                );
                let changed = std::cell::Cell::new(false);
                let wrong = vir::ast_visitor::map_expr_visitor(f.x.body.as_ref().unwrap(), &|e| {
                    if !changed.get() {
                        if let ExprX::Call {
                            target: CallTarget::Fun(k, _, ts, is, attrs),
                            args,
                            post_args,
                            body,
                        } = &e.x
                        {
                            changed.set(true);
                            return Ok(e.new_x(ExprX::Call {
                                target: CallTarget::Fun(
                                    k.clone(),
                                    source.function.x.name.clone(),
                                    ts.clone(),
                                    is.clone(),
                                    attrs.clone(),
                                ),
                                args: args.clone(),
                                post_args: post_args.clone(),
                                body: body.clone(),
                            }));
                        }
                    }
                    Ok(e.clone())
                })
                .unwrap();
                assert!(changed.get());
                let mut bad_target = f.x.clone();
                bad_target.body = Some(wrong);
                assert!(
                    correspondence::check(
                        tcx,
                        cell,
                        source.def_id.expect_local(),
                        &f.new_x(bad_target),
                        admitted.original_headers()
                    )
                    .is_err()
                );
                let mut x = f.x.clone();
                x.body = None;
                assert!(
                    correspondence::check(
                        tcx,
                        cell,
                        source.def_id.expect_local(),
                        &f.new_x(x),
                        admitted.original_headers()
                    )
                    .is_err()
                );
                if let ExprX::Block(ss, tail) = &f.x.body.as_ref().unwrap().x {
                    if ss
                        .iter()
                        .any(|s| matches!(&s.x,StmtX::Expr(e) if matches!(e.x,ExprX::Assign{..})))
                    {
                        let mut x = f.x.clone();
                        let filtered=ss.iter().filter(|s|!matches!(&s.x,StmtX::Expr(e) if matches!(e.x,ExprX::Assign{..}))).cloned().collect();
                        x.body = Some(
                            f.x.body
                                .as_ref()
                                .unwrap()
                                .new_x(ExprX::Block(Arc::new(filtered), tail.clone())),
                        );
                        assert!(
                            correspondence::check(
                                tcx,
                                cell,
                                source.def_id.expect_local(),
                                &f.new_x(x),
                                admitted.original_headers()
                            )
                            .is_err(),
                            "deleted native store cannot preserve correspondence"
                        );
                        if f.x.params.len() == 2 {
                            let next = ss
                                .iter()
                                .find_map(|s| {
                                    if let StmtX::Expr(e) = &s.x {
                                        if let ExprX::Assign { rhs, .. } = &e.x {
                                            Some(rhs.clone())
                                        } else {
                                            None
                                        }
                                    } else {
                                        None
                                    }
                                })
                                .unwrap();
                            let mut scalar = f.x.clone();
                            scalar.body = Some(
                                f.x.body
                                    .as_ref()
                                    .unwrap()
                                    .new_x(ExprX::Block(Arc::new(vec![]), Some(next))),
                            );
                            assert!(
                                correspondence::check(
                                    tcx,
                                    cell,
                                    source.def_id.expect_local(),
                                    &f.new_x(scalar),
                                    admitted.original_headers()
                                )
                                .is_err(),
                                "deleted store with scalar next cannot preserve correspondence; F2 solver still unimplemented"
                            );
                        }
                    }
                }
            }
        }
        assert_eq!(functions.len(), expected, "complete actual full Function inventory");
        assert!(functions.iter().all(|f| f.x.body.is_some() && !f.x.attrs.is_external_body));
        let calls = std::cell::Cell::new(0);
        let stores = std::cell::Cell::new(0);
        for f in &functions {
            let _observed = vir::ast_visitor::map_expr_visitor(f.x.body.as_ref().unwrap(), &|e| {
                match &e.x {
                    ExprX::Call { .. } => calls.set(calls.get() + 1),
                    ExprX::Assign { .. } => stores.set(stores.get() + 1),
                    _ => {}
                }
                Ok(e.clone())
            })
            .unwrap();
            eprintln!("C1 ACTUAL ORIGINAL FUNCTION {:?}: {:?}", f.x.name, f.x.body);
        }
        eprintln!(
            "C1 actual inventory: admitted={}, owners={}, canonical_headers={}, translated={}, native_helpers_installed=0, calls={}, stores={}",
            admitted_functions.len(),
            owners.len(),
            headers.len(),
            functions.len(),
            calls.get(),
            stores.get()
        );
        // No summary/body rewrite or accepted client fact: measure normal WF.
        let mut k = (*pre.krate_without_funs).clone();
        k.functions = functions;
        let k = Arc::new(k);
        let stubs: Arc<HashMap<_, _>> = Arc::new(
            imported
                .functions
                .iter()
                .chain(k.functions.iter())
                .map(|f| (f.x.name.clone(), vir::ast_util::function_to_stub(f)))
                .collect(),
        );
        let traits = vir::traits::merge_external_traits(&k, &[imported.clone()]).unwrap();
        let (initial, errors, _) =
            vir::well_formed::check_crate_initial(&k, &[imported], stubs.clone(), traits, true);
        let mut notes: Vec<_> = errors.iter().map(|e| e.note.clone()).collect();
        let mut checked = 0;
        if let Ok(initial) = initial {
            let demote = vir::traits::DemoteTraitLookup::new(&k, &[], &stubs);
            for f in &k.functions {
                let (wf, errors, _) = vir::well_formed::check_function(
                    &k,
                    stubs.clone(),
                    initial.lookup.clone(),
                    &demote,
                    f,
                    &pre.warning_ctx,
                    true,
                    false,
                );
                notes.extend(errors.iter().map(|e| e.note.clone()));
                match wf {
                    Ok(_) => checked += 1,
                    Err(e) => {
                        notes.extend(e.errors.iter().map(|e| e.note.clone()));
                        notes.extend(e.boundary_errors.iter().map(|(_, e)| e.note.clone()));
                    }
                }
            }
        } else if let Err(e) = initial {
            notes.extend(e.errors.iter().map(|e| e.note.clone()));
        }
        eprintln!(
            "C1 actual checked reconciliation: checked={checked}/{expected}, notes={notes:?}"
        );
        if self.frozen {
            assert!(
                checked < expected,
                "no partial installer may manufacture native client support"
            );
            assert!(!notes.is_empty());
        } else {
            assert_eq!(checked, 1, "zero-helper scalar control is normally WF");
        }
        self.reached = true;
        rustc_driver::Compilation::Stop
    }
}
#[test]
fn original_frontend_inventory_and_first_unproved_native_boundary() {
    const NAME: &str = "checked_owned_cell::clients::original_frontend_inventory_and_first_unproved_native_boundary";
    if std::env::var("C1_PRIVATE_SOURCE_CASE").is_err() {
        for case in ["scalar", "goals", "native"] {
            let r = std::process::Command::new(std::env::current_exe().unwrap())
                .args([NAME, "--exact", "--nocapture", "--test-threads=1"])
                .env("C1_PRIVATE_SOURCE_CASE", case)
                .output()
                .unwrap();
            eprint!("{}", String::from_utf8_lossy(&r.stderr));
            eprint!("{}", String::from_utf8_lossy(&r.stdout));
            assert!(r.status.success(), "original source child {case} failed");
        }
        return;
    }
    let case = std::env::var("C1_PRIVATE_SOURCE_CASE").unwrap();
    let frozen = case != "scalar";
    let raw = case == "native";
    {
        let source = if raw {
            include_str!(
                "../../tools/research/redb_crash_protocol/native_state_link/owned_native.rs"
            )
        } else if frozen {
            include_str!(
                "../../tools/research/redb_crash_protocol/native_state_link/owned_goals.rs"
            )
        } else {
            "use vstd::prelude::*; verus! {pub fn scalar(v:u64)->(r:u64) ensures r==v, {v}}"
        };
        let path =
            std::env::temp_dir().join(format!("C1-original-{}-{frozen}.rs", std::process::id()));
        std::fs::write(&path, source).unwrap();
        let mut a = crate::config::ArgsX::new();
        a.no_cheating = true;
        a.no_external_by_default = raw;
        let normal = crate::verifier::VerifierCallbacksEraseMacro {
            verifier: crate::verifier::Verifier::new(
                Arc::new(a),
                None,
                false,
                crate::cargo_verus_dep_tracker::DepTracker::init(),
            ),
            rust_start_time: Instant::now(),
            rust_end_time: None,
            tc_start_time: None,
            tc_end_time: None,
            verus_externs: None,
            spans: None,
            unresolved_import_deps: HashSet::new(),
        };
        let mut callback = Original { normal, frozen, raw, reached: false };
        let root = std::path::PathBuf::from(std::env::var_os("VERUS_ROOT").unwrap());
        let externs =
            crate::externs::VerusExterns { verus_root: root, has_vstd: true, has_builtin: true };
        let mut args = vec![
            "rustc".into(),
            path.display().to_string(),
            "--crate-name=original_clients".into(),
            "--crate-type=lib".into(),
            "--edition=2024".into(),
            "--cfg=verus_only".into(),
            "--cfg=verus_keep_ghost".into(),
            "--cfg=verus_keep_ghost_body".into(),
            "--sysroot".into(),
            format!(
                "{}/toolchains/1.99.1-x86_64-unknown-linux-gnu",
                std::env::var("RUSTUP_HOME").unwrap()
            ),
        ];
        let mut dependencies: Vec<String> = externs.to_args().collect();
        if raw {
            // Force normal diagnostic crates to load even though raw native Rust
            // has no vstd/prelude reference. No source text or body is changed.
            for d in &mut dependencies {
                if d.starts_with("verus_builtin=") || d.starts_with("vstd=") {
                    *d = format!("force:{d}");
                }
            }
            args.push("-Zunstable-options".into());
        }
        args.extend(dependencies);
        crate::config::enable_default_features_and_verus_attr(&mut args, true, false);
        rustc_driver::run_compiler(&args, &mut callback);
        assert!(callback.reached);
        std::fs::remove_file(path).unwrap();
    }
}
