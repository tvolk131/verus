"""Local acceptance and publication regression trials; all gates survive -O."""
from pathlib import Path
import hashlib
import json
import sys
import trials as t

R=Path(sys.argv[1]); B=R/'bundle'

def raw(label,text,options=(),goals=True,export=None,cargo=False):
    d=R/label;d.mkdir(parents=True,exist_ok=True)
    src=d/'component.rs';src.write_text(text)
    args=[B/'verus',src,'--crate-name=component','--crate-type=lib','--no-cheating','--output-json','--compile','-o',d/'libcomponent.rlib']
    if goals: args+=['--infer-goals']
    args+=list(options)
    if export:args+=['--export',export]
    env={}
    if cargo:
        project=R/'cargo-project';project.mkdir(exist_ok=True)
        (project/'Cargo.toml').write_text('[package]\nname="local_bool"\nversion="0.0.0"\nedition="2021"\n')
        package='local_bool-0.0.0-'+hashlib.sha256(str(project/'Cargo.toml').encode()).hexdigest()[:12]
        env={'__VERUS_DRIVER_VIA_CARGO__':'1','__VERUS_DRIVER_VERIFY_'+package:'1','CARGO_PKG_NAME':'local_bool','CARGO_PKG_VERSION':'0.0.0','CARGO_MANIFEST_DIR':str(project),'CARGO_PRIMARY_PACKAGE':'1'}
        args=[B/'rust_verify',t.TC/'bin/rustc',src,'--crate-name','component','--crate-type=lib','--no-cheating','--output-json','--emit=metadata','--out-dir',d,'--VIA-CARGO','compile-when-primary-package','--extern','verus_builtin='+str(B/'libverus_builtin.rlib'),'--extern','verus_builtin_macros='+str(B/'libverus_builtin_macros.so'),'--extern','vstd='+str(B/'libvstd.rlib'),'--import','vstd='+str(B/'vstd.vir'),'-L','dependency='+str(B)]
        if goals:args+=['--infer-goals']
        args+=list(options)
    return t.run(R,label,args,B,env)

def counts(record,verified,errors):
    v=record['verification'];t.need(v.get('verified')==verified and v.get('errors')==errors,'proof counts')
    t.need(v.get('is-verifying-entire-crate') is True,'complete scope')
    t.need(v.get('success')==(errors==0),'success field')

if __name__=='__main__':
    t.need(sys.flags.dont_write_bytecode,'use -B')
    mode=sys.argv[2]
    if mode=='proofs':
        for label,kind,v,e in [('final-cold','actual',2,0),('final-warm','actual',2,0),('final-false','false',1,1),('final-off','off',0,1),('final-full','full-help',52,0),('final-support','full-helpers',36,0)]:
            d,r=t.trial(R,B,label,kind);counts(r,v,e)
            if kind=='false': t.need(r['failed_functions']==['component::all_bool'],'false caller only')
    elif mode=='exports':
        seed='use vstd::prelude::*; verus! {pub fn seed()->u64 {0}}\n'
        legacy=R/'legacy-ordinary.vir'
        d,r=raw('ordinary-seed',seed,goals=False,export=legacy)
        t.need(r['exit']==0 and legacy.exists(),'healthy ordinary legacy export')
        old=legacy.read_bytes()
        cases=[('complete',(),t.source('actual')),('false',(),t.source('false')),
               ('no-verify',('--no-verify',),t.source('actual')),('no-lifetime',('--no-lifetime',),t.source('actual')),
               ('no-erasure',('--no-erasure-check',),t.source('actual')),('no-traits',('--no-trait-conflicts',),t.source('actual')),
               ('filter-function',('--verify-function','all_bool'),t.source('actual')),
               ('filter-module',('--verify-module','root'),t.source('actual')),
               ('filter-only-module',('--verify-only-module','root'),t.source('actual')),
               ('filter-root',('--verify-root',),t.source('actual')),
               ('type-error',(),t.source('actual')+'fn bad()->u64{()}'),
               ('parse-error',(),t.source('actual')+'fn broken( {')]
        records=[]
        for cargo in [False,True]:
            for name,options,text in cases:
                label=('cargo-' if cargo else 'explicit-')+name
                d=R/label;d.mkdir(exist_ok=True)
                selected=d/'libcomponent.vir' if cargo else d/'selected.vir'
                selected.write_bytes(old)
                out,r=raw(label,text,options,export=None if cargo else selected,cargo=cargo)
                t.need(r['exit']!=0,'export accepted '+label)
                t.need(not selected.exists(),'stale selected export survived '+label)
                stderr=(out/'stderr').read_text()
                if name not in ['type-error','parse-error']:
                    t.need('local-only' in stderr,'wrong export rejection '+label)
                records.append(dict(label=label,selected=str(selected),removed=True,receipt_sha256=t.sha(out/'receipt.json')))
        # Incomplete local runs cannot use the helper even without export.
        for name,options,text in cases[2:10]:
            out,r=raw('local-'+name,text,options)
            t.need(r['exit']!=0,'incomplete local run accepted '+name)
            t.need('complete source/body' in (out/'stderr').read_text(),'wrong incomplete rejection '+name)
        t.write(R/'export-controls.json',dict(legacy_sha256=hashlib.sha256(old).hexdigest(),records=records))
    elif mode=='shapes':
        cases={
            'other-output':'pub fn call(b:bool)->u32 { u32::from(b) }',
            'other-input':'pub fn call(b:u32)->(r:u64) ensures r==b as u64, {u64::from(b)}',
            'unresolved':'pub fn call<T:From<bool>>(b:bool)->T { T::from(b) }',
            'fndef-value':'pub fn call(b:bool)->(r:u64) ensures r==if b {1u64}else{0u64}, {let f=u64::from;f(b)}',
            'coercion':'pub fn call(b:bool)->u64 {let f:fn(bool)->u64=u64::from;f(b)}',
            'user-trait':'pub trait From<T>:Sized {fn from(b:T)->Self;} impl From<bool> for u64 {fn from(b:bool)->Self {2}} pub fn call(b:bool)->u64 {<u64 as From<bool>>::from(b)}',
        }
        records=[]
        for name,body in cases.items():
            text='use vstd::prelude::*;\nverus! {\n'+body+'\n}\n';pair=[]
            for goals in [False,True]:
                label='shape-'+name+('-on' if goals else '-off')
                out,r=raw(label,text,goals=goals)
                output=json.loads((out/'stdout').read_text())
                t.need(not any('checked_foreign_bool_to_u64%' in x for x in output['func-details']),'new law for '+name)
                pair.append({k:r.get('verification',{}).get(k) for k in ['success','verified','errors','encountered-vir-error']})
            t.need(pair[0]==pair[1],'nearby behavior changed '+name)
            records.append(dict(shape=name,off=pair[0],on=pair[1]))
        t.write(R/'call-shape-controls.json',records)
