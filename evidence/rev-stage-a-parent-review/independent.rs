// Appended only to a private diagnostic copy of the existing StageA test module.
mod independent_review {
    use super::*;
    use serde::{Serialize, Deserialize};
    #[derive(Debug, Serialize, Deserialize)]
    enum OldTransparency { Never, WhenVisible(Visibility) }
    fn enc<T: Serialize>(v:&T)->Result<Vec<u8>, bincode_next::error::EncodeError> {
        bincode_next::serde::encode_to_vec(v,bincode_next::config::legacy())
    }
    fn dec(bytes:&[u8])->Result<DatatypeTransparency,bincode_next::error::DecodeError> {
        bincode_next::serde::decode_from_slice(bytes,bincode_next::config::legacy()).map(|(v,n)| {assert_eq!(n,bytes.len());v})
    }
    #[test]
    fn review_wire_actual_serde_bincode_legacy_old_values_and_internal_refusal() {
        let d=definition();
        assert!(enc(&d.x.transparency).is_err());
        assert!(enc(&d.x).is_err());
        assert!(serde_json::to_vec(&d.x.transparency).is_err());
        for vis in [Visibility{restricted_to:None},Visibility{restricted_to:Some(owner())}] {
            let old=OldTransparency::WhenVisible(vis.clone());
            let current=DatatypeTransparency::WhenVisible(vis);
            let bytes=enc(&old).unwrap();
            assert_eq!(bytes,enc(&current).unwrap());
            assert!(matches!(dec(&bytes).unwrap(),DatatypeTransparency::WhenVisible(_)));
            let (back,n): (OldTransparency,usize)=bincode_next::serde::decode_from_slice(&enc(&current).unwrap(),bincode_next::config::legacy()).unwrap();
            assert_eq!(n,bytes.len()); assert!(matches!(back,OldTransparency::WhenVisible(_)));
            assert_eq!(serde_json::to_vec(&old).unwrap(),serde_json::to_vec(&current).unwrap());
        }
        assert_eq!(enc(&OldTransparency::Never).unwrap(),vec![0,0,0,0]);
        assert_eq!(enc(&DatatypeTransparency::Never).unwrap(),enc(&OldTransparency::Never).unwrap());
        assert!(matches!(dec(&[0,0,0,0]).unwrap(),DatatypeTransparency::Never));
        for tag in [2u32,3,255,65535,u32::MAX] {
            let mut bytes=tag.to_le_bytes().to_vec(); bytes.extend_from_slice(&[0u8;64]);
            assert!(dec(&bytes).is_err(),"invalid tag {tag}");
        }
        // Exhaust all one-byte discriminants in the actual legacy representation.
        for tag in 0u32..256 {
            let mut bytes=tag.to_le_bytes().to_vec();bytes.extend_from_slice(&[0u8;32]);
            if let Ok((value,_))=bincode_next::serde::decode_from_slice::<DatatypeTransparency,_>(&bytes,bincode_next::config::legacy()) {
                assert!(!matches!(value,DatatypeTransparency::Internal(_)));
            }
        }
        for bytes in [&[][..],&[1][..],&[1,0,0,0][..],&[1,0,0,0,1][..],&[2,0,0,0][..]] { assert!(dec(bytes).is_err()); }
        for json in [r#""Internal""#,r#"{"Internal":null}"#,r#"{"Internal":{}}"#,r#"2"#] {assert!(serde_json::from_str::<DatatypeTransparency>(json).is_err());}
        println!("WIRE legacy old0/old1 compatible; Internal encode refused; invalid/truncated tags refused");
    }
    #[test]
    fn review_descriptor_exact_parameter_rejects_decorations_and_wrong_shapes() {
        let d=definition(); let i=Arc::new(TypX::TypParam(d.x.typ_params[0].0.clone()));
        for t in [Arc::new(TypX::Decorate(TypDecoration::Ref,None,i.clone())),Arc::new(TypX::Boxed(i.clone())),Arc::new(TypX::MutRef(i)),bool_typ()] {
            let mut x=d.x.clone(); Arc::make_mut(&mut Arc::make_mut(&mut x.variants)[0].fields)[0]=crate::ast_util::ident_binder(&crate::def::field_ident_from_rust("iter"),&(t,Mode::Exec,Visibility{restricted_to:Some(owner())}));
            assert!(crate::internal_repr::validate_datatype(&Spanned::new(span(),x)).is_err());
        }
        let mut x=d.x.clone();Arc::make_mut(&mut x.variants)[0].name=Arc::new("Fake".into());
        assert!(crate::internal_repr::validate_datatype(&Spanned::new(span(),x)).is_err());
        let mut x=d.x.clone();let v=x.variants[0].clone();Arc::make_mut(&mut x.variants).push(v);
        assert!(crate::internal_repr::validate_datatype(&Spanned::new(span(),x)).is_err());
    }
    #[test]
    fn review_poly_preserves_full_argument_objects_including_implpaths_and_decorations() {
        let d=definition();let (ctx,_,_)=fixture_with_datatypes(vec![d.clone()]);
        for arg in [Arc::new(TypX::TypParam(Arc::new("T".into()))),Arc::new(TypX::Decorate(TypDecoration::Ref,None,bool_typ())),Arc::new(TypX::Datatype(d.x.name.clone(),Arc::new(vec![bool_typ()]),Arc::new(vec![ImplPath::TraitImplPath(path(&["dictionary"]))]))),Arc::new(TypX::MutRef(bool_typ()))] {
            let t=typ(&d,arg);let native=crate::poly::coerce_typ_to_native(&ctx,&t);
            assert!(Arc::ptr_eq(&native,&t)); // No weakened types_equal comparison.
            let boxed=crate::poly::coerce_typ_to_poly(&ctx,&t);
            assert!(matches!(&*boxed,TypX::Boxed(inner) if Arc::ptr_eq(inner,&t)));
        }
    }
    #[test]
    fn review_nested_generic_nominal_selector_and_invariant_controls() {
        let d=definition();let u8t=Arc::new(TypX::Int(IntRange::U(8)));let inner=typ(&d,u8t.clone());
        let a=SpannedTyped::new(&span(),&inner,ExpX::Var(var("a")));
        let c=construct(&d,inner.clone(),a.clone());
        assert!(check_expression(&d,vec![(var("a"),inner.clone())],binary(crate::sst::BinaryOp::Eq,select(&d,inner,c),a),false));
        let rtyp=typ(&d,u8t.clone());let r=SpannedTyped::new(&span(),&rtyp,ExpX::Var(var("r")));
        for (bound,valid) in [(255u32,true),(254,false)] {
            let pred=binary(crate::sst::BinaryOp::Inequality(InequalityOp::Le),select(&d,u8t.clone(),r.clone()),integer(bound));
            assert_eq!(check_expression(&d,vec![(var("r"),rtyp.clone())],pred,false),valid);
        }
    }
    #[test]
    fn review_source_visibility_cross_crate_and_module_encoding_identity() {
        let d=definition();let foreign=Arc::new(PathX{krate:CrateId::Id(Arc::new("other".into()),99),segments:owner().segments.clone()});
        for m in [owner(),path(&["iter","adapters","rev","child"]),path(&["iter","adapters"]),foreign.clone()] {
            let expected=m.krate==CrateId::Core && m.segments.len()>=3;
            assert_eq!(crate::ast_util::is_transparent_to(&d.x.transparency,&m),expected);
            assert!(crate::ast_util::is_encoded_to(&d.x.transparency,&m));
            let (mut ctx,_,_)=fixture_with_datatypes(vec![d.clone()]);ctx.module=Spanned::new(span(),ModuleX{path:m,reveals:None});
            let t=typ(&d,bool_typ());
            assert_eq!(format!("{:?}",crate::sst_to_air::typ_to_air(&ctx,&t)),format!("{:?}",crate::sst_to_air::typ_to_air(&fixture_with_datatypes(vec![d.clone()]).0,&t)));
            assert!(ctx.datatypes_with_invariant.contains(&d.x.name));
        }
        let k=bare_krate(d.clone(),foreign.clone());
        assert!(!crate::well_formed::internal_repr_test_source_access(&k,&d.x.name.expect_path(),Some(foreign),false));
    }
    #[test]
    fn review_corruption_rejected_at_real_wf_and_context_entry() {
        let d=definition();let foreign=path(&["consumer"]);
        for corrupt in [false,true] {
            let mut x=d.x.clone();if corrupt {x.destructor=true;}
            let k=bare_krate(Spanned::new(span(),x),foreign.clone());
            let merged=Arc::new(crate::traits::MergedTraitsX{traits:vec![],trait_impls:vec![],trait_map:HashMap::new()});
            let (wf,_,_)=crate::well_formed::check_crate_initial(&k,&[],Arc::new(HashMap::new()),merged,true);
            assert_eq!(wf.is_ok(),!corrupt);
            let (base,_,_)=fixture_with_datatypes(vec![d.clone()]);
            let result=Ctx::new(&k,base.global,k.modules[0].clone(),vec![],vec![],HashSet::new(),crate::prune::UsedBuiltins{uses_array:false,uses_bytestr:false,uses_pointee_metadata:false,uses_ieee_float:false},vec![],vec![]);
            assert_eq!(result.is_ok(),!corrupt);
        }
    }

}
