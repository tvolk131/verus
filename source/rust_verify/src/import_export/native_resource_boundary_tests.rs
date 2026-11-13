use super::*;
use crate::config::ArgsX;
use std::path::Path;
use std::process::Command;
use vir::ast::*;
use vir::ast_util::unit_typ;
use vir::def::Spanned;

const MARKER: &str = "experimental-native-resource-ir:v1:boundary-test";
const MODES: [&str; 8] = [
    "ordinary",
    "no-verify",
    "no-lifetime",
    "verify-root",
    "verify-function",
    "verify-module",
    "verify-only-module",
    "build-sst",
];

fn fixture(marked: bool, private: bool, bodyless: bool) -> Krate {
    let span = vir::messages::Span {
        raw_span: Arc::new(()),
        id: 0,
        data: vec![],
        as_string: "serialized resource boundary fixture".into(),
    };
    let path = Arc::new(PathX {
        krate: CrateId::Id(Arc::new("boundary_fixture".into()), 1),
        segments: Arc::new(vec![]),
    });
    let ret = Spanned::new(
        span.clone(),
        ParamX {
            name: VarIdent(Arc::new("result".into()), VarIdentDisambiguate::RustcId(0)),
            typ: unit_typ(),
            mode: Mode::Exec,
            user_mut: false,
            unwrapped_info: None,
        },
    );
    let function = Spanned::new(
        span.clone(),
        FunctionX {
            name: Arc::new(FunX {
                path: Arc::new(PathX {
                    krate: path.krate.clone(),
                    segments: Arc::new(vec![Arc::new(
                        if marked { "resource_fixture" } else { "ordinary_fixture" }.into(),
                    )]),
                }),
            }),
            proxy: None,
            kind: FunctionKind::Static,
            visibility: Visibility { restricted_to: private.then(|| path.clone()) },
            body_visibility: BodyVisibility::Uninterpreted,
            opaqueness: Opaqueness::Opaque,
            owning_module: Some(path.clone()),
            mode: Mode::Exec,
            typ_params: Arc::new(vec![]),
            typ_bounds: Arc::new(vec![]),
            params: Arc::new(vec![]),
            outer_ret: ret.clone(),
            inner_ret: ret,
            require: Arc::new(vec![]),
            ensure: (Arc::new(vec![]), Arc::new(vec![])),
            returns: None,
            decrease: Arc::new(vec![]),
            decrease_when: None,
            decrease_by: None,
            fndef_axioms: None,
            mask_spec: None,
            atomic_update: None,
            unwind_spec: None,
            item_kind: ItemKind::Function,
            attrs: Arc::new(FunctionAttrsX {
                infer_candidate_id: marked.then(|| MARKER.into()),
                ..FunctionAttrsX::default()
            }),
            body: (!bodyless).then(|| {
                SpannedTyped::new(&span, &unit_typ(), ExprX::Block(Arc::new(vec![]), None))
            }),
            return_opaque_type_eqs: Arc::new(vec![]),
            extra_dependencies: vec![],
            hidden: Arc::new(vec![]),
        },
    );
    let functions = if marked {
        vec![fixture(false, false, false).functions[0].clone(), function]
    } else {
        vec![function]
    };
    Arc::new(KrateX {
        functions,
        reveal_groups: vec![],
        datatypes: vec![],
        traits: vec![],
        trait_impls: vec![],
        assoc_type_impls: vec![],
        modules: vec![Spanned::new(span, ModuleX { path, reveals: None })],
        external_fns: vec![],
        external_types: vec![],
        path_as_rust_names: vec![],
        opaque_types: vec![],
        arch: Arch { word_bits: ArchWordBits::Exactly(64) },
    })
}

fn metadata() -> CrateMetadata {
    CrateMetadata { crate_id: 1, original_files: HashMap::new(), inference_aware: false }
}

fn args(mode: &str) -> ArgsX {
    let mut args = ArgsX::new();
    match mode {
        "ordinary" => {}
        "no-verify" => args.no_verify = true,
        "no-lifetime" => args.no_lifetime = true,
        "verify-root" => args.verify_root = true,
        "verify-function" => {
            args.verify_function = Some("body".into());
            args.verify_only_module = vec!["selected".into()];
        }
        "verify-module" => args.verify_module = vec!["selected".into()],
        "verify-only-module" => args.verify_only_module = vec!["selected".into()],
        "build-sst" => {
            args.no_verify = true;
            args.build_sst = true;
        }
        _ => panic!("unknown boundary test mode"),
    }
    args
}

fn import_args(path: &Path, via_cargo: bool, mode: &str) -> (Args, Vec<(String, String)>) {
    let mut args = args(mode);
    let import = ("boundary_fixture".into(), path.to_str().unwrap().into());
    let cargo = if via_cargo {
        vec![import]
    } else {
        args.import.push(import);
        vec![]
    };
    (Arc::new(args), cargo)
}

// Exercise the real importer, including its TyCtxt-bound foreign checks.
// The transport fixtures contain no foreign certificate; no checker is mocked.
fn import_crates(args: &Args, cargo: Vec<(String, String)>) -> Result<ImportOutput, VirErr> {
    struct Import<'a> {
        args: &'a Args,
        cargo: Option<Vec<(String, String)>>,
        output: Option<Result<ImportOutput, VirErr>>,
    }
    impl rustc_driver::Callbacks for Import<'_> {
        fn after_analysis<'tcx>(
            &mut self,
            _: &rustc_interface::interface::Compiler,
            tcx: rustc_middle::ty::TyCtxt<'tcx>,
        ) -> rustc_driver::Compilation {
            self.output = Some(super::import_crates(self.args, self.cargo.take().unwrap(), tcx));
            rustc_driver::Compilation::Stop
        }
    }
    let directory = tempfile::tempdir().unwrap();
    let input = directory.path().join("import_context.rs");
    std::fs::write(&input, "pub fn context() {}\n").unwrap();
    let sysroot = Command::new("rustc").args(["--print", "sysroot"]).output().unwrap();
    assert!(sysroot.status.success());
    let argv = vec![
        "rustc".into(),
        input.display().to_string(),
        "--crate-type=lib".into(),
        "--sysroot".into(),
        String::from_utf8(sysroot.stdout).unwrap().trim().into(),
        "--out-dir".into(),
        directory.path().display().to_string(),
    ];
    let mut callback = Import { args, cargo: Some(cargo), output: None };
    rustc_driver::run_compiler(&argv, &mut callback);
    callback.output.expect("import callback did not run")
}

fn assert_import_rejected(args: &Args, cargo: Vec<(String, String)>) {
    let error = match import_crates(args, cargo) {
        Ok(_) => panic!("marked resource artifact was imported"),
        Err(error) => error,
    };
    assert!(error.note.contains("experimental native-resource IR: import is unavailable"));
    assert!(error.spans.is_empty(), "import driver requires span-free errors");
}

#[test]
fn ordinary_exports_roundtrip_through_explicit_and_cargo_imports() {
    let directory = tempfile::tempdir().unwrap();
    for via_cargo in [false, true] {
        let path = directory.path().join(format!("ordinary-{via_cargo}.vir"));
        let mut args = ArgsX::new();
        if !via_cargo {
            args.export = Some(path.to_str().unwrap().into());
        }
        export_crate(
            &Arc::new(args),
            &via_cargo.then(|| path.clone()),
            metadata(),
            fixture(false, false, false),
            None,
        )
        .unwrap();
        for mode in MODES {
            let (args, cargo) = import_args(&path, via_cargo, mode);
            let imported = import_crates(&args, cargo).unwrap();
            assert_eq!(imported.vir_crates.len(), 1);
            assert_eq!(imported.vir_crates[0].functions.len(), 1);
            assert!(imported.vir_crates[0].functions[0].x.body.is_none());
            assert!(imported.inference_dependencies.is_empty());
        }
    }
}

#[test]
fn serialized_markers_are_rejected_before_import_filters_and_metadata() {
    let directory = tempfile::tempdir().unwrap();
    for private in [false, true] {
        for bodyless in [false, true] {
            for marked in [false, true] {
                let path =
                    directory.path().join(format!("marked-{marked}-{private}-{bodyless}.vir"));
                let mut metadata = metadata();
                metadata.inference_aware = true;
                atomic_export(
                    &path,
                    &CrateWithMetadata { krate: fixture(marked, private, bodyless), metadata },
                    None,
                )
                .unwrap();
                let mut file = std::fs::OpenOptions::new().append(true).open(&path).unwrap();
                file.write_all(b"not valid inference metadata").unwrap();
                file.write_all(INFER_MAGIC).unwrap();
                drop(file);
                for via_cargo in [false, true] {
                    for mode in MODES {
                        let (args, cargo) = import_args(&path, via_cargo, mode);
                        if marked {
                            assert_import_rejected(&args, cargo);
                        } else {
                            let error = match import_crates(&args, cargo) {
                                Ok(_) => panic!("invalid inference metadata was imported"),
                                Err(error) => error,
                            };
                            assert!(error.note.contains("invalid inference proof metadata"));
                            assert!(error.spans.is_empty());
                        }
                    }
                }
            }
        }
    }
}

#[test]
fn marked_exports_are_rejected_before_private_pruning_and_verified_metadata() {
    let directory = tempfile::tempdir().unwrap();
    let proofs = vir::infer_proof::Proofs::default();
    for private in [false, true] {
        for bodyless in [false, true] {
            for via_cargo in [false, true] {
                for verified in [false, true] {
                    let path = directory.path().join("refused.vir");
                    let mut args = ArgsX::new();
                    if !via_cargo {
                        args.export = Some(path.to_str().unwrap().into());
                    }
                    let error = export_crate(
                        &Arc::new(args),
                        &via_cargo.then(|| path.clone()),
                        metadata(),
                        fixture(true, private, bodyless),
                        verified.then_some((&[], &proofs, &[], &[])),
                    )
                    .unwrap_err();
                    assert!(
                        error
                            .note
                            .contains("experimental native-resource IR: export is unavailable")
                    );
                    assert!(!path.exists());
                    assert_eq!(std::fs::read_dir(directory.path()).unwrap().count(), 0);
                }
            }
        }
    }
}

#[test]
fn stale_export_invalidation_covers_direct_and_cargo_paths() {
    let directory = tempfile::tempdir().unwrap();
    for via_cargo in [false, true] {
        let path = directory.path().join("stale.vir");
        std::fs::write(&path, b"stale certified interface").unwrap();
        let mut args = ArgsX::new();
        if !via_cargo {
            args.export = Some(path.to_str().unwrap().into());
        }
        let args = Arc::new(args);
        let cargo = via_cargo.then(|| path.clone());
        invalidate_export(&args, &cargo).unwrap();
        assert!(!path.exists());
        assert!(export_crate(&args, &cargo, metadata(), fixture(true, true, true), None).is_err());
        assert!(!path.exists());
    }
}

#[test]
#[ignore = "requires matching VERUS_NATIVE_RESOURCE_TEST_DRIVER and VERUS_NATIVE_RESOURCE_TEST_BUILTIN"]
fn actual_driver_rejects_marked_imports_without_panicking_or_leaving_stale_exports() {
    let driver = std::env::var_os("VERUS_NATIVE_RESOURCE_TEST_DRIVER")
        .expect("set VERUS_NATIVE_RESOURCE_TEST_DRIVER to the matching rust_verify binary");
    let builtin = std::env::var_os("VERUS_NATIVE_RESOURCE_TEST_BUILTIN")
        .expect("set VERUS_NATIVE_RESOURCE_TEST_BUILTIN to the matching verus_builtin library");
    let directory = tempfile::tempdir().unwrap();
    let source = directory.path().join("consumer.rs");
    std::fs::write(
        &source,
        concat!(
            "extern crate verus_builtin; #[verifier::verify] pub fn root_body() {} ",
            "pub mod selected { #[verifier::verify] pub fn body() {} }\n",
        ),
    )
    .unwrap();
    let imported = directory.path().join("dependency.vir");
    let exported = directory.path().join("consumer.vir");
    for marked in [false, true] {
        let krate = if marked {
            fixture(true, true, true)
        } else {
            let mut krate = (*fixture(false, false, true)).clone();
            krate.functions.clear();
            krate.modules.clear();
            Arc::new(krate)
        };
        atomic_export(&imported, &CrateWithMetadata { krate, metadata: metadata() }, None).unwrap();
        for mode in MODES {
            std::fs::write(&exported, b"stale certified interface").unwrap();
            let mut command = Command::new(&driver);
            command
                .current_dir(directory.path())
                .arg("--internal-test-mode")
                .arg(&source)
                .args(["--crate-type=lib", "--no-vstd", "--output-json"])
                .args(["--extern", &format!("verus_builtin={}", Path::new(&builtin).display())])
                .args(["--import", &format!("boundary_fixture={}", imported.display())])
                .arg("--export")
                .arg(&exported);
            match mode {
                "ordinary" => {}
                "no-verify" => {
                    command.arg("--no-verify");
                }
                "no-lifetime" => {
                    command.arg("--no-lifetime");
                }
                "verify-root" => {
                    command.arg("--verify-root");
                }
                "verify-function" => {
                    command.args(["--verify-only-module", "selected", "--verify-function", "body"]);
                }
                "verify-module" => {
                    command.args(["--verify-module", "selected"]);
                }
                "verify-only-module" => {
                    command.args(["--verify-only-module", "selected"]);
                }
                "build-sst" => {
                    command.args(["--no-verify", "--build-sst"]);
                }
                _ => panic!("unknown driver test mode"),
            }
            println!("driver marked={marked} mode={mode}: {command:?}");
            let output = command.output().unwrap();
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);
            println!("status={}\nstdout:\n{stdout}\nstderr:\n{stderr}", output.status);
            for marker in ["panicked at", "internal compiler error", "unexpectedly panicked"] {
                assert!(!stderr.contains(marker));
                assert!(!stdout.contains(marker));
            }
            let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
            let filtered = matches!(
                mode,
                "verify-root" | "verify-function" | "verify-module" | "verify-only-module"
            );
            if filtered {
                assert!(report["verification-results"].get("success").is_none());
            } else {
                assert_eq!(report["verification-results"]["success"].as_bool(), Some(!marked));
            }
            assert_eq!(report["verification-results"]["encountered-error"].as_bool(), Some(marked));
            assert_eq!(report["verus-infer"]["crate_success"].as_bool(), Some(!marked));
            if marked {
                assert_eq!(output.status.code(), Some(1));
                assert!(stderr.contains("experimental native-resource IR: import is unavailable"));
                assert_eq!(report["verus-infer"]["interface_written"].as_bool(), Some(false));
                assert!(!exported.exists());
            } else {
                assert!(output.status.success());
                let checked = report["verus-infer"]["verification_scope"]["checked_exec_bodies"]
                    .as_u64()
                    .unwrap();
                if filtered || matches!(mode, "no-verify" | "build-sst") {
                    assert_eq!(checked, 0);
                    assert_eq!(
                        report["verus-infer"]["verification_scope"]["complete_selected_scope_verified"]
                            .as_bool(),
                        Some(false),
                    );
                } else {
                    assert!(checked > 0);
                }
                if !matches!(mode, "no-verify" | "build-sst") {
                    assert!(report["verification-results"]["verified"].as_u64().unwrap() > 0);
                }
                assert!(exported.exists());
            }
        }
    }
}
