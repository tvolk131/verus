"""Ordinary development mutation controls, never a shipping compiler mode.
Run only with Python -B. Restores the exact implementation source in finally.
No races, adversarial security experiments, or prior writing drivers.
"""
from pathlib import Path
import importlib.util
import shutil
import subprocess
import sys
import trials as t

def replace(text,old,new):
    t.need(text.count(old)==1,'unique control mutation: '+old)
    return text.replace(old,new)

if __name__=='__main__':
    t.need(sys.flags.dont_write_bytecode,'use -B')
    root=Path(sys.argv[1]); b=root/'bundle'
    p=t.ROOT/'source/rust_verify/src/checked_foreign.rs'
    packing=t.ROOT/'source/rust_verify/src/checked_foreign_packing.rs'
    original=p.read_text(); oldpacking=packing.read_text()
    controls={
        'generated-body': replace(original,'    // Independent ordinary contract proposal, NOT inference-summary synthesis.',
            '''    let body = if let ExprX::If(c,a,Some(b)) = &body.x {
        body.new_x(ExprX::If(c.clone(),b.clone(),Some(a.clone())))
    } else { unreachable!() };
    // Independent ordinary contract proposal, NOT inference-summary synthesis.'''),
        'generated-type':replace(original,'let u64t = Arc::new(TypX::Int(IntRange::U(64)));','let u64t = Arc::new(TypX::Int(IntRange::U(32)));'),
        'false-proposal':replace(original,'    let root = crate::rust_to_vir::get_root_module_path(ctxt);',
            '    let ensure = e(&bool_typ(), ExprX::Const(Constant::Bool(false)));\n    let root = crate::rust_to_vir::get_root_module_path(ctxt);'),
        'mir-local-type':replace(original,'        let primitive = inspect(ctxt.tcx, env, &body).map_err(|e| {',
            '        let mut body = body; body.local_decls[mir::RETURN_PLACE].ty=ctxt.tcx.types.u32;\n        let primitive = inspect(ctxt.tcx, env, &body).map_err(|e| {'),
        'mir-extra-block':replace(original,'        let primitive = inspect(ctxt.tcx, env, &body).map_err(|e| {',
            '        let mut body = body; let extra=body.basic_blocks[mir::START_BLOCK].clone(); body.basic_blocks.as_mut().push(extra);\n        let primitive = inspect(ctxt.tcx, env, &body).map_err(|e| {'),
    }
    if sys.argv[2:]==["packing-only"]: controls={}
    try:
        for name,code in controls.items():
            p.write_text(code)
            d=root/('fault-'+name);d.mkdir(exist_ok=True)
            (d/'mutated.rs').write_text(code)
            t.write(d/'mutation.json',dict(mutated_sha256=t.sha(p),kind=name))
            build=t.run(root,'build-fault-'+name,['bash','/tmp/local-bool-build.sh'],b)
            t.need(build[1]['exit']==0,'control compilation '+name)
            cb=d/'bundle';cb.mkdir(exist_ok=True)
            for n in t.TOOLS:
                src=root/'target/debug/rust_verify' if n=='rust_verify' else b/n
                shutil.copyfile(src,cb/n);(cb/n).chmod(0o700)
            case,record=t.trial(d,cb,'proof','actual')
            t.need(record['exit']!=0,'fault accepted '+name)
            stderr=(case/'stderr').read_text()
            expected=('postcondition not satisfied' if name=='false-proposal' else
                      'checked foreign MIR unsupported' if name.startswith('mir-') else
                      'checked foreign MIR/VIR correspondence failed')
            t.need(expected in stderr,'wrong control reason '+name)
            p.write_text(original)
        name='packing-proposal'
        d=root/('fault-'+name);d.mkdir(exist_ok=True)
        code=replace(oldpacking,'ExpX::Const(Constant::Int(value))','ExpX::Const(Constant::Int(value + 1))')
        packing.write_text(code);(d/'mutated.rs').write_text(code)
        build=t.run(root,'build-fault-'+name,['bash','/tmp/local-bool-build.sh'],b)
        t.need(build[1]['exit']==0,'packing control compilation')
        cb=d/'bundle';cb.mkdir(exist_ok=True)
        for n in t.TOOLS:
            src=root/'target/debug/rust_verify' if n=='rust_verify' else b/n
            shutil.copyfile(src,cb/n);(cb/n).chmod(0o700)
        case,record=t.trial(d,cb,'proof','full-help')
        t.need(record['exit']!=0 and 'assertion failed' in (case/'stderr').read_text(),'false BV proposal accepted')
    finally:
        p.write_text(original);packing.write_text(oldpacking)
