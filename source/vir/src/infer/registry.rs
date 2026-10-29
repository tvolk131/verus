//! Content-addressed candidate cache. Proof results are deliberately never stored.
use super::*;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::io::Read;
use std::path::PathBuf;

const VERSION: &str = "verus-infer-v42:pointwise-record-traversals:4096";
const MAX_BYTES: u64 = 2 * 1024 * 1024;

fn candidate_key(function: &Function, dependencies: &[String], configuration: &str) -> String {
    Sha256::digest(format!("{VERSION}:{configuration}:{function:?}:{dependencies:?}"))
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

// A cache cannot inject AST nodes, binders, types, spans or executable code.
#[derive(Clone, Serialize, Deserialize)]
enum Formula {
    Bool(bool),
    Int(BigInt),
    Param(usize),
    Not(Box<Formula>),
    BitNot(Option<IntegerTypeBitwidth>, Box<Formula>),
    SliceLen(Box<Formula>),
    Clip(u32, Box<Formula>),
    USizeClip(Box<Formula>),
    ISizeClip(Box<Formula>),
    ISizeMin,
    ISizeMax,
    USizeMax,
    ArchWordBits,
    SignedClip(u32, Box<Formula>),
    Binary(BinaryOp, Box<Formula>, Box<Formula>),
    Logical(LogicalOp, Box<Formula>, Box<Formula>),
    If(Box<Formula>, Box<Formula>, Box<Formula>),
    Ctor(enums::ValueType, String, Option<Box<Formula>>),
    IsVariant(String, Box<Formula>),
    Field(String, Box<Formula>),
    Array(enums::ValueType, usize, Vec<Formula>),
}
#[derive(Clone, Serialize, Deserialize)]
struct Candidate {
    version: String,
    key: String,
    dependencies: Vec<String>,
    pre: Formula,
    post: Formula,
}

fn encode(e: &Expr, params: &Params) -> Formula {
    let enc = |e| Box::new(encode(e, params));
    match &e.x {
        ExprX::UnaryOpr(UnaryOpr::IntegerTypeBound(IntegerTypeBoundKind::SignedMin), _)
            if arithmetic::arch_constant(e).is_some() =>
        {
            Formula::ISizeMin
        }
        ExprX::UnaryOpr(UnaryOpr::IntegerTypeBound(IntegerTypeBoundKind::SignedMax), _)
            if arithmetic::arch_constant(e).is_some() =>
        {
            Formula::ISizeMax
        }
        ExprX::Unary(UnaryOp::Clip { range: IntRange::ISize, .. }, a) => Formula::ISizeClip(enc(a)),

        ExprX::UnaryOpr(UnaryOpr::IntegerTypeBound(IntegerTypeBoundKind::UnsignedMax), _)
            if arithmetic::arch_constant(e).is_some() =>
        {
            Formula::USizeMax
        }
        ExprX::UnaryOpr(UnaryOpr::IntegerTypeBound(IntegerTypeBoundKind::ArchWordBits), _)
            if arithmetic::arch_constant(e).is_some() =>
        {
            Formula::ArchWordBits
        }
        ExprX::Unary(UnaryOp::Clip { range: IntRange::USize, .. }, a) => Formula::USizeClip(enc(a)),
        ExprX::ArrayLiteral(elements) => {
            let (element, length) = arrays::shape(&e.typ).expect("generated array type");
            Formula::Array(
                enums::ValueType::from_typ(&element).unwrap(),
                length,
                elements.iter().map(|e| encode(e, params)).collect(),
            )
        }
        ExprX::Const(Constant::Bool(v)) => Formula::Bool(*v),
        ExprX::Const(Constant::Int(v)) => Formula::Int(v.clone()),
        ExprX::Var(v) => Formula::Param(params.iter().position(|p| &p.x.name == v).unwrap()),
        ExprX::Unary(UnaryOp::BitNot(w), a) => Formula::BitNot(*w, enc(a)),
        ExprX::Unary(UnaryOp::Not, a) => Formula::Not(enc(a)),
        ExprX::Unary(UnaryOp::Length(ArrayKind::Slice), a) => Formula::SliceLen(enc(a)),
        ExprX::Unary(UnaryOp::Clip { range: IntRange::U(bits), .. }, a) => {
            Formula::Clip(*bits, enc(a))
        }
        ExprX::Unary(UnaryOp::Clip { range: IntRange::I(bits), .. }, a) => {
            Formula::SignedClip(*bits, enc(a))
        }
        ExprX::Binary(op, a, b) => Formula::Binary(*op, enc(a), enc(b)),
        ExprX::Logical(op, a, b) => Formula::Logical(*op, enc(a), enc(b)),
        ExprX::If(c, a, Some(b)) => Formula::If(enc(c), enc(a), enc(b)),
        ExprX::Ctor(_, variant, fields, None) => Formula::Ctor(
            enums::ValueType::from_typ(&e.typ).unwrap(),
            variant.to_string(),
            fields.first().map(|f| enc(&f.a)),
        ),
        ExprX::UnaryOpr(UnaryOpr::IsVariant { variant, .. }, a) => {
            Formula::IsVariant(variant.to_string(), enc(a))
        }
        ExprX::UnaryOpr(UnaryOpr::Field(op), a) => Formula::Field(op.variant.to_string(), enc(a)),
        _ => unreachable!("only generated formulas are cached"),
    }
}
fn boolean_type(t: &Term) -> bool {
    matches!(&*t.expr.typ, TypX::Bool)
}

fn decode(
    f: &Formula,
    function: &Function,
    remaining: &mut usize,
    allow_result: bool,
) -> Result<Term, VirErr> {
    let span = &function.span;
    let invalid = || unsupported(span, "invalid cached formula");
    if *remaining == 0 {
        return Err(invalid());
    }
    *remaining -= 1;
    match f {
        Formula::ISizeMin => Ok(arithmetic::arch_bound(span, IntegerTypeBoundKind::SignedMin)),
        Formula::ISizeMax => Ok(arithmetic::arch_bound(span, IntegerTypeBoundKind::SignedMax)),
        Formula::ISizeClip(a) => summary::unary(
            span,
            UnaryOp::Clip { range: IntRange::ISize, truncate: true },
            decode(a, function, remaining, allow_result)?,
        ),

        Formula::USizeMax => Ok(arithmetic::usize_max(span)),
        Formula::ArchWordBits => Ok(arithmetic::arch_bits(span)),
        Formula::USizeClip(a) => summary::unary(
            span,
            UnaryOp::Clip { range: IntRange::USize, truncate: true },
            decode(a, function, remaining, allow_result)?,
        ),
        Formula::Array(element, length, elements) => {
            let typ =
                arrays::typ(element.typ().ok_or_else(invalid)?, *length).ok_or_else(invalid)?;
            if elements.len() != *length {
                return Err(invalid());
            }
            arrays::literal(
                span,
                &typ,
                elements
                    .iter()
                    .map(|e| decode(e, function, remaining, allow_result))
                    .collect::<Result<Vec<_>, _>>()?,
            )
        }
        Formula::Ctor(typ, variant, field) => enums::ctor(
            span,
            &typ.typ().ok_or_else(invalid)?,
            &Arc::new(variant.clone()),
            field.as_ref().map(|f| decode(f, function, remaining, allow_result)).transpose()?,
        ),
        Formula::IsVariant(variant, a) => enums::is_variant(
            span,
            decode(a, function, remaining, allow_result)?,
            &Arc::new(variant.clone()),
        ),
        Formula::Field(variant, a) => enums::field(
            span,
            decode(a, function, remaining, allow_result)?,
            &Arc::new(variant.clone()),
        ),
        Formula::Bool(b) => Ok(boolean(span, *b)),
        Formula::Int(n) if n.bits() <= 4096 => {
            term(span, &int_typ(), ExprX::Const(Constant::Int(n.clone())), 1)
        }
        Formula::Param(i) => {
            let p = if allow_result && *i == function.x.params.len() {
                &function.x.inner_ret
            } else {
                function.x.params.get(*i).ok_or_else(invalid)?
            };
            term(span, &p.x.typ, ExprX::Var(p.x.name.clone()), 1)
        }
        Formula::BitNot(w, a) => {
            summary::unary(span, UnaryOp::BitNot(*w), decode(a, function, remaining, allow_result)?)
        }
        Formula::Not(a) => {
            summary::unary(span, UnaryOp::Not, decode(a, function, remaining, allow_result)?)
        }
        Formula::SliceLen(a) => summary::unary(
            span,
            UnaryOp::Length(ArrayKind::Slice),
            decode(a, function, remaining, allow_result)?,
        ),
        Formula::Clip(bits @ (8 | 16 | 32 | 64 | 128), a) => summary::unary(
            span,
            UnaryOp::Clip { range: IntRange::U(*bits), truncate: true },
            decode(a, function, remaining, allow_result)?,
        ),
        Formula::SignedClip(bits @ (8 | 16 | 32 | 64 | 128), a) => summary::unary(
            span,
            UnaryOp::Clip { range: IntRange::I(*bits), truncate: true },
            decode(a, function, remaining, allow_result)?,
        ),
        Formula::Binary(op, a, b) => summary::binary(
            span,
            *op,
            decode(a, function, remaining, allow_result)?,
            decode(b, function, remaining, allow_result)?,
        ),
        Formula::Logical(op, a, b) => summary::logical(
            span,
            *op,
            decode(a, function, remaining, allow_result)?,
            decode(b, function, remaining, allow_result)?,
        ),
        Formula::If(c, a, b) => summary::conditional(
            span,
            decode(c, function, remaining, allow_result)?,
            decode(a, function, remaining, allow_result)?,
            decode(b, function, remaining, allow_result)?,
        ),
        _ => Err(invalid()),
    }
}

pub struct Registry {
    configuration: String,
    directory: Option<PathBuf>,
    entries: HashMap<String, Candidate>,
    originals: HashMap<Fun, Function>,
    projected: std::collections::HashSet<Fun>,
    datatypes: Vec<Datatype>,
    pub hits: usize,
    pub misses: usize,
    pub generated: Vec<GeneratedCandidate>,
}
impl Registry {
    pub fn from_environment(configuration: &str) -> Self {
        let mut registry = Self::new(std::env::var_os("VERUS_INFER_CACHE").map(PathBuf::from));
        registry.configuration = configuration.into();
        registry
    }
    pub fn new(directory: Option<PathBuf>) -> Self {
        Self {
            configuration: String::new(),
            directory,
            entries: HashMap::new(),
            originals: HashMap::new(),
            projected: std::collections::HashSet::new(),
            datatypes: vec![],
            hits: 0,
            misses: 0,
            generated: Vec::new(),
        }
    }
    fn read(&self, key: &str) -> Option<Candidate> {
        if let Some(c) = self.entries.get(key) {
            return Some(c.clone());
        }
        let file =
            std::fs::File::open(self.directory.as_ref()?.join(format!("{key}.json"))).ok()?;
        if file.metadata().ok()?.len() > MAX_BYTES {
            return None;
        }
        serde_json::from_reader(file.take(MAX_BYTES)).ok()
    }
    fn write(&mut self, c: Candidate) {
        if let Some(dir) = &self.directory {
            let _ = (|| -> std::io::Result<()> {
                std::fs::create_dir_all(dir)?;
                let path = dir.join(format!("{}.json", c.key));
                let tmp = dir.join(format!("{}.{}.tmp", c.key, std::process::id()));
                std::fs::write(&tmp, serde_json::to_vec(&c)?)?;
                std::fs::rename(tmp, path)
            })();
        }
        self.entries.insert(c.key.clone(), c);
    }
    #[cfg(test)]
    pub fn infer(&mut self, functions: &[Function]) -> Result<Vec<Function>, VirErr> {
        self.infer_with_imports(functions, &[])
    }
    pub fn infer_with_imports(
        &mut self,
        functions: &[Function],
        imported: &[Function],
    ) -> Result<Vec<Function>, VirErr> {
        let mut combined = functions.to_vec();
        combined
            .extend(imported.iter().filter(|f| f.x.attrs.infer || is_inference_contract(f) || is_approved_model(f)
                || f.x.name.path.krate != CrateId::Vstd && is_pure_observation(f)).cloned());
        let all: HashMap<_, _> = combined.iter().map(|f| (f.x.name.clone(), f.clone())).collect();
        let mut candidates = HashMap::new();
        let mut keys: HashMap<Fun, String> = HashMap::new();
        // Pure vocabulary can occur in specialized view-query equations. It
        // never becomes an executable model; include its definition in keys.
        for f in combined.iter().filter(|f| f.x.name.path.krate != CrateId::Vstd && is_pure_observation(f)) {
            let key = Sha256::digest(format!("{VERSION}:spec-vocabulary:{f:?}")).iter().map(|b| format!("{b:02x}")).collect();
            keys.insert(f.x.name.clone(), key);
            candidates.insert(f.x.name.clone(), f.clone());
        }
        for f in combined.iter().filter(|f|is_inference_contract(f)) {
            let key=Sha256::digest(format!("{VERSION}:checked-contract:{f:?}")).iter().map(|b|format!("{b:02x}")).collect();
            keys.insert(f.x.name.clone(),key);
            candidates.insert(f.x.name.clone(),f.clone());
        }
        for f in imported.iter().filter(|f| is_approved_model(f)) {
            let key: String = Sha256::digest(format!("{VERSION}:trusted-model:{f:?}"))
                .iter()
                .map(|b| format!("{b:02x}"))
                .collect();
            keys.insert(f.x.name.clone(), key);
            candidates.insert(f.x.name.clone(), f.clone());
        }
        for f in imported.iter().filter(|f| f.x.attrs.infer) {
            if is_owned_summary(f){owned::validate_summary(f,&all,&self.datatypes)?;}
            else if records::candidate(f) { records::validate_summary(f,&all,&self.datatypes)?; }
            else if collections::kind(f).is_some() { collections::validate_summary(f, &all)?; }
            else { validate_imported_summary(f)?; }
            let key: String = Sha256::digest(format!("{VERSION}:import:{f:?}"))
                .iter()
                .map(|b| format!("{b:02x}"))
                .collect();
            keys.insert(f.x.name.clone(), key);
            candidates.insert(f.x.name.clone(), f.clone());
        }
        for name in dependency_order(&combined)? {
            if candidates.contains_key(&name) {
                continue;
            }
            let f = &all[&name];
            let start = std::time::Instant::now();
            if is_owned_summary(f){
                let mut dependencies:Vec<_>=keys.values().cloned().collect();dependencies.sort();dependencies.push(format!("{:?}",self.datatypes));
                let key=candidate_key(self.originals.get(&name).unwrap_or(f),&dependencies,&self.configuration);
                let wrappers=newtypes::Newtypes::new(&self.datatypes);
                let callees=candidates.iter().map(|(n,c)|Ok((n.clone(),if self.projected.contains(n){wrappers.lift(&self.originals[n],c)?.install_lifted(&self.originals[n],&keys[n])?}else{c.clone()}))).collect::<Result<HashMap<_,_>,VirErr>>()?;
                let (contract,body)=owned::infer(f,&callees,&self.datatypes)?;
                let inferred=contract.install_owned(f,&key,body)?;
                self.misses+=1;let mut row=GeneratedCandidate::new(&inferred,false,start.elapsed().as_micros());row.record_template=true;self.generated.push(row);
                keys.insert(name.clone(),key);candidates.insert(name,inferred);continue;
            }
            if records::candidate(f) {
                let mut dependencies: Vec<_> = keys.values().cloned().collect(); dependencies.sort();
                dependencies.push(format!("{:?}",self.datatypes));
                let key=candidate_key(self.originals.get(&name).unwrap_or(f),&dependencies,&self.configuration);
                // Record analysis retains application types. Scalar synthesis
                // temporarily projects wrappers, so lift checked callee formulas
                // before composing them into a record/query candidate.
                let wrappers = newtypes::Newtypes::new(&self.datatypes);
                let record_callees = candidates.iter().map(|(n, c)| {
                    let c = if self.projected.contains(n) {
                        wrappers.lift(&self.originals[n], c)?
                            .install_lifted(&self.originals[n], &keys[n])?
                    } else { c.clone() };
                    Ok((n.clone(), c))
                }).collect::<Result<HashMap<_, _>, VirErr>>()?;
                let (contract, body) = records::infer(f, &record_callees, &self.datatypes)?;
                let inferred = if let Some(body) = body {
                    contract.install_owned(f, &key, body)?
                } else { contract.install_lifted(f, &key)? };
                self.misses+=1;
                let mut row=GeneratedCandidate::new(&inferred,false,start.elapsed().as_micros());row.record_template=true;self.generated.push(row);
                keys.insert(name.clone(),key); candidates.insert(name,inferred); continue;
            }
            if collections::kind(f).is_some() {
                let mut dependencies: Vec<_> = all.values()
                    .filter(|f| collections::model(f).is_some())
                    .map(|f| keys[&f.x.name].clone()).collect();
                dependencies.sort();
                let key = candidate_key(self.originals.get(&name).unwrap_or(f), &dependencies, &self.configuration);
                let inferred = collections::infer(f, &all)?.install_lifted(f, &key)?;
                // Regenerate these bounded templates; never decode arbitrary
                // state expressions through the scalar candidate-cache format.
                self.misses += 1;
                self.generated.push(GeneratedCandidate::new(&inferred, false, start.elapsed().as_micros()));
                keys.insert(name.clone(), key);
                candidates.insert(name, inferred);
                continue;
            }
            validate_signature(f)?;
            validate_body(f, &all)?;
            if !f.x.attrs.infer_allow_approx && uses_approximate_callee(f, &candidates) {
                return Err(unsupported(
                    &f.span,
                    "exact inference requires exact-mode callees; use precision = \"sound\" or an ordinary verified caller for sound approximations",
                ));
            }
            let mut dependencies = Vec::new();
            propagation::visit_calls(
                f.x.body.as_ref().unwrap(),
                &f.x.inner_ret.x.typ,
                &all,
                &mut |e, _| {
                    if let ExprX::Call { target: CallTarget::Fun(_, name, ..), .. } = &e.x {
                        dependencies.push(keys[name].clone());
                    }
                    Ok::<_, VirErr>(())
                },
            )?;
            // Signed operators lower to existing vstd spec functions even
            // though no explicit call appears in their source AST. Record
            // those definitions and fail cleanly if vstd was not imported.
            crate::ast_visitor::expr_visitor_check(f.x.body.as_ref().unwrap(), &mut |_, e| {
                if let Some(name) = arithmetic::division_dependency(e) {
                    dependencies.push(keys.get(&name).ok_or_else(|| unsupported(&e.span,
                        "signed division/remainder requires vstd arithmetic models; import vstd::prelude::*"))?.clone());
                }
                for name in slices::dependencies(e, &all) {
                    dependencies.push(keys.get(&name).ok_or_else(|| unsupported(&e.span,
                        "slice indexing/length requires vstd slice models; import vstd::prelude::*"))?.clone());
                }
                Ok::<_, VirErr>(())
            })?;
            // Includes the entire typed function (body, signature, attributes,
            // safety policy), plus callee/model keys in deterministic traversal order.
            let key = candidate_key(
                self.originals.get(&name).unwrap_or(f),
                &dependencies,
                &self.configuration,
            );
            let hit = self
                .read(&key)
                .filter(|c| c.version == VERSION && c.key == key && c.dependencies == dependencies)
                .and_then(|c| {
                    let mut pre_budget = FORMULA_BUDGET;
                    let mut output_budget = FORMULA_BUDGET;
                    let pre = decode(&c.pre, f, &mut pre_budget, false).ok()?;
                    let output = decode(&c.post, f, &mut output_budget, true).ok()?;
                    if !boolean_type(&pre) || !boolean_type(&output) {
                        return None;
                    }
                    let candidate =
                        Contract::new(pre.expr, output.expr).ok()?.install(f, &key).ok()?;
                    let mut exported = candidate.x.clone();
                    exported.body = None;
                    validate_imported_summary(&candidate.new_x(exported)).ok()?;
                    Some(candidate)
                });
            let cache_hit = hit.is_some();
            let inferred = if let Some(hit) = hit {
                self.hits += 1;
                hit
            } else {
                self.misses += 1;
                let inferred = infer_with_callees(f, &candidates)?.install(f, &key)?;
                let mut post_params = (*f.x.params).clone();
                post_params.push(f.x.inner_ret.clone());
                self.write(Candidate {
                    version: VERSION.into(),
                    key: key.clone(),
                    dependencies,
                    pre: encode(&inferred.x.require[0], &f.x.params),
                    post: encode(&inferred.x.ensure.0[0], &Arc::new(post_params)),
                });
                inferred
            };
            self.generated.push(GeneratedCandidate::new(
                &inferred,
                cache_hit,
                start.elapsed().as_micros(),
            ));
            keys.insert(name.clone(), key);
            candidates.insert(name, inferred);
        }
        Ok(functions.iter().map(|f| candidates.get(&f.x.name).unwrap_or(f).clone()).collect())
    }

    pub fn infer_with_datatypes(
        &mut self,
        functions: &[Function],
        imported: &[Function],
        datatypes: &[Datatype],
    ) -> Result<Vec<Function>, VirErr> {
        let trait_contracts=iterators::prepare(&trait_getters::prepare(functions,imported),imported);
        let prepared=sequence_views::prepare(&trait_contracts,imported,datatypes);
        let prepared=cloning::prepare(&prepared,imported,datatypes);
        let prepared=owned::prepare(&prepared,imported,datatypes);
        let prepared=owned::borrowed::prepare(&prepared,imported)?;
        let functions=prepared.as_slice();
        let wrappers = newtypes::Newtypes::new(datatypes);
        for f in functions.iter().chain(imported).filter(|f| wrapper_traits::kind(f).is_some()) {
            let valid = match wrapper_traits::kind(f).unwrap() {
                wrapper_traits::Kind::From => f.x.params.iter().any(|p| wrappers.wrapper(&p.x.typ))
                    || wrappers.wrapper(&f.x.inner_ret.x.typ) || is_owned_summary(f),
                wrapper_traits::Kind::AddAssign | wrapper_traits::Kind::Compare => wrappers.wrapper(&f.x.params[0].x.typ),
            };
            if !valid {
                return Err(unsupported(&f.span, "wrapper trait inference requires a transparent nongeneric single scalar field"));
            }
        }
        self.datatypes=datatypes.to_vec();
        let configuration = self.configuration.clone();
        self.configuration.push_str(&wrappers.fingerprint());
        self.originals = functions.iter().chain(imported).map(|f| (f.x.name.clone(), f.clone())).collect();
        let result = (|| {
            let all = functions.iter().chain(imported).map(|f| (f.x.name.clone(), f.clone())).collect();
            let rewritten = functions.iter().map(|f| wrapper_traits::rewrite(f, &all))
                .collect::<Result<Vec<_>, _>>()?;
            let projected =
                rewritten.iter().map(|f| wrappers.project(f)).collect::<Result<Vec<_>, _>>()?;
            let imports = imported.iter().map(|f| wrappers.project(f)).collect::<Result<Vec<_>, _>>()?;
            self.projected = rewritten.iter().zip(&projected).chain(imported.iter().zip(&imports))
                .filter(|(a, b)| !Arc::ptr_eq(a, b)).map(|(a, _)| a.x.name.clone()).collect();
            let inferred = self.infer_with_imports(&projected, &imports)?;
            let lifted = functions
                .iter()
                .zip(rewritten.iter())
                .zip(projected.iter())
                .zip(inferred.iter())
                .map(|(((original, rewritten), projection), f)| {
                    if !original.x.attrs.infer { return Ok(f.clone()); }
                    if is_owned_summary(original){return Contract::new(f.x.require[0].clone(),f.x.ensure.0[0].clone())?.install_owned(original,f.x.attrs.infer_candidate_id.as_ref().unwrap(),owned::restore_body(original,f.x.body.as_ref().unwrap())?);}
                    if records::has_loops(original) {
                        return Contract::new(f.x.require[0].clone(),f.x.ensure.0[0].clone())?
                            .install_owned(original,f.x.attrs.infer_candidate_id.as_ref().unwrap(),owned::restore_body(original,f.x.body.as_ref().unwrap())?);
                    }
                    if Arc::ptr_eq(rewritten, projection) {
                        return Contract::new(f.x.require[0].clone(), f.x.ensure.0[0].clone())?
                            .install_lifted(original, f.x.attrs.infer_candidate_id.as_ref().unwrap());
                    }
                    wrappers.lift(original, f)?.install_lifted(
                        original,
                        f.x.attrs.infer_candidate_id.as_ref().unwrap(),
                    )
                })
                .collect::<Result<Vec<_>, VirErr>>()?;
            let by_name: HashMap<_, _> = lifted.iter().map(|f| (&f.x.name, f)).collect();
            for row in &mut self.generated {
                let record_template=row.record_template;
                *row = GeneratedCandidate::new(
                    by_name[&row.function.x.name], row.cache_hit, row.elapsed_micros,
                );
                row.record_template=record_template;
            }
            Ok(lifted)
        })();
        self.configuration = configuration;
        self.originals.clear();
        self.projected.clear();
        self.datatypes.clear();
        if result.is_err() {
            self.generated.clear();
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infer::tests::{function, input};

    #[test]
    fn malformed_slice_length_and_index_cache_types_are_misses() {
        let f = slices::tests::slice_function(false);
        let imports = [slices::tests::model_function(false), slices::tests::model_function(true)];
        let mut registry = Registry::new(None);
        registry.infer_with_imports(&[f.clone()], &imports).unwrap();
        let key = registry.entries.keys().next().unwrap().clone();
        let original = registry.entries[&key].clone();
        let malformed = [
            Formula::SliceLen(Box::new(Formula::Bool(true))),
            Formula::SliceLen(Box::new(Formula::Param(1))),
            Formula::Binary(
                BinaryOp::Index(ArrayKind::Slice, BoundsCheck::Allow),
                Box::new(Formula::Param(0)),
                Box::new(Formula::Bool(false)),
            ),
            Formula::Binary(
                BinaryOp::Index(ArrayKind::Slice, BoundsCheck::Allow),
                Box::new(Formula::Bool(false)),
                Box::new(Formula::Param(1)),
            ),
        ];
        for bad in malformed {
            let mut poison = original.clone();
            poison.pre = bad;
            registry.entries.insert(key.clone(), poison);
            registry.infer_with_imports(&[f.clone()], &imports).unwrap();
        }
        assert_eq!((registry.hits, registry.misses), (0, 5));
    }

    #[test]
    fn disk_roundtrip_corruption_version_and_configuration() {
        let directory = tempfile::tempdir().unwrap();
        let f = function("identity", input("x"));
        let functions = [f];
        let mut first = Registry::new(Some(directory.path().into()));
        first.infer(&functions).unwrap();
        let mut second = Registry::new(Some(directory.path().into()));
        second.infer(&functions).unwrap();
        assert_eq!((second.hits, second.misses), (1, 0));
        let path = std::fs::read_dir(directory.path()).unwrap().next().unwrap().unwrap().path();
        std::fs::write(&path, "corrupted").unwrap();
        let mut corrupted = Registry::new(Some(directory.path().into()));
        corrupted.infer(&functions).unwrap();
        assert_eq!((corrupted.hits, corrupted.misses), (0, 1));
        let mut entry: Candidate = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
        entry.version = "stale".into();
        std::fs::write(&path, serde_json::to_vec(&entry).unwrap()).unwrap();
        let mut stale = Registry::new(Some(directory.path().into()));
        stale.infer(&functions).unwrap();
        assert_eq!((stale.hits, stale.misses), (0, 1));
        let mut configured = Registry::new(Some(directory.path().into()));
        configured.configuration = "different solver/safety/tool configuration".into();
        configured.infer(&functions).unwrap();
        assert_eq!((configured.hits, configured.misses), (0, 1));
    }

    #[test]
    fn malformed_cache_types_are_misses() {
        let f = function("identity", input("x"));
        let mut registry = Registry::new(None);
        registry.infer(&[f.clone()]).unwrap();
        registry.entries.values_mut().next().unwrap().pre = Formula::Param(0); // integer where bool required
        registry.infer(&[f]).unwrap();
        assert_eq!((registry.hits, registry.misses), (0, 2));
    }

    #[test]
    fn cache_cannot_bypass_source_eligibility() {
        let f = function(
            "bad",
            SpannedTyped::new(
                &crate::infer::tests::span(),
                &bool_typ(),
                ExprX::AssertAssume {
                    is_assume: true,
                    msg: None,
                    expr: boolean(&crate::infer::tests::span(), false).expr,
                },
            ),
        );
        let mut registry = Registry::new(None);
        let key = candidate_key(&f, &[], "");
        // A valid-shaped candidate for this exact source key would pass cache
        // decoding. Source assumptions must be rejected before it can be used.
        registry.entries.insert(
            key.clone(),
            Candidate {
                version: VERSION.into(),
                key,
                dependencies: vec![],
                pre: Formula::Bool(true),
                post: Formula::Binary(
                    BinaryOp::Eq(Mode::Spec),
                    Box::new(Formula::Param(f.x.params.len())),
                    Box::new(Formula::Param(0)),
                ),
            },
        );
        assert!(registry.infer(&[f]).is_err());
        assert_eq!(registry.hits, 0);
    }

    #[test]
    fn malformed_and_oversized_disk_candidates_are_regenerated() {
        let directory = tempfile::tempdir().unwrap();
        let functions = [function("identity", input("x"))];
        Registry::new(Some(directory.path().into())).infer(&functions).unwrap();
        let path = std::fs::read_dir(directory.path()).unwrap().next().unwrap().unwrap().path();
        let original = std::fs::read(&path).unwrap();
        let mut wrong_types: Candidate = serde_json::from_slice(&original).unwrap();
        wrong_types.pre = Formula::Logical(
            LogicalOp::And,
            Box::new(Formula::Param(0)),
            Box::new(Formula::Bool(true)),
        );
        let mut wrong_payload: Candidate = serde_json::from_slice(&original).unwrap();
        wrong_payload.pre = Formula::IsVariant("Some".into(), Box::new(Formula::Param(0)));
        let mut oversized_number: Candidate = serde_json::from_slice(&original).unwrap();
        oversized_number.pre = Formula::Binary(
            BinaryOp::Eq(Mode::Spec),
            Box::new(Formula::Int(BigInt::one() << 4096)),
            Box::new(Formula::Param(0)),
        );
        let mut deeply_nested: Candidate = serde_json::from_slice(&original).unwrap();
        for _ in 0..256 {
            deeply_nested.pre = Formula::Not(Box::new(deeply_nested.pre));
        }
        let corruptions = [
            original[..original.len() / 2].to_vec(),
            vec![b' '; MAX_BYTES as usize + 1],
            serde_json::to_vec(&deeply_nested).unwrap(),
            serde_json::to_vec(&wrong_types).unwrap(),
            serde_json::to_vec(&wrong_payload).unwrap(),
            serde_json::to_vec(&oversized_number).unwrap(),
        ];
        for bytes in corruptions {
            std::fs::write(&path, bytes).unwrap();
            let mut registry = Registry::new(Some(directory.path().into()));
            registry.infer(&functions).unwrap();
            assert_eq!((registry.hits, registry.misses), (0, 1));
            assert_eq!(std::fs::read(&path).unwrap(), original);
        }
    }

    #[test]
    fn malformed_array_cache_types_and_lengths_are_misses() {
        let f = crate::infer::tests::array_function();
        let mut registry = Registry::new(None);
        registry.infer(&[f.clone()]).unwrap();
        let key = registry.entries.keys().next().unwrap().clone();
        let original = registry.entries[&key].clone();
        let malformed = [
            Formula::Array(
                enums::ValueType::Unsigned(8),
                2,
                vec![Formula::Bool(true), Formula::Bool(false)],
            ),
            Formula::Array(enums::ValueType::Unsigned(8), 2, vec![]),
            Formula::Array(enums::ValueType::Unsigned(8), 65, vec![]),
            Formula::Array(
                enums::ValueType::Option(Box::new(enums::ValueType::Unsigned(8))),
                0,
                vec![],
            ),
        ];
        for array in malformed {
            let mut poison = original.clone();
            poison.post = Formula::Binary(
                BinaryOp::Eq(Mode::Spec),
                Box::new(Formula::Param(f.x.params.len())),
                Box::new(Formula::Binary(
                    BinaryOp::Index(ArrayKind::Array, BoundsCheck::Allow),
                    Box::new(array),
                    Box::new(Formula::Int(0.into())),
                )),
            );
            registry.entries.insert(key.clone(), poison);
            registry.infer(&[f.clone()]).unwrap();
        }
        assert_eq!((registry.hits, registry.misses), (0, 5));
    }
}
