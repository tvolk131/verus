//! Untrusted, bounded construction of complete logical owned values.
//! Original executable bodies and ordinary library specifications remain authoritative.
use super::*;
use records::{atom, coerce};
pub(in crate::infer) mod loops;
mod models;
mod sums;
mod callbacks;
pub(in crate::infer) mod native_for;
mod mutations;
pub(crate) mod result_map;
mod variants;
pub(crate) mod borrowed;
pub(crate) mod factory_callbacks;
pub(in crate::infer) use models::Model;
pub(crate) use models::{call as model_call, model};
fn same(a: &Typ, b: &Typ) -> bool {
    crate::ast_util::types_equal(a, b)
}
fn base(t: &Typ) -> &Typ {
    if let TypX::Decorate(TypDecoration::Ref, None, t) = &**t { t } else { t }
}
// An invariant-bearing executable object can be preserved as a complete value.
// No representation or clone equation is inferred for that opaque object.
fn opaque_identity(t: &Typ, ds: &[Datatype]) -> bool {
    matches!(&**base(t),TypX::Datatype(dt,_,_) if ds.iter().any(|d|
        &d.x.name==dt && d.x.mode==Mode::Exec && d.x.user_defined_invariant_fn.is_some()))
}
fn option(t: &Typ) -> Option<&Typ> {
    match &**t {
        TypX::Datatype(Dt::Path(p), ts, _)
            if models::path_is(p, CrateId::Core, &["option", "Option"]) && ts.len() == 1 =>
        {
            Some(&ts[0])
        }
        _ => None,
    }
}
fn sequence_type(t: Typ) -> Typ {
    Arc::new(TypX::Datatype(
        Dt::Path(Arc::new(PathX {
            krate: CrateId::Vstd,
            segments: Arc::new(vec![Arc::new("seq".into()), Arc::new("Seq".into())]),
        })),
        Arc::new(vec![t]),
        Arc::new(vec![]),
    ))
}
fn heap_known(t: &Typ, all: &HashMap<Fun, Function>, ds: &[Datatype], depth: usize) -> bool {
    depth > 0
        && (collections::vec_args(base(t)).is_some()
            || sequence_views::find(base(t), all, ds).is_some()
            || option(t).is_some_and(|t| heap_known(t, all, ds, depth - 1))
            || sums::result(t).is_some_and(|(a, b)| {
                heap_known(a, all, ds, depth - 1) || heap_known(b, all, ds, depth - 1)
            })
            || records::shape::variants(base(t), ds).is_some_and(|vs|vs.iter().any(|(_,fs)|
                fs.iter().any(|(_,t)|heap_known(t,all,ds,depth-1))))
            || records::shape::fields(base(t), ds)
                .is_some_and(|fs| fs.iter().any(|(_, t)| heap_known(t, all, ds, depth - 1))))
}
fn result_query(f:&Function)->bool {
    records::candidate(f) && !records::mutable(f) && sums::result(&f.x.inner_ret.x.typ).is_some()
}
fn structured_return(f:&Function,ds:&[Datatype])->bool {
    matches!(f.x.kind,FunctionKind::Static)
        && (identity_borrow_return(f)
            || records::shape::fields(&f.x.inner_ret.x.typ,ds).is_some_and(|fs|
                fs.len()>1 || records::candidate(f) && records::shape::root_type(f).is_some_and(|t|
                    records::shape::fields(t,ds).is_some_and(|fs|fs.len()>1))))
}
pub(super) fn prepare(fs: &[Function], imported: &[Function], ds: &[Datatype]) -> Vec<Function> {
    let all=fs.iter().chain(imported).map(|f|(f.x.name.clone(),f.clone())).collect();
    fs.iter()
        .map(|f| {
            let mut x = f.x.clone();
            Arc::make_mut(&mut x.attrs).infer_owned =
                x.attrs.infer && (heap_known(&x.inner_ret.x.typ, &all, ds, 8) || result_query(f)
                    || structured_return(f,ds)
                    || sums::has_value_enum(&x.inner_ret.x.typ,ds,5)
                    || x.params.iter().any(|p|sums::has_value_enum(&p.x.typ,ds,5)) || iterators::uses(f,&all) || equality::uses(f, &all,ds) || result_map::uses(f,&all));
            f.new_x(x)
        })
        .collect()
}
#[derive(Clone)]
struct Val {
    typ: Typ,
    node: Node,
}
#[derive(Clone)]
enum Node {
    Atom(Term),
    Seq(Term),
    // Whether an earlier lazy filter has a partial callback. A subsequent
    // short-circuiting consumer needs a more precise safety model in that case.
    Iterator(Term, bool),
    Fields(Vec<(FieldOpr, Val)>),
    Option(Term, Option<Box<Val>>),
    Result(Term, Option<Box<Val>>, Option<Box<Val>>),
    Variants(Vec<variants::Branch>),
}
impl Val {
    fn atom(t: Term) -> Self {
        Self { typ: t.expr.typ.clone(), node: Node::Atom(t) }
    }
    fn scalar(&self, span: &Span) -> Result<Term, VirErr> {
        match &self.node {
            Node::Atom(t)=>Ok(t.clone()),
            Node::Fields(fs) => {
                let (op,_)=fs.first().ok_or_else(||unsupported(span,"empty value record"))?;
                let fields=fs.iter().map(|(p,v)|Ok(Arc::new(air::ast::BinderX{
                    name:p.field.clone(),a:v.scalar(span)?.expr,
                }))).collect::<Result<Vec<_>,VirErr>>()?;
                atom(SpannedTyped::new(span,&self.typ,ExprX::Ctor(op.datatype.clone(),op.variant.clone(),Arc::new(fields),None)))
            }
            Node::Result(ok,a,b)=>{
                let ctor=|variant:&str,v:&Val|->Result<Term,VirErr>{
                    let TypX::Datatype(dt,..)=&*self.typ else{return Err(unsupported(span,"Result value type"));};
                    let value=v.scalar(span)?;
                    term(span,&self.typ,ExprX::Ctor(dt.clone(),Arc::new(variant.into()),Arc::new(vec![Arc::new(air::ast::BinderX{
                        name:crate::def::positional_field_ident(0),a:value.expr,
                    })]),None),value.size+1)
                };
                if is_bool(ok,true){return ctor("Ok",a.as_ref().ok_or_else(||unsupported(span,"missing success value"))?);}
                if is_bool(ok,false){return ctor("Err",b.as_ref().ok_or_else(||unsupported(span,"missing error value"))?);}
                conditional(span,ok.clone(),ctor("Ok",a.as_ref().ok_or_else(||unsupported(span,"missing success value"))?)?,ctor("Err",b.as_ref().ok_or_else(||unsupported(span,"missing error value"))?)?)
            }
            _=>Err(unsupported(span, "owned scalar expression")),
        }
    }
}
#[derive(Clone)]
struct State {
    locals: HashMap<VarIdent, Val>,
    // Original immutable objects are separate from their logical sequence views.
    // An opaque checked query may observe the object, not just its contents.
    objects: HashMap<VarIdent, Term>,
    active: Term,
}
struct Engine<'a> {
    f: &'a Function,
    all: &'a HashMap<Fun, Function>,
    ds: &'a [Datatype],
    requirements: Vec<Term>,
    exits: Vec<(Term, Val)>,
    plans: Vec<loops::Plan>,
    callbacks: Vec<callbacks::Plan>,
    remaining: usize,
}
impl Engine<'_> {
    // Value-only results have a complete logical value. Keep that value intact
    // across calls/branches rather than duplicating it into tag and payload
    // observations. Results containing owned collections still need views.
    fn value_result(&self, typ: &Typ) -> bool {
        sums::result(typ).is_some_and(|(a, b)| {
            [a, b].iter().all(|t| records::shape::value_type(t, &self.f.x.typ_params, self.ds, 5))
        })
    }
    fn result_value(&self, span: &Span, typ: Typ, node: Node) -> Result<Val, VirErr> {
        let value = Val { typ, node };
        if self.value_result(&value.typ) {
            Ok(Val::atom(value.scalar(span)?))
        } else {
            Ok(value)
        }
    }
    fn object(&self,s:&State,e:&Expr)->Result<Term,VirErr>{
        fn place(s:&State,p:&Place,depth:usize)->Result<Term,VirErr>{
            if depth==0{return Err(unsupported(&p.span,"query receiver projection depth"));}
            match &p.x {
                PlaceX::Local(n)=>s.objects.get(n).cloned().or_else(||match &s.locals.get(n)?.node {
                    Node::Atom(t)=>Some(t.clone()),_=>None,
                }).ok_or_else(||unsupported(&p.span,"query receiver needs a complete immutable value")),
                PlaceX::Temporary(e)=>expr(s,e,depth-1),
                PlaceX::Field(op,base)=>{let v=place(s,base,depth-1)?;term(&p.span,&p.typ,ExprX::UnaryOpr(UnaryOpr::Field(op.clone()),v.expr),v.size+1)},
                _=>Err(unsupported(&p.span,"query receiver projection")),
            }
        }
        fn expr(s:&State,e:&Expr,depth:usize)->Result<Term,VirErr>{
            if depth==0{return Err(unsupported(&e.span,"query receiver projection depth"));}
            match &e.x {
                ExprX::Var(n)=>s.objects.get(n).cloned().or_else(||match &s.locals.get(n)?.node {
                    Node::Atom(t)=>Some(t.clone()),_=>None,
                }).ok_or_else(||unsupported(&e.span,"query receiver needs a complete immutable value")),
                ExprX::ReadPlace(p,_)=>place(s,p,depth-1),
                ExprX::UnaryOpr(UnaryOpr::Field(op),a)=>{let v=expr(s,a,depth-1)?;term(&e.span,&e.typ,ExprX::UnaryOpr(UnaryOpr::Field(op.clone()),v.expr),v.size+1)},
                ExprX::Block(ss,Some(tail)) if ss.is_empty()=>expr(s,tail,depth-1),
                _=>Err(unsupported(&e.span,"query receiver must borrow an original immutable object")),
            }
        }
        expr(s,e,8)
    }
    fn spend(&mut self, s: &Span) -> Result<(), VirErr> {
        self.remaining = self
            .remaining
            .checked_sub(1)
            .ok_or_else(|| unsupported(s, "owned source budget exceeded"))?;
        Ok(())
    }
    fn require(&mut self, s: &State, p: Term) -> Result<(), VirErr> {
        self.requirements.push(conditional(
            &self.f.span,
            s.active.clone(),
            p,
            boolean(&self.f.span, true),
        )?);
        Ok(())
    }
    fn lib(&self, s: &Span, m: Model, types: Vec<Typ>, args: Vec<Term>) -> Result<Term, VirErr> {
        let mut found = self.all.values().filter(|f| model(f) == Some(m));
        let f = found.next().ok_or_else(|| {
            unsupported(s, &format!("owned inference requires canonical collection model {m:?}"))
        })?;
        if found.next().is_some()
            || f.x.typ_params.len() != types.len()
            || f.x.params.len() != args.len()
        {
            return Err(unsupported(s, "ambiguous owned collection model"));
        }
        let ts = f.x.typ_params.iter().cloned().zip(types.iter().cloned()).collect();
        let args = args
            .into_iter()
            .zip(f.x.params.iter())
            .map(|(v, p)| {
                let typ = crate::sst_util::subst_typ(&ts, &p.x.typ);
                if matches!(&*typ, TypX::Int(_)) && matches!(&*v.expr.typ, TypX::Int(_)) {
                    Ok(v.expr)
                } else {
                    coerce(s, v.clone(), &typ).map(|t| t.expr).map_err(|_| {
                        unsupported(
                            s,
                            &format!("owned model argument {:?} expected {:?}", v.expr.typ, typ),
                        )
                    })
                }
            })
            .collect::<Result<Vec<_>, VirErr>>()?;
        atom(SpannedTyped::new(
            s,
            &crate::sst_util::subst_typ(&ts, &f.x.inner_ret.x.typ),
            ExprX::Call {
                target: CallTarget::Fun(
                    CallTargetKind::Static,
                    f.x.name.clone(),
                    Arc::new(types),
                    Arc::new(vec![]),
                    CallTargetAttrs {
                        autospec: AutospecUsage::Final,
                        const_var: false,
                        assume_external_allowed: false,
                    },
                ),
                args: Arc::new(args),
                post_args: None,
                body: None,
            },
        ))
    }
    fn standard(
        &self,
        s: &Span,
        m: collections::Model,
        types: Vec<Typ>,
        args: Vec<Term>,
    ) -> Result<Term, VirErr> {
        let t = records::Interpreter {
            function: self.f,
            shape: records::shape::Shape { typ: self.f.x.inner_ret.x.typ.clone(), fields: vec![] },
            all: self.all,
            datatypes: self.ds,
            requirements: vec![],
            remaining: FORMULA_BUDGET,
            plans: vec![],
        }
        .model_call(s, m, types, args)?;
        if m == collections::Model::SeqLen {
            return term(
                s,
                &int_typ(),
                ExprX::Unary(UnaryOp::Clip { range: IntRange::Int, truncate: false }, t.expr),
                t.size + 1,
            );
        }
        Ok(t)
    }
    fn observe(&self, e: &Expr) -> Result<Term, VirErr> {
        if let Some(args) = collections::vec_args(base(&e.typ)) {
            return self.standard(
                &e.span,
                collections::Model::View,
                (**args).clone(),
                vec![atom(e.clone())?],
            );
        }
        let view = sequence_views::find(base(&e.typ), self.all, self.ds).ok_or_else(|| {
            unsupported(&e.span, "owned private storage requires a certified direct view")
        })?;
        sequence_views::call(&view, e.clone())
    }
    fn layout(&self, e: Expr, depth: usize) -> Result<Val, VirErr> {
        if depth == 0 {
            return Err(unsupported(&e.span, "owned type depth exceeded"));
        }
        let typ = base(&e.typ).clone();
        let node = if collections::vec_args(&typ).is_some()
            || sequence_views::find(&typ, self.all, self.ds).is_some()
        {
            Node::Seq(self.observe(&e)?)
        } else if let Some(payload) = option(&typ).filter(|t| heap_known(t, self.all, self.ds, 5)) {
            let tag = atom(SpannedTyped::new(
                &e.span,
                &bool_typ(),
                ExprX::UnaryOpr(
                    UnaryOpr::IsVariant {
                        datatype: match &*typ {
                            TypX::Datatype(dt, ..) => dt.clone(),
                            _ => unreachable!(),
                        },
                        variant: Arc::new("Some".into()),
                    },
                    e.clone(),
                ),
            ))?;
            Node::Option(
                tag,
                Some(Box::new(
                    self.layout(field_expr(&e, &option_field(&typ), payload), depth - 1)?,
                )),
            )
        } else if self.value_result(&typ) {
            Node::Atom(atom(e)?)
        } else if let Some((ok, err)) = sums::result(&typ) {
            Node::Result(
                atom(sums::tag(&e, "Ok"))?,
                Some(Box::new(
                    self.layout(field_expr(&e, &sums::field(&typ, "Ok"), ok), depth - 1)?,
                )),
                Some(Box::new(
                    self.layout(field_expr(&e, &sums::field(&typ, "Err"), err), depth - 1)?,
                )),
            )
        } else if let Some(node) = self.variants_layout(&e, depth)? {
            node
        } else if opaque_identity(&typ, self.ds) {
            Node::Atom(atom(e)?)
        } else if let Some(fields) = records::shape::fields(&typ, self.ds) {
            Node::Fields(
                fields
                    .into_iter()
                    .map(|(op, t)| {
                        Ok((op.clone(), self.layout(field_expr(&e, &op, &t), depth - 1)?))
                    })
                    .collect::<Result<_, VirErr>>()?,
            )
        } else {
            if !records::shape::value_type(&e.typ, &self.f.x.typ_params, self.ds, 5)
                && !sums::flat_enum(&typ, self.ds)
            {
                return Err(unsupported(&e.span, "unsupported owned input/output leaf"));
            }
            Node::Atom(atom(e)?)
        };
        Ok(Val { typ, node })
    }
    fn field(&self, v: Val, op: &FieldOpr, s: &Span) -> Result<Val, VirErr> {
        match v.node {
            Node::Fields(fs) => fs
                .into_iter()
                .find(|(p, _)| p.field == op.field && p.datatype == op.datatype)
                .map(|(_, v)| v)
                .ok_or_else(|| unsupported(s, "owned field")),
            Node::Atom(t) => {
                let fs = records::constructor_fields(s, &v.typ, &op.datatype, &op.variant, self.ds)
                    .or_else(|err| {
                        if !opaque_identity(&v.typ,self.ds) {return Err(err);}
                        let TypX::Datatype(dt,args,..)=&*v.typ else{return Err(err);};
                        let d=self.ds.iter().find(|d|&d.x.name==dt).ok_or_else(||err.clone())?;
                        if &op.datatype!=dt || d.x.variants.len()!=1 || op.variant!=d.x.variants[0].name {
                            return Err(err);
                        }
                        let ts=d.x.typ_params.iter().zip(args.iter()).map(|(p,t)|(p.0.clone(),t.clone())).collect();
                        Ok(d.x.variants[0].fields.iter().filter(|f|f.a.1==Mode::Exec).map(|f|
                            (f.name.clone(),crate::sst_util::subst_typ(&ts,&f.a.0))).collect())
                    })?;
                let typ = fs
                    .into_iter()
                    .find(|(n, _)| n == &op.field)
                    .ok_or_else(|| unsupported(s, "owned scalar field"))?
                    .1;
                // Keep complete value equations compact when a helper returns
                // a record constructor. Reading one field must not substitute
                // every sibling (and their entire caller expressions) again.
                let mut whole=&t.expr;
                while let ExprX::ReadPlace(p,_)=&whole.x {
                    if let PlaceX::Temporary(inner)=&p.x {whole=inner;} else {break;}
                }
                let e=if let ExprX::Ctor(dt,variant,fs,None)=&whole.x {
                    if dt==&op.datatype && variant==&op.variant {
                        fs.iter().find(|f|f.name==op.field).map(|f|f.a.clone())
                            .ok_or_else(||unsupported(s,"missing complete constructor field"))?
                    } else {field_expr(&t.expr,op,&typ)}
                } else {field_expr(&t.expr,op,&typ)};
                if heap_known(&typ,self.all,self.ds,8) {self.layout(e,8)} else {Ok(Val::atom(atom(e)?))}
            }
            Node::Seq(t) => {
                let view = sequence_views::find(&v.typ, self.all, self.ds)
                    .ok_or_else(|| unsupported(s, "owned physical vector field"))?;
                if !records::shape::same_path(&[op.clone()], &[view.storage]) {
                    return Err(unsupported(s, "owned vector field mismatch"));
                }
                let typ = records::shape::fields(&v.typ, self.ds)
                    .and_then(|fs| {
                        fs.into_iter().find(|(p, _)| p.field == op.field).map(|(_, t)| t)
                    })
                    .ok_or_else(|| unsupported(s, "owned vector storage unavailable"))?;
                Ok(Val { typ, node: Node::Seq(t) })
            }
            Node::Variants(vs)=>vs.into_iter().find(|(v,_,_)|v==&op.variant)
                .and_then(|(_,_,fs)|fs).and_then(|fs|fs.into_iter().find(|(field,_)|
                    field.datatype==op.datatype && field.field==op.field)).map(|(_,v)|v)
                .ok_or_else(||unsupported(s,"structured enum field outside a present payload")),
            _ => Err(unsupported(s, "owned field projection")),
        }
    }
    fn place(&mut self, s: &mut State, p: &Place) -> Result<Val, VirErr> {
        self.spend(&p.span)?;
        match &p.x {
            PlaceX::Local(n) => {
                s.locals.get(n).cloned().ok_or_else(|| unsupported(&p.span, "unbound owned local"))
            }
            PlaceX::Temporary(e) => self.expr(s, e),
            PlaceX::Field(op, b) => {
                let v = self.place(s, b)?;
                self.field(v, op, &p.span)
            }
            _ => Err(unsupported(&p.span, "owned place or alias")),
        }
    }
    fn binary(
        &mut self,
        s: &State,
        span: &Span,
        op: BinaryOp,
        a: Term,
        b: Term,
    ) -> Result<Term, VirErr> {
        let norm = |t: Term| {
            let typ = base(&t.expr.typ).clone();
            coerce(span, t, &typ)
        };
        let a = norm(a)?;
        let b = norm(b)?;
        let (math, range, div) = eligibility::source_binary(op)
            .ok_or_else(|| unsupported(span, "owned scalar operator"))?;
        if div {
            return Err(unsupported(span, "owned division/remainder"));
        }
        let t = summary::binary(span, math, a, b)?;
        if let Some(r) = range {
            self.require(
                s,
                logical(
                    span,
                    LogicalOp::And,
                    summary::binary(
                        span,
                        BinaryOp::Inequality(InequalityOp::Le),
                        arithmetic::minimum(span, r)?,
                        t.clone(),
                    )?,
                    summary::binary(
                        span,
                        BinaryOp::Inequality(InequalityOp::Le),
                        t.clone(),
                        arithmetic::maximum(span, r)?,
                    )?,
                )?,
            )?;
            return coerce(span, t, &Arc::new(TypX::Int(r)));
        }
        Ok(t)
    }
    fn expr(&mut self, s: &mut State, e: &Expr) -> Result<Val, VirErr> {
        self.spend(&e.span)?;
        if let Some((negate, args)) = equality::source_call(e, self.all) {
            let a = self.expr(s, &args[0])?.scalar(&e.span)?;
            let b = self.expr(s, &args[1])?.scalar(&e.span)?;
            let a = coerce(&e.span, a, &base(&args[0].typ))?;
            let b = coerce(&e.span, b, &base(&args[1].typ))?;
            let value = summary::binary(&e.span, BinaryOp::Eq(Mode::Spec), a, b)?;
            return Ok(Val::atom(if negate { exits::not(&e.span, value)? } else { value }));
        }
        if let Some(source)=native_for::source(e){return self.native_for(s,&source);}
        let span = &e.span;
        if let Some(args) = result_map::call(e, self.all) { return self.map_error(s, e, &args); }
        if let Some((spec,types,args))=iterators::source_call(e,self.all) {
            let values=args.iter().enumerate().map(|(i,a)|{
                if i==0 {self.object(s,a)}else{self.expr(s,a)?.scalar(&a.span)}
            }).collect::<Result<Vec<_>,_>>()?;
            let value=trait_getters::instantiate(&spec,&types,values,&self.f.x.typ_bounds)?;
            return Ok(Val{typ:e.typ.clone(),node:Node::Iterator(value,false)});
        }
        if let Some((op,args))=iterators::operation(e,self.all) {
            let receiver=match &args[0].x {
                ExprX::BorrowMut(p)|ExprX::TwoPhaseBorrowMut(p)=>{
                    let PlaceX::Temporary(e)=&p.x else{return Err(unsupported(span,"native search currently consumes a temporary iterator"));};e
                }
                _=>&args[0],
            };
            let value=self.expr(s,receiver)?;
            let Node::Iterator(sequence,partial)=value.node else{return Err(unsupported(span,"native consumers require a certified finite iterator factory"));};
            if op==iterators::Operation::Any && partial {return Err(unsupported(span,"native short-circuit search after a partial filter"));}
            let (sequence,new_partial)=if op==iterators::Operation::Count{(sequence,false)}else{self.predicate(s,&args[1],sequence,op==iterators::Operation::Any)?};
            if op==iterators::Operation::Filter{return Ok(Val{typ:e.typ.clone(),node:Node::Iterator(sequence,partial||new_partial)});}
            let item=iterators::sequence(&sequence.expr.typ).ok_or_else(||unsupported(span,"native iterator observation type"))?.clone();
            let len=self.standard(span,collections::Model::SeqLen,vec![item],vec![sequence])?;
            if op==iterators::Operation::Any{
                let zero=term(span,&int_typ(),ExprX::Const(Constant::Int(0.into())),1)?;
                return Ok(Val::atom(summary::binary(span,BinaryOp::Inequality(InequalityOp::Lt),zero,len)?));
            }
            return Ok(Val::atom(len));
        }
        if panics::literal(e,self.all).is_some() {
            self.require(s,boolean(span,false))?;
            s.active=boolean(span,false);
            return Ok(Val::atom(exits::unit_value(span)?));
        }
        if let Some((_query,spec,types,args))=trait_getters::source_call(e,self.all) {
            if self.object(s, &args[0]).is_err() && args.len() == 1 {
                let value = self.expr(s, &args[0])?;
                if let Node::Seq(sequence) = value.node {
                    return Ok(Val::atom(trait_getters::specialize_view(&spec, &types, sequence, self.all, self.f)?));
                }
                return Err(unsupported(span, "constructed query receiver requires a direct sequence view"));
            }
            let args=args.iter().map(|a|{
                if let Ok(v)=self.object(s,a){return Ok(v);}
                self.expr(s,a)?.scalar(&a.span)
            }).collect::<Result<Vec<_>,_>>()?;
            return Ok(Val::atom(trait_getters::instantiate_source(&spec,&types,args,&self.f.x.typ_bounds,e)?));
        }
        if let Some(p)=propagation::recognize(e,self.all,&self.f.x.inner_ret.x.typ)? {
            return self.propagate(s,e,p);
        }
        if let Some(t) = arithmetic::arch_constant(e) {
            return Ok(Val::atom(t));
        }
        match &e.x {
            ExprX::Const(Constant::Bool(_) | Constant::Int(_)) | ExprX::Var(_) => {
                if let ExprX::Var(n) = &e.x {
                    s.locals.get(n).cloned().ok_or_else(|| unsupported(span, "owned variable"))
                } else {
                    Ok(Val::atom(atom(e.clone())?))
                }
            }
            ExprX::ReadPlace(p, _) => self.place(s, p),
            ExprX::UnaryOpr(UnaryOpr::Field(op), a) => {
                let v = self.expr(s, a)?;
                self.field(v, op, span)
            }
            ExprX::UnaryOpr(op @ UnaryOpr::IsVariant { .. }, a) => {
                let value = self.expr(s, a)?.scalar(span)?;
                let UnaryOpr::IsVariant { datatype, variant } = op else { unreachable!() };
                Ok(Val::atom(variant_test(span, value, datatype, variant)?))
            }
            ExprX::Unary(op @ UnaryOp::Clip { .. }, a) => {
                let a = self.expr(s, a)?.scalar(span)?;
                Ok(Val::atom(summary::unary(span, *op, a)?))
            }
            ExprX::Unary(UnaryOp::Not, a) => {
                let a = self.expr(s, a)?.scalar(span)?;
                Ok(Val::atom(exits::not(span, a)?))
            }
            ExprX::Binary(op, a, b) => {
                let a = self.expr(s, a)?.scalar(span)?;
                let b = self.expr(s, b)?.scalar(span)?;
                Ok(Val::atom(self.binary(s, span, *op, a, b)?))
            }
            ExprX::Logical(op, a, b) => {
                let a = self.expr(s, a)?.scalar(span)?;
                let mut branch = s.clone();
                branch.active = logical(
                    span,
                    LogicalOp::And,
                    s.active.clone(),
                    if *op == LogicalOp::Or { exits::not(span, a.clone())? } else { a.clone() },
                )?;
                let b = self.expr(&mut branch, b)?.scalar(span)?;
                Ok(Val::atom(logical(span, *op, a, b)?))
            }
            ExprX::Ctor(dt, variant, fs, None) => {
                if variants::has_heap_enum(&e.typ,self.all,self.ds,8) && records::shape::variants(&e.typ,self.ds).is_some() {
                    return self.variants_ctor(s,e,variant,fs);
                }
                if sums::result(&e.typ).is_some() {
                    if fs.len() != 1 || !matches!(variant.as_str(), "Ok" | "Err") {
                        return Err(unsupported(span, "Result constructor shape"));
                    }
                    let value = Some(Box::new(self.expr(s, &fs[0].a)?));
                    return self.result_value(
                        span,
                        e.typ.clone(),
                        if variant.as_str() == "Ok" {
                            Node::Result(boolean(span, true), value, None)
                        } else {
                            Node::Result(boolean(span, false), None, value)
                        },
                    );
                }
                if sums::flat_enum(&e.typ, self.ds) && fs.is_empty() {
                    return Ok(Val::atom(atom(e.clone())?));
                }

                if let Some(payload) =
                    option(&e.typ).filter(|t| heap_known(t, self.all, self.ds, 5))
                {
                    let _ = payload;
                    let value = if variant.as_str() == "None" && fs.is_empty() {
                        None
                    } else if variant.as_str() == "Some" && fs.len() == 1 {
                        Some(Box::new(self.expr(s, &fs[0].a)?))
                    } else {
                        return Err(unsupported(span, "owned Option constructor"));
                    };
                    return Ok(Val {
                        typ: e.typ.clone(),
                        node: Node::Option(boolean(span, value.is_some()), value),
                    });
                }
                if let Some(view) = sequence_views::find(&e.typ, self.all, self.ds) {
                    if fs.len() != 1 || fs[0].name != view.storage.field {
                        return Err(unsupported(span, "owned view constructor"));
                    }
                    let mut v = self.expr(s, &fs[0].a)?;
                    v.typ = e.typ.clone();
                    return Ok(v);
                }
                let expected = records::constructor_fields(span, &e.typ, dt, variant, self.ds)?;
                if fs.len() != expected.len() {
                    return Err(unsupported(span, "owned constructor field coverage"));
                }
                let values = expected
                    .into_iter()
                    .map(|(name, t)| {
                        let f = fs
                            .iter()
                            .find(|f| f.name == name)
                            .ok_or_else(|| unsupported(span, "owned field missing"))?;
                        let op = FieldOpr {
                            datatype: dt.clone(),
                            variant: variant.clone(),
                            field: name,
                            get_variant: false,
                            check: VariantCheck::None,
                        };
                        Ok((op, self.expr(s, &f.a)?, t))
                    })
                    .collect::<Result<Vec<_>, VirErr>>()?;
                if records::shape::fields(&e.typ, self.ds).is_some() {
                    Ok(Val {
                        typ: e.typ.clone(),
                        node: Node::Fields(values.into_iter().map(|(p, v, _)| (p, v)).collect()),
                    })
                } else {
                    let fs = values
                        .into_iter()
                        .map(|(p, v, t)| {
                            Ok(Arc::new(air::ast::BinderX {
                                name: p.field,
                                a: coerce(span, v.scalar(span)?, &t)?.expr,
                            }))
                        })
                        .collect::<Result<Vec<_>, VirErr>>()?;
                    Ok(Val::atom(atom(e.new_x(ExprX::Ctor(
                        dt.clone(),
                        variant.clone(),
                        Arc::new(fs),
                        None,
                    )))?))
                }
            }
            ExprX::Block(stmts, tail) => {
                for st in stmts.iter() {
                    self.spend(&st.span)?;
                    match &st.x {
                        StmtX::Decl {
                            pattern,
                            mode: None | Some((Mode::Exec, DeclProph::Default)),
                            init: Some(init),
                            els: None,
                            assert_irrefutable: false,
                        } => {
                            let v = self.place(s, init)?;
                            match &pattern.x {
                                PatternX::Var(b) if matches!(b.by_ref, ByRef::No) => {
                                    if b.user_mut == Some(true)
                                        && !matches!(v.node, Node::Seq(_))
                                        && !same(&b.typ, &Arc::new(TypX::Int(IntRange::USize)))
                                    {
                                        return Err(unsupported(span, "owned mutable local"));
                                    }
                                    if b.user_mut!=Some(true) {
                                        if let Ok(origin)=self.object(s,&crate::ast_util::place_to_spec_expr(init)) {
                                            s.objects.insert(b.name.clone(),origin);
                                        } else if let Ok(complete)=v.scalar(span) {
                                            s.objects.insert(b.name.clone(),complete);
                                        }
                                    }
                                    s.locals.insert(b.name.clone(), v);
                                }
                                PatternX::Wildcard => {}
                                _ => {let test=self.pattern(s,pattern,v)?;if !is_bool(&test,true){return Err(unsupported(span,"owned binding must be irrefutable"));}},
                            }
                        }
                        StmtX::Expr(e) => {
                            self.expr(s, e)?;
                        }
                        _ => return Err(unsupported(&st.span, "owned statement")),
                    }
                }
                if let Some(tail) = tail {
                    self.expr(s, tail)
                } else {
                    Ok(Val::atom(exits::unit_value(span)?))
                }
            }
            ExprX::If(c, a, b) => {
                let c = self.expr(s, c)?.scalar(span)?;
                let mut left = s.clone();
                left.active = logical(span, LogicalOp::And, s.active.clone(), c.clone())?;
                let a = self.expr(&mut left, a)?;
                let mut right = s.clone();
                right.active =
                    logical(span, LogicalOp::And, s.active.clone(), exits::not(span, c.clone())?)?;
                let b = if let Some(b) = b {
                    self.expr(&mut right, b)?
                } else {
                    Val::atom(exits::unit_value(span)?)
                };
                s.active = logical(span, LogicalOp::Or, left.active, right.active)?;
                choose(span, c, a, b)
            }
            ExprX::Return(Some(v)) => {
                let v = self.expr(s, v)?;
                self.exits.push((s.active.clone(), v));
                s.active = boolean(span, false);
                Ok(Val::atom(exits::unit_value(span)?))
            }
            ExprX::Match(value, arms, _) => self.match_expr(s, e, value, arms),
            ExprX::NeverToAny(e) => self.expr(s, e),
            ExprX::Loop { .. } => self.loop_expr(s, e),
            ExprX::Call {
                target: CallTarget::Fun(_, name, types, ..),
                args,
                post_args: None,
                body: None,
            } => {
                if let Some(m @ (collections::Model::SeqLen|collections::Model::SeqIndex|collections::Model::SeqUpdate)) = self.all.get(name).and_then(collections::model) {
                    let values =
                        args.iter().map(|a| self.formula(s, a)).collect::<Result<Vec<_>, _>>()?;
                    return Ok(Val::atom(self.standard(
                        span,
                        m,
                        (**types).clone(),
                        values,
                    )?));
                }
                if self.all.get(name).is_some_and(record_models::as_ref) && args.len() == 1 {
                    let value = self.expr(s, &args[0])?.scalar(span)?;
                    let typ = base(&value.expr.typ).clone();
                    let value = coerce(span, value, &typ)?;
                    let payload = option(&typ)
                        .ok_or_else(|| unsupported(span, "borrowed Option model input"))?;
                    let target = option(&e.typ)
                        .ok_or_else(|| unsupported(span, "borrowed Option model output"))?;
                    let field = coerce(
                        span,
                        atom(field_expr(&value.expr, &option_field(&typ), payload))?,
                        target,
                    )?;
                    let TypX::Datatype(dt, ..) = &*e.typ else { unreachable!() };
                    let some = term(
                        span,
                        &e.typ,
                        ExprX::Ctor(
                            dt.clone(),
                            Arc::new("Some".into()),
                            Arc::new(vec![Arc::new(air::ast::BinderX {
                                name: crate::def::positional_field_ident(0),
                                a: field.expr,
                            })]),
                            None,
                        ),
                        field.size + 1,
                    )?;
                    let none = term(
                        span,
                        &e.typ,
                        ExprX::Ctor(dt.clone(), Arc::new("None".into()), Arc::new(vec![]), None),
                        1,
                    )?;
                    return Ok(Val::atom(conditional(
                        span,
                        atom(tag_expr(&value.expr))?,
                        some,
                        none,
                    )?));
                }
                if cloning::call(e, self.all) {
                    let mut v = self.expr(s, &args[0])?;
                    if let Node::Atom(t)=&v.node {
                        if records::shape::fields(base(&v.typ),self.ds).is_some() {
                            v=self.layout(t.expr.clone(),8)?;
                        }
                    }
                    if !matches!(v.node, Node::Seq(_) | Node::Fields(_)) {
                        return Err(unsupported(
                            span,
                            "native Clone inference requires a certified sequence view",
                        ));
                    }
                    return Ok(Val { typ: e.typ.clone(), node: v.node });
                }
                if let Some(m) = model_call(e, self.all) {
                    return self.collection(s, e, m, args);
                }
                if let Some(arg) = widening::call(e, self.all) {
                    let a = self.expr(s, arg)?.scalar(span)?;
                    return Ok(Val::atom(coerce(span, a, &e.typ)?));
                }
                if self.all.get(name).and_then(collections::model) == Some(collections::Model::Len)
                    && args.len() == 1
                {
                    let v = self.expr(s, &args[0])?;
                    if let Node::Seq(seq) = v.node {
                        return Ok(Val::atom(self.standard(
                            span,
                            collections::Model::SeqLen,
                            vec![element(&v.typ, self.all, self.ds)?],
                            vec![seq],
                        )?));
                    }
                }
                if self.all.get(name).and_then(collections::model)
                    == Some(collections::Model::Index)
                    && args.len() == 2
                {
                    let v = self.expr(s, &args[0])?;
                    let i = self.expr(s, &args[1])?.scalar(span)?;
                    if let Node::Seq(seq) = v.node {
                        let t = element(&v.typ, self.all, self.ds)?;
                        let len = self.standard(
                            span,
                            collections::Model::SeqLen,
                            vec![t.clone()],
                            vec![seq.clone()],
                        )?;
                        self.require(
                            s,
                            summary::binary(
                                span,
                                BinaryOp::Inequality(InequalityOp::Lt),
                                i.clone(),
                                len,
                            )?,
                        )?;
                        return Ok(Val::atom(self.standard(
                            span,
                            collections::Model::SeqIndex,
                            vec![t],
                            vec![seq, i],
                        )?));
                    }
                }
                let callee = self
                    .all
                    .get(name)
                    .filter(|f| {
                        f.x.attrs.infer
                            && f.x.params.len() == args.len()
                            && f.x.typ_params.len() == types.len()
                    })
                    .ok_or_else(|| {
                        unsupported(span, "owned calls require checked inferred helpers")
                    })?
                    .clone();
                self.helper(s, e, &callee, types, args)
            }
            _ => Err(unsupported(
                span,
                "owned expression outside the bounded construction/copy subset",
            )),
        }
    }
    fn collection(
        &mut self,
        s: &mut State,
        e: &Expr,
        m: Model,
        args: &Exprs,
    ) -> Result<Val, VirErr> {
        let span = &e.span;
        match m {
            Model::New | Model::Capacity => {
                for a in args.iter() {
                    self.expr(s, a)?;
                }
                let t = element(&e.typ, self.all, self.ds)?;
                Ok(Val {
                    typ: e.typ.clone(),
                    node: Node::Seq(self.lib(span, Model::Empty, vec![t], vec![])?),
                })
            }
            Model::Repeat => {
                if args.len() != 2 {
                    return Err(unsupported(span, "repeat arguments"));
                }
                let v = self.expr(s, &args[0])?.scalar(span)?;
                let n = self.expr(s, &args[1])?.scalar(span)?;
                if !clone_identity(&v.expr.typ)
                    && !matches!(&v.expr.x,ExprX::Ctor(_,v,fs,None) if v.as_str()=="None" && fs.is_empty())
                {
                    return Err(unsupported(
                        span,
                        "repeat requires checked identity cloning or None",
                    ));
                }
                let seq = self.repeat(span, n, v)?;
                Ok(Val { typ: e.typ.clone(), node: Node::Seq(seq) })
            }
            Model::Clone => {
                if args.len() != 1 {
                    return Err(unsupported(span, "clone arguments"));
                }
                let v = self.expr(s, &args[0])?;
                if !clone_identity(&element(&v.typ, self.all, self.ds)?) {
                    return Err(unsupported(
                        span,
                        "generic Clone does not establish value equality; use a verified copy loop",
                    ));
                }
                Ok(Val { typ: e.typ.clone(), node: v.node })
            }
            _ => {
                Err(unsupported(span, "collection calls must occur in a checked fill/copy pattern"))
            }
        }
    }
    fn repeat(&self, span: &Span, n: Term, value: Term) -> Result<Term, VirErr> {
        let typ = Arc::new(TypX::SpecFn(Arc::new(vec![int_typ()]), value.expr.typ.clone()));
        let name =
            VarIdent(Arc::new("__verus_infer_repeat_index".into()), VarIdentDisambiguate::AirLocal);
        let lambda = atom(SpannedTyped::new(
            span,
            &typ,
            ExprX::Closure(
                Arc::new(vec![Arc::new(VarBinderX { name, a: int_typ() })]),
                value.expr.clone(),
            ),
        ))?;
        self.lib(span, Model::SeqNew, vec![value.expr.typ.clone()], vec![n, lambda])
    }
    fn helper(
        &mut self,
        s: &mut State,
        e: &Expr,
        f: &Function,
        types: &Typs,
        args: &Exprs,
    ) -> Result<Val, VirErr> {
        crate::infer_proof::CandidateId::from_function(f)?;
        if f.x.attrs.infer_allow_approx || f.x.require.len() != 1 || f.x.ensure.0.len() != 1 {
            return Err(unsupported(&e.span, "owned helpers require exact summaries"));
        }
        if records::mutable(f){return self.mutable_sequence_helper(s,e,f,types,args);}
        let values=args.iter().map(|a|Ok((self.expr(s,a)?,self.object(s,a).ok())))
            .collect::<Result<Vec<_>,VirErr>>()?;
        self.helper_values(s,e,f,types,values)
    }
    fn helper_values(
        &mut self, s:&mut State, e:&Expr, f:&Function, types:&Typs,
        values:Vec<(Val,Option<Term>)>,
    )->Result<Val,VirErr>{
        crate::infer_proof::CandidateId::from_function(f)?;
        if f.x.attrs.infer_allow_approx || f.x.require.len() != 1 || f.x.ensure.0.len() != 1
            || f.x.params.len()!=values.len() || f.x.typ_params.len()!=types.len() {
            return Err(unsupported(&e.span, "owned helpers require exact summaries"));
        }
        let ts: HashMap<Ident, Typ> =
            f.x.typ_params.iter().cloned().zip(types.iter().cloned()).collect();
        let subst = |expr: &Expr| {
            crate::ast_visitor::map_expr_visitor_env(
                expr,
                &mut crate::ast_visitor::VisitorScopeMap::new(),
                &mut (),
                &|_, _, e| Ok(e.clone()),
                &|_, _, st| Ok(vec![st.clone()]),
                &|_, t| {
                    Ok(if let TypX::TypParam(n) = &**t {
                        ts.get(n).cloned().unwrap_or_else(|| t.clone())
                    } else {
                        t.clone()
                    })
                },
                &|_, _, p| Ok(p.clone()),
                &|_, _, p| Ok(p.clone()),
            )
        };
        let mut local = State { locals: HashMap::new(), objects:HashMap::new(), active: s.active.clone() };
        for (p, (value,origin)) in f.x.params.iter().zip(values) {
            if let Some(origin)=origin {local.objects.insert(p.x.name.clone(),origin);}
            else if let Node::Atom(t)=&value.node {local.objects.insert(p.x.name.clone(),t.clone());}
            local.locals.insert(p.x.name.clone(), value);
        }
        let pre = self.formula(&mut local, &subst(&f.x.require[0])?)?;
        self.require(s, pre)?;
        if identity_borrow_return(f) {
            let post=subst(&f.x.ensure.0[0])?;
            let rhs=find_rhs(&SpannedTyped::new(&f.span,&crate::sst_util::subst_typ(&ts,&f.x.inner_ret.x.typ),
                ExprX::Var(f.x.inner_ret.x.name.clone())),&[post])?;
            return Ok(Val::atom(self.formula(&local,&rhs)?));
        }
        if !f.x.attrs.infer_owned {
            let post = subst(&f.x.ensure.0[0])?;
            let ExprX::Binary(BinaryOp::Eq(Mode::Spec), lhs, rhs) = &post.x else {
                return Err(unsupported(&e.span, "owned value helper relation"));
            };
            if !matches!(&lhs.x,ExprX::Var(n) if n==&f.x.inner_ret.x.name) {
                return Err(unsupported(&e.span, "owned value helper output"));
            }
            return self.formula_value(&mut local, rhs);
        }
        let ret = crate::sst_util::subst_typ(&ts, &f.x.inner_ret.x.typ);
        let root = SpannedTyped::new(&f.span, &ret, ExprX::Var(f.x.inner_ret.x.name.clone()));
        let layout = self.layout(root.clone(), 8)?;
        let post = subst(&f.x.ensure.0[0])?;
        let mut clauses = vec![];
        split(&post, &mut clauses);
        self.decode_parts(&mut local, &root, &layout, &clauses)
    }
    fn formula(&self, s: &State, e: &Expr) -> Result<Term, VirErr> {
        if let Some(value)=arithmetic::arch_constant(e) {return Ok(value);}
        // Summary calls are pure and use only existing sequence/view models.
        if let ExprX::Call {
            target: CallTarget::Fun(kind, name, types, ..),
            args,
            post_args: None,
            body: None,
        } = &e.x
        {
            let resolved = if let CallTargetKind::DynamicResolved { resolved, .. } = kind {
                resolved
            } else {
                name
            };
            if let Some(spec)=self.all.get(name).filter(|f|f.x.attrs.infer_trait_observation && trait_getters::observation(f)
                || f.x.attrs.infer_iterator_observation && iterators::observation(f)) {
                let values=args.iter().map(|a|self.object(s,a).or_else(|_|self.formula(s,a))).collect::<Result<Vec<_>,_>>()?;
                return trait_getters::instantiate(spec,types,values,&self.f.x.typ_bounds);
            }
            if self.all.get(resolved).is_some_and(|f| f.x.attrs.infer_sequence_view)
                || self.all.get(resolved).and_then(collections::model)
                    == Some(collections::Model::View)
            {
                let v = self.formula_value(s, &args[0])?;
                if let Node::Seq(t) = v.node {
                    return Ok(t);
                }
            }
            if let Some(m @ (collections::Model::SeqLen|collections::Model::SeqIndex|collections::Model::SeqUpdate))=self.all.get(name).and_then(collections::model) {
                let values=args.iter().map(|a|self.formula(s,a)).collect::<Result<Vec<_>,_>>()?;
                return self.standard(&e.span,m,(**types).clone(),values);
            }
            if model(self.all.get(name).ok_or_else(|| unsupported(&e.span, "owned summary model"))?)
                == Some(Model::SeqNew)
            {
                let n = self.formula(s, &args[0])?;
                let ExprX::Closure(_, body) = &args[1].x else {
                    return Err(unsupported(&e.span, "owned repeat summary"));
                };
                let v = self.formula(s, body)?;
                return self.repeat(&e.span, n, v);
            }
            if let Some(m @ (Model::Empty | Model::Take | Model::SeqPush | Model::SeqFilter | Model::SeqFold | Model::Scatter)) =
                self.all.get(name).and_then(model)
            {
                let values =
                    args.iter().map(|a| self.formula(s, a)).collect::<Result<Vec<_>, _>>()?;
                return self.lib(&e.span, m, (**types).clone(), values);
            }
            if self.all.get(name).is_some_and(is_pure_observation) {
                let whole=self.all.get(name).is_some_and(borrowed::is_value);
                let values = args.iter().map(|a| {
                    if whole {self.object(s,a).or_else(|_|self.formula(s,a))} else {self.formula(s,a)}
                }.map(|v|v.expr)).collect::<Result<Vec<_>, _>>()?;
                let ExprX::Call { target, .. } = &e.x else { unreachable!() };
                return atom(e.new_x(ExprX::Call { target: target.clone(), args: Arc::new(values), post_args: None, body: None }));
            }
        }
        match &e.x {
            ExprX::Const(Constant::Bool(_)|Constant::Int(_))=>atom(e.clone()),
            ExprX::Var(_)|ExprX::ReadPlace(_,_)
                |ExprX::UnaryOpr(UnaryOpr::Field(_),_)=>self.formula_value(s,e)?.scalar(&e.span),
            ExprX::UnaryOpr(UnaryOpr::IsVariant{datatype,variant},a)=>{
                let value=self.formula_value(s,a)?;
                if let Node::Variants(vs)=&value.node {
                    return vs.iter().find(|(v,_,_)|v==variant).map(|(_,tag,_)|tag.clone())
                        .ok_or_else(||unsupported(&e.span,"pure enum tag"));
                }
                variant_test(&e.span,value.scalar(&e.span)?,datatype,variant)
            }
            ExprX::Block(ss,Some(tail)) if ss.is_empty()=>self.formula(s,tail),
            ExprX::Ctor(..) => self.formula_value(s, e)?.scalar(&e.span),
            ExprX::Closure(params,body)|ExprX::Quant(_,params,body)=>{
                if params.is_empty() || params.len()>2{return Err(unsupported(&e.span,"native summary binder budget"));}
                let mut local=s.clone();
                for p in params.iter(){
                    local.locals.insert(p.name.clone(),self.layout(SpannedTyped::new(&e.span,&p.a,ExprX::Var(p.name.clone())),5)?);
                }
                let value=self.formula(&mut local,body)?;
                let x=match &e.x {ExprX::Closure(..)=>ExprX::Closure(params.clone(),value.expr),ExprX::Quant(q,..)=>ExprX::Quant(*q,params.clone(),value.expr),_=>unreachable!()};
                term(&e.span,&e.typ,x,value.size+1)
            }
            ExprX::Logical(op, a, b) => {
                let a = self.formula(s, a)?;
                let b = self.formula(s, b)?;
                logical(&e.span, *op, a, b)
            }
            ExprX::Binary(op, a, b) => {
                let a = self.formula(s, a)?;
                let b = self.formula(s, b)?;
                // Borrowed value payloads use the same logical value as their
                // copied scalar. Substitution must retain this normalization.
                let a = coerce(&e.span, a.clone(), base(&a.expr.typ))?;
                let b = coerce(&e.span, b.clone(), base(&b.expr.typ))?;
                summary::binary(&e.span, *op, a, b)
            }
            ExprX::If(c, a, Some(b)) => {
                let c = self.formula(s, c)?;
                let a = self.formula(s, a)?;
                let b = self.formula(s, b)?;
                conditional(&e.span, c, a, b)
            }
            ExprX::Unary(UnaryOp::Clip { range: IntRange::Int, .. }, a) => self.formula(s, a),
            ExprX::Unary(op, a) => {
                let a = self.formula(s, a)?;
                summary::unary(&e.span, *op, a)
            }
            _ => Err(unsupported(&e.span,"invalid pure owned summary expression")),
        }
    }
    fn formula_value(&self, s: &State, e: &Expr) -> Result<Val, VirErr> {
        match &e.x {
            ExprX::Var(n)=>return s.locals.get(n).cloned().ok_or_else(||unsupported(&e.span,"pure summary variable")),
            ExprX::ReadPlace(p,_)=>return self.formula_place(s,p),
            ExprX::UnaryOpr(UnaryOpr::Field(op),a)=>{
                let value=self.formula_value(s,a)?;
                return self.field(value,op,&e.span);
            }
            ExprX::If(c,a,Some(b))=>{
                let c=self.formula(s,c)?;
                if is_bool(&c,true){return self.formula_value(s,a);}
                if is_bool(&c,false){return self.formula_value(s,b);}
                let a=self.formula_value(s,a)?;let b=self.formula_value(s,b)?;
                return choose(&e.span,c,a,b);
            }
            ExprX::Block(ss,Some(tail)) if ss.is_empty()=>return self.formula_value(s,tail),
            _=>{}
        }
        if let ExprX::Ctor(dt, variant, fs, None) = &e.x {
            if let Some(fields) = records::shape::fields(&e.typ, self.ds) {
                return Ok(Val {
                    typ: e.typ.clone(),
                    node: Node::Fields(
                        fields
                            .into_iter()
                            .map(|(op, _)| {
                                let f = fs
                                    .iter()
                                    .find(|f| f.name == op.field)
                                    .ok_or_else(|| unsupported(&e.span, "owned helper field"))?;
                                Ok((op, self.formula_value(s, &f.a)?))
                            })
                            .collect::<Result<_, VirErr>>()?,
                    ),
                });
            }
            let mut values = vec![];
            for field in fs.iter() {
                values.push(Arc::new(air::ast::BinderX {
                    name: field.name.clone(),
                    a: self.formula_value(s, &field.a)?.scalar(&e.span)?.expr,
                }));
            }
            return Ok(Val::atom(atom(e.new_x(ExprX::Ctor(
                dt.clone(),
                variant.clone(),
                Arc::new(values),
                None,
            )))?));
        }
        Ok(Val::atom(self.formula(s, e)?))
    }
    fn formula_place(&self,s:&State,p:&Place)->Result<Val,VirErr>{
        match &p.x {
            PlaceX::Local(n)=>s.locals.get(n).cloned().ok_or_else(||unsupported(&p.span,"pure summary place")),
            PlaceX::Temporary(e)=>self.formula_value(s,e),
            PlaceX::Field(op,base)=>{let value=self.formula_place(s,base)?;self.field(value,op,&p.span)},
            _=>Err(unsupported(&p.span,"unsupported pure summary place")),
        }
    }
    fn post(&self, v: &Val, out: &Expr) -> Result<Term, VirErr> {
        let span = &out.span;
        match &v.node {
            Node::Atom(t) => records::equal(span, atom(out.clone())?, t.clone()),
            Node::Seq(t) => records::equal(span, self.observe(out)?, t.clone()),
            Node::Iterator(..) => Err(unsupported(span,"inferred iterator outputs require a supplied finite view")),
            Node::Fields(fs) => {
                if records::shape::value_type(&v.typ,&self.f.x.typ_params,self.ds,8) {
                    return records::equal(span,atom(out.clone())?,v.scalar(span)?);
                }
                let mut p = boolean(span, true);
                for (op, v) in fs {
                    p = logical(
                        span,
                        LogicalOp::And,
                        p,
                        self.post(v, &field_expr(out, op, &v.typ))?,
                    )?;
                }
                Ok(p)
            }
            Node::Result(ok, a, b) => self.result_post(ok, a.as_deref(), b.as_deref(), out),
            Node::Variants(branches) => self.variants_post(branches,out),
            Node::Option(some, payload) => {
                let tag = tag_expr(out);
                let mut p = records::equal(span, atom(tag.clone())?, some.clone())?;
                if let Some(v) = payload {
                    let q = self.post(v, &field_expr(out, &option_field(&out.typ), &v.typ))?;
                    p = logical(
                        span,
                        LogicalOp::And,
                        p,
                        logical(span, LogicalOp::Implies, atom(tag)?, q)?,
                    )?;
                }
                Ok(p)
            }
        }
    }
    fn decode(&mut self, s: &mut State, out: &Expr, post: &Expr) -> Result<Val, VirErr> {
        if identity_borrow_return(self.f) {
            let rhs=find_rhs(out,&[post.clone()])?;
            let value=Val::atom(self.formula(s,&rhs)?);
            if skeleton(&self.post(&value,out)?.expr)!=skeleton(post) {
                return Err(unsupported(&post.span,"invalid borrowed identity summary"));
            }
            return Ok(value);
        }
        let layout = self.layout(out.clone(), 8)?;
        let mut clauses = vec![];
        split(post, &mut clauses);
        let v = self.decode_parts(s, out, &layout, &clauses)?;
        // Exact shape, guards and complete output coverage are checked on imports.
        let expected = self.post(&v, out)?;
        if skeleton(&expected.expr) != skeleton(post) {
            return Err(unsupported(&post.span, "invalid owned summary coverage or guard"));
        }
        Ok(v)
    }
    fn decode_parts(
        &mut self,
        s: &mut State,
        out: &Expr,
        layout: &Val,
        clauses: &[Expr],
    ) -> Result<Val, VirErr> {
        let node = match &layout.node {
            Node::Iterator(..) => return Err(unsupported(&out.span,"imported iterator output layout")),
            Node::Atom(_) | Node::Seq(_) => {
                let lhs = if matches!(layout.node, Node::Seq(_)) {
                    self.observe(out)?.expr
                } else {
                    out.clone()
                };
                let rhs = find_rhs(&lhs, clauses)?;
                let t = self.formula(s, &rhs)?;
                if matches!(layout.node, Node::Seq(_)) { Node::Seq(t) } else { Node::Atom(t) }
            }
            Node::Fields(_) if records::shape::value_type(&layout.typ,&self.f.x.typ_params,self.ds,8)
                && clauses.iter().any(|e|matches!(&e.x,ExprX::Binary(BinaryOp::Eq(Mode::Spec),a,_) if skeleton(a)==skeleton(out))) => {
                let rhs=find_rhs(out,clauses)?;
                Node::Atom(self.formula(s,&rhs)?)
            }
            Node::Fields(fs) => Node::Fields(
                fs.iter()
                    .map(|(op, v)| {
                        Ok((
                            op.clone(),
                            self.decode_parts(s, &field_expr(out, op, &v.typ), v, clauses)?,
                        ))
                    })
                    .collect::<Result<_, VirErr>>()?,
            ),
            Node::Result(_, a, b) => {
                self.result_decode(s, out, a.as_deref(), b.as_deref(), clauses)?
            }
            Node::Variants(branches)=>self.variants_decode(s,out,branches,clauses)?,
            Node::Option(_, payload) => {
                let tag = tag_expr(out);
                let rhs = find_rhs(&tag, clauses)?;
                let some = self.formula(s, &rhs)?;
                let guarded = clauses.iter().find_map(|e| {
                    if let ExprX::Logical(LogicalOp::Implies, c, q) = &e.x {
                        (skeleton(c) == skeleton(&tag)).then_some(q.clone())
                    } else {
                        None
                    }
                });
                let p = if let Some(q) = guarded {
                    let mut cs = vec![];
                    split(&q, &mut cs);
                    let v = payload.as_ref().unwrap();
                    Some(Box::new(self.decode_parts(
                        s,
                        &field_expr(out, &option_field(&out.typ), &v.typ),
                        v,
                        &cs,
                    )?))
                } else if is_bool(&some, false) {
                    None
                } else {
                    return Err(unsupported(&out.span, "owned Option missing payload summary"));
                };
                Node::Option(some, p)
            }
        };
        Ok(Val { typ: layout.typ.clone(), node })
    }
}
fn field_expr(e: &Expr, op: &FieldOpr, t: &Typ) -> Expr {
    SpannedTyped::new(&e.span, t, ExprX::UnaryOpr(UnaryOpr::Field(op.clone()), e.clone()))
}
fn option_field(t: &Typ) -> FieldOpr {
    let TypX::Datatype(dt, ..) = &**t else { unreachable!() };
    FieldOpr {
        datatype: dt.clone(),
        variant: Arc::new("Some".into()),
        field: crate::def::positional_field_ident(0),
        get_variant: false,
        check: VariantCheck::None,
    }
}
fn tag_expr(e: &Expr) -> Expr {
    let TypX::Datatype(dt, ..) = &*e.typ else { unreachable!() };
    SpannedTyped::new(
        &e.span,
        &bool_typ(),
        ExprX::UnaryOpr(
            UnaryOpr::IsVariant { datatype: dt.clone(), variant: Arc::new("Some".into()) },
            e.clone(),
        ),
    )
}
// Push tag observations through complete value conditionals. In particular,
// is_Some(if c { Some(x) } else { None }) is just c. Keep this in the
// untrusted candidate builder; original-body precision proofs remain unchanged.
fn variant_test(s: &Span, value: Term, datatype: &Dt, variant: &Ident) -> Result<Term, VirErr> {
    match &value.expr.x {
        ExprX::Ctor(dt, actual, _, None) if dt == datatype => Ok(boolean(s, actual == variant)),
        ExprX::If(c, a, Some(b)) => {
            let c = atom(c.clone())?;
            let a = variant_test(s, atom(a.clone())?, datatype, variant)?;
            let b = variant_test(s, atom(b.clone())?, datatype, variant)?;
            if is_bool(&a, true) && is_bool(&b, false) { return Ok(c); }
            if is_bool(&a, false) && is_bool(&b, true) { return exits::not(s, c); }
            if is_bool(&a, false) && is_bool(&b, false) { return Ok(a); }
            conditional(s, c, a, b)
        }
        _ => term(s, &bool_typ(), ExprX::UnaryOpr(UnaryOpr::IsVariant {
            datatype: datatype.clone(), variant: variant.clone(),
        }, value.expr), value.size + 1),
    }
}
fn element(t: &Typ, all: &HashMap<Fun, Function>, ds: &[Datatype]) -> Result<Typ, VirErr> {
    if let Some(ts) = collections::vec_args(base(t)) {
        return Ok(ts[0].clone());
    }
    sequence_views::find(base(t), all, ds)
        .map(|v| v.element)
        .ok_or_else(|| crate::messages::error_bare("unsupported owned sequence type"))
}
fn clone_identity(t: &Typ) -> bool {
    scalar(t)
        || option(t).is_some_and(clone_identity)
        || matches!(&**t,TypX::Datatype(Dt::Tuple(_),ts,_) if ts.iter().all(clone_identity))
}
fn choose(s: &Span, c: Term, a: Val, b: Val) -> Result<Val, VirErr> {
    if is_bool(&c, true) {
        return Ok(a);
    }
    if is_bool(&c, false) {
        return Ok(b);
    }
    let node = match (a.node, b.node) {
        (Node::Atom(a), Node::Atom(b)) => Node::Atom(conditional(s, c, a, b)?),
        (Node::Seq(a), Node::Seq(b)) => Node::Seq(term(
            s,
            &a.expr.typ,
            ExprX::If(c.expr, a.expr.clone(), Some(b.expr)),
            c.size + a.size + b.size + 1,
        )?),
        (Node::Fields(a), Node::Fields(b)) if a.len() == b.len() => Node::Fields(
            a.into_iter()
                .zip(b)
                .map(|((p, a), (q, b))| {
                    if p.datatype == q.datatype && p.field == q.field {
                        Ok((p, choose(s, c.clone(), a, b)?))
                    } else {
                        Err(unsupported(s, "owned branch fields"))
                    }
                })
                .collect::<Result<_, VirErr>>()?,
        ),
        (Node::Result(at, aa, ab), Node::Result(bt, ba, bb)) => {
            let merge = |a: Option<Box<Val>>, b: Option<Box<Val>>| -> Result<_, VirErr> {
                Ok(match (a, b) {
                    (Some(a), Some(b)) => Some(Box::new(choose(s, c.clone(), *a, *b)?)),
                    (a, None) | (None, a) => a,
                })
            };
            Node::Result(conditional(s, c.clone(), at, bt)?, merge(aa, ba)?, merge(ab, bb)?)
        }
        (Node::Variants(a),Node::Variants(b)) if a.len()==b.len()=>Node::Variants(
            a.into_iter().zip(b).map(|((av,at,af),(bv,bt,bf))| {
                if av!=bv{return Err(unsupported(s,"structured enum branch mismatch"));}
                let fields=match (af,bf) {
                    (Some(a),Some(b)) if a.len()==b.len()=>Some(a.into_iter().zip(b).map(|((op,a),(other,b))| {
                        if !records::shape::same_path(&[op.clone()],&[other]) {return Err(unsupported(s,"structured enum payload mismatch"));}
                        Ok((op,choose(s,c.clone(),a,b)?))
                    }).collect::<Result<Vec<_>,VirErr>>()?),
                    (a,None)|(None,a)=>a,
                    _=>return Err(unsupported(s,"structured enum payload count")),
                };
                Ok((av,conditional(s,c.clone(),at,bt)?,fields))
            }).collect::<Result<Vec<_>,VirErr>>()?),
        (Node::Option(at, ap), Node::Option(bt, bp)) => Node::Option(
            conditional(s, c.clone(), at, bt)?,
            match (ap, bp) {
                (Some(a), Some(b)) => Some(Box::new(choose(s, c, *a, *b)?)),
                (a, None) | (None, a) => a,
            },
        ),
        _ => return Err(unsupported(s, "owned branch type")),
    };
    Ok(Val { typ: a.typ, node })
}
fn split(e: &Expr, out: &mut Vec<Expr>) {
    if let ExprX::Logical(LogicalOp::And, a, b) = &e.x {
        split(a, out);
        split(b, out);
    } else {
        out.push(e.clone());
    }
}
fn find_rhs(lhs: &Expr, clauses: &[Expr]) -> Result<Expr, VirErr> {
    let mut found = clauses.iter().filter_map(|e| {
        if let ExprX::Binary(BinaryOp::Eq(Mode::Spec), a, b) = &e.x {
            (skeleton(a) == skeleton(lhs)).then_some(b.clone())
        } else {
            None
        }
    });
    let rhs =
        found.next().ok_or_else(|| unsupported(&lhs.span, "missing owned output observation"))?;
    if found.next().is_some() {
        return Err(unsupported(&lhs.span, "duplicate owned observation"));
    }
    Ok(rhs)
}
pub(super) fn skeleton(e: &Expr) -> String {
    // Spans are deliberately excluded; names, types and operations remain bound.
    let mut value = serde_json::to_value(e).unwrap();
    fn strip(v: &mut serde_json::Value) {
        match v {
            serde_json::Value::Object(m) => {
                m.remove("span");
                if m.contains_key("preliminary_kind") {
                    m.remove("id");
                }
                for v in m.values_mut() {
                    strip(v)
                }
            }
            serde_json::Value::Array(v) => {
                for v in v {
                    strip(v)
                }
            }
            _ => {}
        }
    }
    strip(&mut value);
    value.to_string()
}
/// Reject user loop proofs before dependency traversal reaches their spec calls.
pub(super) fn validate_source(f: &Function) -> Result<(), VirErr> {
    let native=native_for::sources(f);
    if native.len()>1{return Err(unsupported(&f.span,"one native for-loop per owned constructor"));}

    if let Some(body) = &f.x.body {
        crate::ast_visitor::expr_visitor_check(body, &mut |_, e| {
            if let ExprX::Loop {
                invs,
                decrease,
                assume_termination,
                is_for_loop,
                atomic_call,
                ..
            } = &e.x
            {
                let native_loop=*is_for_loop && native.iter().any(|s|s.loop_expr.span.id==e.span.id);
                if !native_loop && (!invs.is_empty() || !decrease.is_empty() || *is_for_loop)
                    || *assume_termination || *atomic_call
                {
                    return Err(unsupported(
                        &e.span,
                        "owned loops require one empty-vector fill or prefix-copy pattern with no handwritten annotations or termination bypass",
                    ));
                }
            }
            if !native.is_empty() && matches!(&e.x,ExprX::AssertAssume{is_assume:true,..}) {
                return Err(unsupported(&e.span,"native for inference rejects assumptions"));
            }
            Ok(())
        })?;
    }
    Ok(())
}
pub(crate) fn native_scaffold_call(f:&Function,e:&Expr,all:&HashMap<Fun,Function>)->bool{native_for::scaffold_call(f,e,all)}

pub(super) fn infer(
    f: &Function,
    all: &HashMap<Fun, Function>,
    ds: &[Datatype],
) -> Result<(Contract, Expr), VirErr> {
    validate_signature(f, all, ds)?;
    let mut eng = Engine {
        f,
        all,
        ds,
        requirements: vec![],
        exits: vec![],
        plans: vec![], callbacks:vec![],
        remaining: FORMULA_BUDGET,
    };
    let mut s = State { locals: HashMap::new(), objects:HashMap::new(), active: boolean(&f.span, true) };
    for p in f.x.params.iter() {
        s.objects.insert(p.x.name.clone(),atom(SpannedTyped::new(&p.span,&p.x.typ,ExprX::Var(p.x.name.clone())))?);
        s.locals.insert(
            p.x.name.clone(),
            eng.layout(SpannedTyped::new(&p.span, &p.x.typ, ExprX::Var(p.x.name.clone())), 8)?,
        );
    }
    if identity_borrow_return(f) {
        if let Some(value)=eng.borrowed_value(&s,f)? {
            let body=f.x.body.as_ref().ok_or_else(||unsupported(&f.span,"borrowed projection body missing"))?;
            let out=SpannedTyped::new(&f.span,&f.x.inner_ret.x.typ,ExprX::Var(f.x.inner_ret.x.name.clone()));
            return Ok((Contract::new(boolean(&f.span,true).expr,records::equal(&f.span,atom(out)?,value)?.expr)?,body.clone()));
        }
    }
    let result = eng.expr(
        &mut s,
        f.x.body.as_ref().ok_or_else(|| unsupported(&f.span, "owned body missing"))?,
    )?;
    let mut result = if is_bool(&s.active, false) {
        eng.exits
            .last()
            .map(|(_, v)| v.clone())
            .ok_or_else(|| unsupported(&f.span, "owned return missing"))?
    } else {
        result
    };
    for (c, v) in eng.exits.iter().rev() {
        result = choose(&f.span, c.clone(), v.clone(), result)?;
    }
    let mut pre = boolean(&f.span, true);
    for p in &eng.requirements {
        pre = logical(&f.span, LogicalOp::And, pre, p.clone())?;
    }
    if !native_for::sources(f).is_empty(){pre=boolean(&f.span,true);}
    let out =
        SpannedTyped::new(&f.span, &f.x.inner_ret.x.typ, ExprX::Var(f.x.inner_ret.x.name.clone()));
    let post = eng.post(&result, &out)?;
    let body = loops::annotate(f, &eng.plans)?;
    let body = callbacks::annotate(f,&body,&eng.callbacks)?;
    Ok((Contract::new(pre.expr, post.expr)?, body))
}
fn validate_signature(
    f: &Function,
    all: &HashMap<Fun, Function>,
    ds: &[Datatype],
) -> Result<(), VirErr> {
    let x = &f.x;
    if !x.attrs.infer_owned
        || x.attrs.infer_allow_approx
        || x.mode != Mode::Exec
        || !(matches!(x.kind, FunctionKind::Static) || wrapper_traits::kind(f)==Some(wrapper_traits::Kind::From))
        || !matches!(x.item_kind, ItemKind::Function)
        || x.inner_ret.x.mode != Mode::Exec
        || x.attrs.broadcast_forall
        || x.typ_bounds.len() > 24
        || x.proxy.is_some()
        || x.attrs.is_external_body
        || x.attrs.is_unsafe
        || x.attrs.is_async
        || x.attrs.prophecy_dependent
        || x.attrs.exec_assume_termination
        || x.attrs.exec_allows_no_decreases_clause
        || x.attrs.autospec.is_some()
        || x.attrs.bit_vector
        || x.attrs.integer_ring
        || x.params.len() > 8
        || x.typ_params.len() > 4
        || x.params
            .iter()
            .any(|p| p.x.mode != Mode::Exec || p.x.user_mut || matches!(&*p.x.typ, TypX::MutRef(_)))
        || !x.require.is_empty()
        || !x.ensure.0.is_empty()
        || !x.ensure.1.is_empty()
        || x.returns.is_some()
        || !x.decrease.is_empty()
        || x.decrease_by.is_some()
        || x.decrease_when.is_some()
        || !(heap_known(&x.inner_ret.x.typ, all, ds, 8) || result_query(f)
            || structured_return(f,ds)
            || sums::has_value_enum(&x.inner_ret.x.typ,ds,5)
            || x.params.iter().any(|p|sums::has_value_enum(&p.x.typ,ds,5)) || iterators::uses(f,all) || equality::uses(f,all,ds)
            || result_map::uses(f,all) || f.x.body.is_none() && eligibility::scalar(&x.inner_ret.x.typ))
        || !same(&x.inner_ret.x.typ, &x.outer_ret.x.typ)
    {
        return Err(unsupported(
            &f.span,
            "owned inference requires a bounded pure constructor with no handwritten contracts or proof bypass",
        ));
    }
    // Bound type expansion before building any observation expressions.
    fn consume(
        t: &Typ,
        all: &HashMap<Fun, Function>,
        ds: &[Datatype],
        depth: usize,
        left: &mut usize,
    ) -> bool {
        if depth == 0 {
            return false;
        }
        let t = base(t);
        let sequence =
            collections::vec_args(t).is_some() || sequence_views::find(t, all, ds).is_some();
        if !sequence && !opaque_identity(t,ds) {
            if let Some(payload) = option(t).filter(|p| heap_known(p, all, ds, 5)) {
                if *left == 0 {
                    return false;
                }
                *left -= 1;
                return consume(payload, all, ds, depth - 1, left);
            }
            if let Some((a, b)) = sums::result(t) {
                if *left == 0 {
                    return false;
                }
                *left -= 1;
                return consume(a, all, ds, depth - 1, left)
                    && consume(b, all, ds, depth - 1, left);
            }
            if let Some(vs)=records::shape::variants(t,ds) {
                for (_,fs) in vs {
                    if *left==0{return false;} *left-=1;
                    if !fs.iter().all(|(_,t)|consume(t,all,ds,depth-1,left)){return false;}
                }
                return true;
            }
            if let Some(fs) = records::shape::fields(t, ds) {
                return fs.iter().all(|(_, t)| consume(t, all, ds, depth - 1, left));
            }
        }
        if *left == 0 {
            return false;
        }
        *left -= 1;
        true
    }
    let extended=variants::has_heap_enum(&x.inner_ret.x.typ,all,ds,8);
    let (depth,budget)=if extended {(8,64)}else{(5,16)};
    if !consume(&x.inner_ret.x.typ, all, ds, depth, &mut budget.clone()) {
        return Err(unsupported(&f.span, &format!("owned output exceeds {budget} observations or type depth")));
    }
    for p in x.params.iter() {
        if !consume(&p.x.typ, all, ds, depth, &mut budget.clone()) {
            return Err(unsupported(
                &p.span,
                "owned input exceeds observation budget or type depth",
            ));
        }
    }
    let engine =
        Engine { f, all, ds, requirements: vec![], exits: vec![], plans: vec![], callbacks:vec![], remaining: 0 };
    engine.layout(
        SpannedTyped::new(&f.span, &x.inner_ret.x.typ, ExprX::Var(x.inner_ret.x.name.clone())),
        8,
    )?;

    Ok(())
}
pub(super) fn validate_summary(
    f: &Function,
    all: &HashMap<Fun, Function>,
    ds: &[Datatype],
) -> Result<(), VirErr> {
    crate::infer_proof::CandidateId::from_function(f)?;
    if f.x.body.is_some()
        || f.x.require.len() != 1
        || f.x.ensure.0.len() != 1
        || !f.x.ensure.1.is_empty()
    {
        return Err(unsupported(&f.span, "invalid owned summary"));
    }
    let mut x = f.x.clone();
    x.require = Arc::new(vec![]);
    x.ensure = (Arc::new(vec![]), Arc::new(vec![]));
    validate_signature(&f.new_x(x), all, ds)?;
    let mut e = Engine {
        f,
        all,
        ds,
        requirements: vec![],
        exits: vec![],
        plans: vec![], callbacks:vec![],
        remaining: FORMULA_BUDGET,
    };
    let mut s = State { locals: HashMap::new(), objects:HashMap::new(), active: boolean(&f.span, true) };
    for p in f.x.params.iter() {
        s.objects.insert(p.x.name.clone(),atom(SpannedTyped::new(&p.span,&p.x.typ,ExprX::Var(p.x.name.clone())))?);
        s.locals.insert(
            p.x.name.clone(),
            e.layout(SpannedTyped::new(&p.span, &p.x.typ, ExprX::Var(p.x.name.clone())), 8)?,
        );
    }
    validate_formula(&f.x.require[0], all)?;
    validate_formula(&f.x.ensure.0[0], all)?;
    e.formula(&mut s, &f.x.require[0])?;
    e.decode(
        &mut s,
        &SpannedTyped::new(&f.span, &f.x.inner_ret.x.typ, ExprX::Var(f.x.inner_ret.x.name.clone())),
        &f.x.ensure.0[0],
    )?;
    Ok(())
}
pub(crate) fn validate_decoration(f:&Function,body:&Expr)->Result<(),VirErr>{callbacks::validate(f,body)}
pub(super) fn restore_body(f: &Function, decorated: &Expr) -> Result<Expr, VirErr> {
    let mut plans = vec![];
    crate::ast_visitor::expr_visitor_check(decorated, &mut |_, e| {
        if let ExprX::Loop { invs, decrease, .. } = &e.x {
            plans.push(loops::Plan {
                span: e.span.id,
                invs: invs.clone(),
                decrease: decrease.clone(),
            });
        }
        Ok::<_, VirErr>(())
    })?;
    let body=loops::annotate(f,&plans)?;
    callbacks::annotate(f,&body,&callbacks::plans(decorated))
}

fn validate_formula(e: &Expr, all: &HashMap<Fun, Function>) -> Result<(), VirErr> {
    let mut budget = FORMULA_BUDGET;
    crate::ast_visitor::expr_visitor_check(e, &mut |_, e| {
        budget = budget
            .checked_sub(1)
            .ok_or_else(|| unsupported(&e.span, "owned imported formula budget"))?;
        let allowed = match &e.x {
            ExprX::Const(_)
            | ExprX::Var(_)
            | ExprX::Unary(_, _)
            | ExprX::UnaryOpr(_, _)
            | ExprX::Binary(_, _, _)
            | ExprX::Logical(_, _, _)
            | ExprX::Ctor(_, _, _, None)
            | ExprX::Closure(_, _)
            | ExprX::Quant(_, _, _)
            | ExprX::If(_, _, Some(_)) => true,
            ExprX::ReadPlace(p, _) => {
                fn place(p: &Place) -> bool {
                    match &p.x {
                        PlaceX::Local(_) => true,
                        PlaceX::Field(_, p) => place(p),
                        PlaceX::Temporary(_) => true,
                        _ => false,
                    }
                }
                place(p)
            }
            ExprX::Call {
                target: CallTarget::Fun(CallTargetKind::Dynamic,n,types,impls,_),
                args,post_args:None,body:None,
            } if types.len()==1 && impls.is_empty() => {
                all.get(n).is_some_and(|f| args.len()==f.x.params.len()
                    && (is_pure_observation(f) || f.x.attrs.infer_trait_observation && trait_getters::observation(f)))
            }
            ExprX::Call {
                target: CallTarget::Fun(kind, n, ..),
                post_args: None,
                body: None,
                ..
            } => {
                let n = if let CallTargetKind::DynamicResolved { resolved, .. } = kind {
                    resolved
                } else {
                    n
                };
                all.get(n).is_some_and(|f| {
                    f.x.mode == Mode::Spec
                        && (is_pure_observation(f) || collections::model(f).is_some()
                            || model(f).is_some()
                            || f.x.attrs.infer_sequence_view
                            || f.x.attrs.infer_trait_observation && trait_getters::observation(f)
                            || f.x.attrs.infer_iterator_observation && iterators::observation(f))
                })
            }
            _ => false,
        };
        if allowed {
            Ok(())
        } else {
            Err(unsupported(&e.span, "non-model expression in owned imported formula"))
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn owned_loop_gate_rejects_executable_changes_and_assumptions() {
        let f = crate::infer::loops::tests::counted_function();
        let span = f.span.clone();
        let inv = LoopInvariant {
            kind: LoopInvariantKind::InvariantAndEnsures,
            inv: boolean(&span, true).expr,
        };
        let measure = SpannedTyped::new(&span, &int_typ(), ExprX::Const(Constant::Int(0.into())));
        let plan = loops::Plan {
            span: span.id,
            invs: Arc::new(vec![inv.clone(), inv]),
            decrease: Arc::new(vec![measure]),
        };
        let decorated = loops::annotate(&f, &[plan]).unwrap();
        validate_decoration(&f, &decorated).unwrap();
        let changed = crate::ast_visitor::map_expr_visitor(&decorated, &|e| {
            if let ExprX::Loop {
                loop_isolation,
                allow_complex_invariants,
                is_for_loop,
                assume_termination,
                label,
                body,
                atomic_call,
                invs,
                decrease,
                ..
            } = &e.x
            {
                Ok(e.new_x(ExprX::Loop {
                    loop_isolation: *loop_isolation,
                    allow_complex_invariants: *allow_complex_invariants,
                    is_for_loop: *is_for_loop,
                    assume_termination: *assume_termination,
                    label: label.clone(),
                    cond: Some(boolean(&e.span, false).expr),
                    body: body.clone(),
                    atomic_call: *atomic_call,
                    invs: invs.clone(),
                    decrease: decrease.clone(),
                }))
            } else {
                Ok(e.clone())
            }
        })
        .unwrap();
        assert!(validate_decoration(&f, &changed).is_err());
        let assumption = SpannedTyped::new(
            &span,
            &crate::ast_util::unit_typ(),
            ExprX::AssertAssume { is_assume: true, expr: boolean(&span, false).expr, msg: None },
        );
        assert!(validate_formula(&assumption, &HashMap::new()).is_err());
        assert!(loops::pure(&assumption).is_err());
        assert!(validate_decoration(&f, &assumption).is_err());
    }
}
