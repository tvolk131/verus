#![feature(rustc_private)]
extern crate rustc_driver;
extern crate rustc_hir;
extern crate rustc_interface;
extern crate rustc_middle;

use rustc_middle::ty::{self, TyCtxt};
use std::path::PathBuf;

fn quote(s: &str) -> String {
    let mut out = String::from("\"");
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\u{0}'..='\u{1f}' => out.push_str(&format!("\\u{:04x}", c as u32)),
            _ => out.push(c),
        }
    }
    out.push('"');
    out
}

struct Query {
    output: PathBuf,
}

impl rustc_driver::Callbacks for Query {
    fn after_analysis<'tcx>(
        &mut self,
        _: &rustc_interface::interface::Compiler,
        tcx: TyCtxt<'tcx>,
    ) -> rustc_driver::Compilation {
        let env = ty::TypingEnv::fully_monomorphized();
        let mut records = Vec::new();
        for id in tcx.hir_body_owners() {
            if tcx.def_kind(id.to_def_id()) != rustc_hir::def::DefKind::Fn {
                continue;
            }
            let name = tcx.item_name(id.to_def_id()).to_string();
            if name != "xor_key" && name != "u64" && !name.starts_with("probe_") {
                continue;
            }
            let sig = tcx.fn_sig(id).instantiate_identity().skip_binder();
            for (index, parameter) in sig.inputs().iter().enumerate() {
                let (subject, reference_kind) = match parameter.kind() {
                    ty::Ref(_, subject, mutability) => (*subject, format!("{mutability:?}")),
                    _ => (*parameter, "value".to_owned()),
                };
                let sized = subject.is_sized(tcx, env);
                let byte_array = matches!(subject.kind(), ty::Array(element, _)
                    if matches!(element.kind(), ty::Uint(ty::UintTy::U8)));
                let byte_slice = matches!(subject.kind(), ty::Slice(element)
                    if matches!(element.kind(), ty::Uint(ty::UintTy::U8)));
                let layout = tcx.layout_of(env.as_query_input(subject)).expect("subject layout");
                let parameter_layout =
                    tcx.layout_of(env.as_query_input(*parameter)).expect("parameter layout");
                records.push(format!(
                    "{{\"function\":{},\"parameter\":{},\"parameter_type\":{},\"reference_kind\":{},\"subject_type\":{},\"byte_array\":{},\"byte_slice\":{},\"unpin\":{},\"unsafe_unpin\":{},\"freeze\":{},\"sized\":{},\"static_size\":{},\"minimum_size\":{},\"align\":{},\"backend\":{},\"parameter_size\":{}}}",
                    quote(&name), index, quote(&parameter.to_string()), quote(&reference_kind),
                    quote(&subject.to_string()), byte_array, byte_slice, subject.is_unpin(tcx, env),
                    subject.is_unsafe_unpin(tcx, env), subject.is_freeze(tcx, env), sized,
                    if sized { layout.size.bytes().to_string() } else { "null".to_owned() },
                    layout.size.bytes(), layout.align.abi.bytes(),
                    quote(&format!("{:?}", layout.backend_repr)), parameter_layout.size.bytes(),
                ));
            }
        }
        records.sort();
        let output = format!(
            "{{\"classification\":\"compiler_type_queries_only\",\"native_correspondences\":0,\"publication\":false,\"pointer_bits\":{},\"records\":[{}]}}\n",
            tcx.data_layout.pointer_size().bits(), records.join(",")
        );
        std::fs::write(&self.output, output).expect("write query output");
        rustc_driver::Compilation::Stop
    }
}

fn main() {
    let mut args = std::env::args().skip(1);
    let output = PathBuf::from(args.next().expect("output.json"));
    let mut compiler_args = vec!["entry-type-query".to_owned()];
    compiler_args.extend(args);
    rustc_driver::run_compiler(&compiler_args, &mut Query { output });
}
