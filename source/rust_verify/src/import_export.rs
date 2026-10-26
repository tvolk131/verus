use crate::config::Args;
use crate::spans::FileStartEndPos;
use crate::verifier::io_vir_err;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, HashMap};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use vir::ast::{Fun, Krate, Mode, VirErr};
use vir::infer::ProofStatus;

// Ordinary .vir files retain their existing representation. Inference proof
// metadata is an optional trailer, required whenever an inferred API is present.
const INFER_MAGIC: &[u8; 8] = b"VRINFER1";
const INFER_SCHEMA: u32 = 12;
const INFER_POLICY: &str = "candidate-bound-evidence-v12";
const MAX_INFER_METADATA: u64 = 1024 * 1024;

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct CrateMetadata {
    pub crate_id: u64,
    pub original_files: HashMap<Vec<u8>, FileStartEndPos>,
}

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct CrateWithMetadata {
    krate: Krate,
    metadata: CrateMetadata,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct InferenceDependency {
    pub crate_id: u64,
    pub fingerprint: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct InferenceMetadata {
    schema: u32,
    policy: String,
    producer: String,
    soundness: bool,
    maximality: bool,
    exactness: bool,
    proofs: Vec<(Fun, ProofStatus)>,
    summaries: String,
    dependencies: Vec<InferenceDependency>,
    trusted_models: Vec<(Fun, String)>,
}

fn digest(bytes: impl AsRef<[u8]>) -> String {
    hex::encode(Sha256::digest(bytes))
}

fn producer() -> String {
    let info = crate::util::verus_build_info();
    format!("{}:{}:{}", info.version, info.toolchain, info.sha)
}

fn summaries(krate: &Krate) -> Result<String, VirErr> {
    let functions: Vec<_> = krate.functions.iter().filter(|f| f.x.attrs.infer).collect();
    bincode_next::serde::encode_to_vec((&functions, &krate.arch), bincode_next::config::legacy())
        .map(digest)
        .map_err(|_| crate::util::error("failed to fingerprint inferred contracts"))
}

pub(crate) fn trusted_model_fingerprint(
    function: &vir::ast::Function,
    artifact: &str,
) -> Result<String, VirErr> {
    // Cover the model's spec functions and axioms too, not just the executable
    // declaration. The ordinary library remains trusted, but cannot silently
    // change underneath an inference certificate from a previous build.
    bincode_next::serde::encode_to_vec((function, artifact), bincode_next::config::legacy())
        .map(digest)
        .map_err(|_| crate::util::error("failed to fingerprint trusted standard model"))
}

fn inference_error(path: &str, detail: &str) -> VirErr {
    crate::util::error(format!(
        "Verus Infer: {detail} in imported library `{path}`; rebuild the library and its dependents"
    ))
}

pub(crate) struct ImportOutput {
    pub(crate) vir_crates: Vec<Krate>,
    pub(crate) metadatas: Vec<CrateMetadata>,
    pub(crate) inference_dependencies: Vec<InferenceDependency>,
    pub(crate) trusted_models: Vec<(Fun, String)>,
    pub(crate) available_trusted_models: HashMap<Fun, String>,
}

pub(crate) fn import_crates(
    args: &Args,
    import_virs_via_cargo: Vec<(String, String)>,
) -> Result<ImportOutput, VirErr> {
    let mut metadatas = Vec::new();
    let mut vir_crates = Vec::new();
    let mut inference = Vec::new();
    let mut fingerprints = BTreeMap::new();
    let mut available_trusted_models = HashMap::new();
    for (_, file_path) in args.import.iter().chain(import_virs_via_cargo.iter()) {
        let mut file = std::io::BufReader::new(std::fs::File::open(file_path).map_err(|err| {
            io_vir_err(format!("could not open imported library file `{file_path}`"), err)
        })?);
        let CrateWithMetadata { krate, metadata } = bincode_next::serde::decode_from_std_read(
            &mut file, bincode_next::config::legacy(),
        ).map_err(|_| crate::util::error(format!(
            "failed to deserialize imported library file `{file_path}` - it may need to be rebuilt by Verus"
        )))?;
        let models: Vec<_> =
            krate.functions.iter().filter(|f| vir::infer::is_approved_model(f)).collect();
        if !models.is_empty() {
            let artifact = digest(
                std::fs::read(file_path)
                    .map_err(|e| io_vir_err("read trusted-model artifact".into(), e))?,
            );
            for function in models {
                available_trusted_models.insert(
                    function.x.name.clone(),
                    trusted_model_fingerprint(function, &artifact)?,
                );
            }
        }
        if krate.functions.iter().any(|f| f.x.attrs.infer) {
            let mut trailer = Vec::new();
            file.by_ref()
                .take(MAX_INFER_METADATA + 1)
                .read_to_end(&mut trailer)
                .map_err(|e| io_vir_err("read inference metadata".into(), e))?;
            if trailer.len() as u64 > MAX_INFER_METADATA || !trailer.ends_with(INFER_MAGIC) {
                return Err(inference_error(
                    file_path,
                    "missing or invalid inference proof metadata",
                ));
            }
            let certificate: InferenceMetadata =
                serde_json::from_slice(&trailer[..trailer.len() - INFER_MAGIC.len()])
                    .map_err(|_| inference_error(file_path, "invalid inference proof metadata"))?;
            if certificate.schema != INFER_SCHEMA
                || certificate.policy != INFER_POLICY
                || certificate.producer != producer()
            {
                return Err(inference_error(
                    file_path,
                    "incompatible inference schema, safety policy, or producer",
                ));
            }
            let proofs: HashMap<_, _> = certificate.proofs.iter().cloned().collect();
            let inferred: Vec<_> = krate.functions.iter().filter(|f| f.x.attrs.infer).collect();
            if proofs.len() != certificate.proofs.len()
                || proofs.len() != inferred.len()
                || !certificate.soundness
                || certificate.maximality != certificate.proofs.iter().all(|(_, p)| p.maximality)
                || certificate.exactness != certificate.proofs.iter().all(|(_, p)| p.exactness)
                || inferred.iter().any(|f| {
                    !proofs
                        .get(&f.x.name)
                        .is_some_and(|p| p.satisfies(f.x.attrs.infer_allow_approx))
                })
            {
                return Err(inference_error(file_path, "incomplete inference proof guarantees"));
            }
            if krate.functions.iter().any(|f| f.x.attrs.infer && f.x.body.is_some())
                || certificate.summaries != summaries(&krate)?
            {
                return Err(inference_error(file_path, "inferred contract fingerprint mismatch"));
            }
            for f in krate.functions.iter().filter(|f| f.x.attrs.infer) {
                vir::infer::validate_imported_summary(f)?;
            }
            let fingerprint = digest(serde_json::to_vec(&certificate).unwrap());
            if let Some(old) = fingerprints.insert(metadata.crate_id, fingerprint.clone()) {
                if old != fingerprint {
                    return Err(inference_error(
                        file_path,
                        "conflicting inference artifacts for one crate",
                    ));
                }
            }
            inference.push((file_path.clone(), metadata.crate_id, certificate));
        }
        vir_crates.push(krate);
        metadatas.push(metadata);
    }
    let mut trusted_models = BTreeMap::new();
    for (path, id, certificate) in &inference {
        for (name, fingerprint) in &certificate.trusted_models {
            if available_trusted_models.get(name) != Some(fingerprint) {
                return Err(inference_error(path, "stale or missing trusted-model fingerprint"));
            }
            trusted_models.insert(name.clone(), fingerprint.clone());
        }
        for dependency in &certificate.dependencies {
            if dependency.crate_id == *id
                || fingerprints.get(&dependency.crate_id) != Some(&dependency.fingerprint)
            {
                return Err(inference_error(
                    path,
                    "stale or missing inferred dependency fingerprint",
                ));
            }
        }
    }
    let inference_dependencies = fingerprints
        .into_iter()
        .map(|(crate_id, fingerprint)| InferenceDependency { crate_id, fingerprint })
        .collect();
    Ok(ImportOutput {
        vir_crates,
        metadatas,
        inference_dependencies,
        trusted_models: trusted_models.into_iter().collect(),
        available_trusted_models,
    })
}

fn export_path(args: &Args, via_cargo: &Option<PathBuf>) -> Option<PathBuf> {
    args.export.as_ref().map(PathBuf::from).or_else(|| via_cargo.clone())
}

/// An unsuccessful re-verification must not leave an older certified inference
/// artifact at the requested output path. Ordinary exports retain their behavior.
pub(crate) fn invalidate_inferred_export(
    args: &Args,
    via_cargo: &Option<PathBuf>,
) -> Result<(), VirErr> {
    let Some(path) = export_path(args, via_cargo) else {
        return Ok(());
    };
    let mut file = match std::fs::File::open(&path) {
        Ok(file) => file,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(e) => return Err(io_vir_err("open previous inference export".into(), e)),
    };
    if file.metadata().map_err(|e| io_vir_err("stat previous inference export".into(), e))?.len()
        >= INFER_MAGIC.len() as u64
    {
        file.seek(SeekFrom::End(-(INFER_MAGIC.len() as i64)))
            .map_err(|e| io_vir_err("seek previous inference export".into(), e))?;
        let mut marker = [0; 8];
        file.read_exact(&mut marker)
            .map_err(|e| io_vir_err("read previous inference export".into(), e))?;
        if &marker == INFER_MAGIC {
            drop(file);
            std::fs::remove_file(&path)
                .map_err(|e| io_vir_err("invalidate previous inference export".into(), e))?;
        }
    }
    Ok(())
}

pub(crate) fn export_crate(
    args: &Args,
    export_vir_path_via_cargo: &Option<PathBuf>,
    vir_metadata: CrateMetadata,
    vir_crate: Krate,
    // Some is supplied only by the post-verification publication barrier.
    verified_inference: Option<(
        &[InferenceDependency],
        &vir::infer_proof::Proofs,
        &[(Fun, String)],
    )>,
) -> Result<(), VirErr> {
    let Some(file_path) = export_path(args, export_vir_path_via_cargo) else {
        return Ok(());
    };
    if vir_crate.functions.iter().any(|f| f.x.attrs.infer) && verified_inference.is_none() {
        return Err(crate::util::error("Verus Infer: cannot export unverified inferred contracts"));
    }
    let mut kratex = (*vir_crate).clone();
    kratex.functions.retain(|f| f.x.visibility.restricted_to.is_none());
    for func in kratex.functions.iter_mut() {
        let mut functionx = func.x.clone();
        functionx.decrease_by = None;
        functionx.return_opaque_type_eqs = Arc::new(vec![]);
        if (functionx.mode != Mode::Spec || !functionx.body_visibility.is_public())
            && !matches!(&functionx.kind, vir::ast::FunctionKind::TraitMethodDecl { .. })
        {
            functionx.body = None;
        }
        *func = func.new_x(functionx);
    }
    let krate = Arc::new(kratex);
    let inference = if krate.functions.iter().any(|f| f.x.attrs.infer) {
        let (dependencies, verified, models) = verified_inference.unwrap();
        let mut proofs = Vec::new();
        for function in krate.functions.iter().filter(|f| f.x.attrs.infer) {
            let proof = verified.status(function)?;
            if !proof.satisfies(function.x.attrs.infer_allow_approx) {
                return Err(crate::util::error(
                    "Verus Infer: cannot export unproved inference guarantees",
                ));
            }
            proofs.push((function.x.name.clone(), proof));
        }
        Some(InferenceMetadata {
            schema: INFER_SCHEMA,
            policy: INFER_POLICY.into(),
            producer: producer(),
            soundness: true,
            maximality: proofs.iter().all(|(_, p)| p.maximality),
            exactness: proofs.iter().all(|(_, p)| p.exactness),
            proofs,
            summaries: summaries(&krate)?,
            dependencies: dependencies.to_vec(),
            trusted_models: models.to_vec(),
        })
    } else {
        None
    };
    atomic_export(
        &file_path,
        &CrateWithMetadata { krate, metadata: vir_metadata },
        inference.as_ref(),
    )
}

fn atomic_export(
    path: &Path,
    krate: &CrateWithMetadata,
    inference: Option<&InferenceMetadata>,
) -> Result<(), VirErr> {
    let tmp = path.with_extension(format!("vir.{}.tmp", std::process::id()));
    let result = (|| -> Result<(), VirErr> {
        let mut file = std::io::BufWriter::new(
            std::fs::File::create(&tmp)
                .map_err(|e| io_vir_err(format!("create export `{}`", tmp.display()), e))?,
        );
        bincode_next::serde::encode_into_std_write(
            krate,
            &mut file,
            bincode_next::config::legacy(),
        )
        .map_err(|_| crate::util::error("failed to serialize exported library"))?;
        if let Some(inference) = inference {
            serde_json::to_writer(&mut file, inference)
                .map_err(|_| crate::util::error("failed to serialize inference metadata"))?;
            file.write_all(INFER_MAGIC)
                .map_err(|e| io_vir_err("write inference metadata".into(), e))?;
        }
        file.flush().map_err(|e| io_vir_err("flush exported library".into(), e))?;
        drop(file);
        std::fs::rename(&tmp, path)
            .map_err(|e| io_vir_err(format!("publish exported library `{}`", path.display()), e))
    })();
    if result.is_err() {
        let _ = std::fs::remove_file(tmp);
    }
    result
}

#[cfg(test)]
mod infer_arch_tests {
    use super::*;
    use vir::ast::{Arch, ArchWordBits, KrateX};

    #[test]
    fn inference_fingerprint_binds_architecture_without_function_changes() {
        let mut krate = Arc::new(KrateX {
            functions: vec![],
            reveal_groups: vec![],
            datatypes: vec![],
            traits: vec![],
            trait_impls: vec![],
            assoc_type_impls: vec![],
            modules: vec![],
            external_fns: vec![],
            external_types: vec![],
            path_as_rust_names: vec![],
            opaque_types: vec![],
            arch: Arch { word_bits: ArchWordBits::Either32Or64 },
        });
        let portable = summaries(&krate).unwrap();
        Arc::make_mut(&mut krate).arch.word_bits = ArchWordBits::Exactly(32);
        let bits32 = summaries(&krate).unwrap();
        Arc::make_mut(&mut krate).arch.word_bits = ArchWordBits::Exactly(64);
        let bits64 = summaries(&krate).unwrap();
        assert_ne!(portable, bits32);
        assert_ne!(portable, bits64);
        assert_ne!(bits32, bits64);
    }
}
