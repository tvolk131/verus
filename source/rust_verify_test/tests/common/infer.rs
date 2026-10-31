//! Reusable Infer integration support. Proof expectations remain test oracles.
use super::{TestErr, verify_one_file};

pub(crate) struct InferCrates {
    pub(crate) directory: tempfile::TempDir,
    pub(crate) profile: std::path::PathBuf,
}

impl InferCrates {
    pub(crate) fn new() -> Self {
        let executable = std::env::current_exe().unwrap();
        Self {
            directory: tempfile::tempdir().unwrap(),
            profile: executable.parent().unwrap().parent().unwrap().into(),
        }
    }
    pub(crate) fn vir(&self, name: &str) -> std::path::PathBuf {
        self.directory.path().join(format!("{name}.vir"))
    }
    pub(crate) fn build(
        &self,
        name: &str,
        body: &str,
        dependencies: &[&str],
        options: &[&str],
    ) -> std::process::Output {
        self.build_mode(name, body, dependencies, options, true)
    }
    // These model-only tests skip erased native-code checks: global size
    // assertions would otherwise reject the other pointer width on this host.
    pub(crate) fn verify(
        &self,
        name: &str,
        body: &str,
        dependencies: &[&str],
        options: &[&str],
    ) -> std::process::Output {
        self.build_mode(name, body, dependencies, options, false)
    }
    fn build_mode(
        &self,
        name: &str,
        body: &str,
        dependencies: &[&str],
        options: &[&str],
        compile: bool,
    ) -> std::process::Output {
        self.build_source(
            name,
            &format!("use verus_builtin::*; use verus_builtin_macros::*; verus! {{ {body} }}"),
            dependencies,
            options,
            compile,
        )
    }
    pub(crate) fn build_source(
        &self,
        name: &str,
        body: &str,
        dependencies: &[&str],
        options: &[&str],
        compile: bool,
    ) -> std::process::Output {
        let source = self.directory.path().join(format!("{name}.rs"));
        std::fs::write(&source, body).unwrap();
        let mut command = std::process::Command::new(
            self.profile.join(format!("rust_verify{}", std::env::consts::EXE_SUFFIX)),
        );
        command
            .args([
                "--internal-test-mode",
                "--crate-type=lib",
                "--edition=2021",
                "--crate-name",
                name,
            ])
            .arg("--export")
            .arg(self.vir(name))
            .arg("-o")
            .arg(self.directory.path().join(format!("lib{name}.rlib")))
            .arg("--extern")
            .arg(format!("verus_builtin={}", self.profile.join("libverus_builtin.rlib").display()))
            .arg("--extern")
            .arg(format!(
                "verus_builtin_macros={}",
                self.profile
                    .join(format!(
                        "{}verus_builtin_macros.{}",
                        std::env::consts::DLL_PREFIX,
                        std::env::consts::DLL_EXTENSION
                    ))
                    .display()
            ))
            .arg("-L")
            .arg(format!("dependency={}", self.profile.display()))
            .arg("-L")
            .arg(format!("dependency={}", self.directory.path().display()))
            .env(
                "VERUS_Z3_PATH",
                std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                    .parent()
                    .unwrap()
                    .join(if cfg!(windows) { "z3.exe" } else { "z3" }),
            );
        if compile {
            command.arg("--compile");
        } else {
            command.arg("--no-erasure-check");
        }
        if body.contains("vstd::") || body.contains("global size_of") {
            let override_vir = self.directory.path().join("vstd.vir");
            let vstd_vir =
                if override_vir.exists() { override_vir } else { self.profile.join("vstd.vir") };
            command
                .arg("--extern")
                .arg(format!("vstd={}", self.profile.join("libvstd.rlib").display()))
                .arg("--import")
                .arg(format!("vstd={}", vstd_vir.display()));
        } else {
            command.arg("--cfg=verus_no_vstd");
        }
        for dependency in dependencies {
            command
                .arg("--extern")
                .arg(format!(
                    "{dependency}={}",
                    self.directory.path().join(format!("lib{dependency}.rlib")).display()
                ))
                .arg("--import")
                .arg(format!("{dependency}={}", self.vir(dependency).display()));
        }
        // Repeated builds exercise candidate reuse; proof results are never cached.
        command.env("VERUS_INFER_CACHE", self.directory.path().join("candidate-cache"));
        command.args(options).arg(source).output().unwrap()
    }
    pub(crate) fn succeeds(
        &self,
        name: &str,
        body: &str,
        dependencies: &[&str],
    ) -> std::process::Output {
        let run = self.build(name, body, dependencies, &[]);
        assert!(run.status.success(), "{}", String::from_utf8_lossy(&run.stderr));
        run
    }
    pub(crate) fn edit_metadata(&self, name: &str, edit: impl FnOnce(&mut serde_json::Value)) {
        let path = self.vir(name);
        let mut bytes = std::fs::read(&path).unwrap();
        let start =
            bytes.windows(b"{\"schema\":".len()).rposition(|s| s == b"{\"schema\":").unwrap();
        let mut metadata: serde_json::Value =
            serde_json::from_slice(&bytes[start..bytes.len() - 8]).unwrap();
        edit(&mut metadata);
        bytes.truncate(start);
        bytes.extend(serde_json::to_vec(&metadata).unwrap());
        bytes.extend(b"VRINFER1");
        std::fs::write(path, bytes).unwrap();
    }
}

pub(crate) fn json_inference(run: &std::process::Output) -> serde_json::Value {
    let output: serde_json::Value = serde_json::from_slice(&run.stdout).unwrap_or_else(|e| {
        panic!(
            "invalid JSON: {e}: {}\n{}",
            String::from_utf8_lossy(&run.stdout),
            String::from_utf8_lossy(&run.stderr)
        )
    });
    assert_eq!(output["verus-infer"]["schema_version"], 1);
    output["verus-infer"].clone()
}

pub(crate) fn source_name(prefix: &str, source: &str) -> String {
    use std::hash::{Hash, Hasher};
    let mut hash = std::collections::hash_map::DefaultHasher::new();
    source.hash(&mut hash);
    format!("{prefix}_{:x}", hash.finish())
}

pub(crate) fn assert_exact_report(report: &serde_json::Value, count: usize) {
    let functions = report["functions"].as_array().unwrap();
    assert_eq!(functions.len(), count, "{report}");
    for function in functions {
        assert_eq!(function["mode"], "exact", "{function}");
        assert_eq!(function["required_checks_passed"], true, "{function}");
        for obligation in ["soundness", "maximality", "exactness"] {
            assert_eq!(function["proofs"][obligation]["status"], "proved", "{function}");
            assert_eq!(function["proofs"][obligation]["required"], true, "{function}");
        }
    }
}

pub(crate) fn verify_exact(name: &str, source: String, count: usize) -> TestErr {
    let diagnostics =
        verify_one_file(name, source, &["vstd", "--no-cheating", "--output-json"]).unwrap();
    assert_exact_report(&diagnostics.json_output.as_ref().unwrap()["verus-infer"], count);
    diagnostics
}

/// Locate one actual preparation decision without consulting production policy.
pub(crate) fn assistance_decision<'a>(
    report: &'a serde_json::Value,
    profile: &str,
    function_suffix: &str,
) -> &'a serde_json::Value {
    let decisions: Vec<_> = report["assistance_explanations"]["decisions"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|decision| {
            decision["profile"] == profile
                && decision["function"].as_str().unwrap().ends_with(function_suffix)
        })
        .collect();
    assert_eq!(decisions.len(), 1, "{report}");
    decisions[0]
}
