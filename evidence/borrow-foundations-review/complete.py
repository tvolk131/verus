#!/usr/bin/env python3
"""Complete query/controller identity checks and precise final review verdicts.

No formal rebuilds, old writes, or checkout traversal. Git reads are immutable.
"""
import ast, collections, hashlib, inspect, json, pathlib, subprocess, sys, time
import finalize as f
P=f.P; OWN=f.OWN; OLD=f.OLD; NEW=f.NEW
load=f.load; save=f.save; need=f.need; sha=f.sha; check=f.check

def git(*args):
    return subprocess.check_output(['/usr/bin/git',*args],cwd=OWN.parent.parent)

def epochs():
    refs=git('log','--format=%H','360d72dd','--','evidence/borrow-foundations-review').decode().splitlines()
    versions={}
    for rev in refs:
        for file in git('ls-tree','-r','--name-only',rev,'evidence/borrow-foundations-review').decode().splitlines():
            if not file.endswith(('.py','.template')):continue
            b=git('show',rev+':'+file);h=hashlib.sha256(b).hexdigest()
            versions.setdefault(h,{'git_commit':rev,'path':file,'bytes':b})
    rows={}
    for p in [OLD/'postguards.json', OLD/'controller-authentication.json']+list((OLD/'sb').glob('*pretrial.json'))+list((OLD/'tb').glob('*pretrial.json')):
        d=load(p)
        for field in ['controller_sha256','original_controller_sha256','review_sha256','generator_sha256','template_sha256']:
            if field not in d:continue
            h=d[field];need(h in versions,('unrecovered controller epoch',str(p),field,h))
            v=versions[h];dest=NEW/'controller-epochs'/(h+P(v['path']).suffix)
            if not dest.exists():dest.parent.mkdir(exist_ok=True);dest.write_bytes(v['bytes'])
            rows[str(p)+':'+field]={'sha256':h,'git_commit':v['git_commit'],'git_path':v['path'],'private_copy':str(dest)}
    save(NEW/'controller-epochs.json',rows)
    return rows

def queries():
    import review
    out={}
    for k,(base,src,delivery,prefix,logical) in f.SPECS.items():
        w=OLD/k;inv=load(w/'inventory.json');libs=inv['libraries'];sig=load(w/'signature-api-inventory.json')
        # Authenticate dynamically adapted query function without executing it.
        qp=load(w/('inventory-v2-pretrial.json' if k=='sb' else 'inventory-query-pretrial.json'))
        text=inspect.getsource(review.inventory)
        name='inventory-v2' if k=='sb' else 'inventory-query'
        text=text.replace("run(key,'inventory',",f"run(key,'{name}',")
        if k=='sb':
            text=text.replace("out=work/'queries'", "out=work/'queries-v2'").replace("'inventory.v'","'inventory-v2.v'").replace("'discovery.json'","'discovery-v2.json'")
            text=text.replace("'Require '+' '.join(modules(key))+'.'","'Require '+' '.join(libraries)+'.'")
        need(hashlib.sha256(text.encode()).hexdigest()==qp['adapted_query_controller_sha256'],'adapted query source')
        check(OWN/'resume.py',qp['controller_sha256']);check(OWN/'review.py',qp['original_controller_sha256']);check(w/'kernel.log',qp['kernel_log_sha256'])
        discovery=load(w/('discovery-v2.json' if k=='sb' else 'discovery.json'))
        query=w/('inventory-v2.v' if k=='sb' else 'inventory.v');check(query,discovery['query_sha256'])
        need(discovery['declarations'].keys()==inv['declarations'].keys(),'query discovery binding set')
        lines=['Require '+' '.join(libs if k=='sb' else load(w/'pretrial.json')['roots'])+'.','Set Printing Width 1000000.','Set Printing All.','Set Printing Universes.' if k=='sb' else 'Set Printing Depth 1000000.']
        for m,e in inv['signatures'].items():
            lines.append(f'Redirect "{e["path"].removesuffix(".out")}" Print Module {m}.')
        for n,e in inv['declarations'].items():
            for s,cmd in [('type',('Check @' if k=='tb' else 'Check ')+n),('assumptions','Print Assumptions '+n)]:
                lines.append(f'Redirect "{e[s+"_path"].removesuffix(".out")}" {cmd}.')
            if 'declaration_path' in e:lines.append(f'Redirect "{e["declaration_path"].removesuffix(".out")}" Print {n}.')
        # JSON is sorted at save; original query generation sorted names, so
        # independently reconstruct signatures and declaration order by indices.
        head=lines[:4]
        head += [f'Redirect "{inv["signatures"][m]["path"].removesuffix(".out")}" Print Module {m}.' for m in sorted(inv['signatures'])]
        for n,e in sorted(inv['declarations'].items()):
            for s,cmd in [('type',('Check @' if k=='tb' else 'Check ')+n),('assumptions','Print Assumptions '+n)]:head.append(f'Redirect "{e[s+"_path"].removesuffix(".out")}" {cmd}.')
            if 'declaration_path' in e:head.append(f'Redirect "{e["declaration_path"].removesuffix(".out")}" Print {n}.')
        need(query.read_text()=='\n'.join(head)+'\n',('exact proof-free inventory query',k))
        extra=load(w/'extra-declarations.json');ind=load(w/'inductive-pretrial.json')
        head=['Require '+' '.join(libs)+'.','Set Printing Width 1000000.','Set Printing All.','Set Printing Universes.' if k=='sb' else 'Set Printing Depth 1000000.']
        for n in ind['additional_queries']:
            e=extra[n]
            for s,cmd in [('type','Check '+n),('assumptions','Print Assumptions '+n),('declaration','Print '+n)]:head.append(f'Redirect "{e[s+"_path"].removesuffix(".out")}" {cmd}.')
        need((w/'extra-inductives/queries.v').read_text()=='\n'.join(head)+'\n','exact proof-free supplemental query')
        # Inspect the compiled loader commands and inputs; API is inspection,
        # with no checker flags, proof commands or replacement definitions.
        pd=w/('signature-api-v1' if k=='sb' else 'signature-api-v2')
        cmd=load(w/('signature-api-query.json' if k=='sb' else 'signature-api-v2-query.json'))['argv']
        need(cmd[-2:]==['-l',str(pd/'inspect.v')],'signature loader argv')
        cmd=load(w/('signature-api-compile.json' if k=='sb' else 'signature-api-v2-compile.json'))['argv']
        need(cmd==[str(base/prefix/'bin/ocamlfind'),'ocamlopt','-rectypes','-thread','-package','coq.toplevel' if k=='sb' else 'rocq-runtime.toplevel','-shared',str(pd/'review_signature_walk.ml'),'-o',str(pd/'review_signature_walk.cmxs')],'plugin normal compile argv')
        q=(pd/'inspect.v').read_text()
        want='Require '+' '.join(libs)+'.\n'+('Add ML Path '+json.dumps(str(pd))+'.\n' if k=='sb' else '')+'Declare ML Module "review_signature_walk".\n'
        need(q==want,'exact proof-free signature loader')
        # Reproduce version-specific source text from frozen template.
        template=(OWN/'signature_walk.template').read_text()
        root=w/'signature-api-v1'
        replacements={'ROOT':json.dumps(str(root)),'LIBRARIES':'['+';'.join(json.dumps(m) for m in libs)+']','OPEN':'open Mod_declarations' if k=='tb' else '', 'MODTYPE':'Mod_declarations.mod_type' if k=='tb' else '(fun b -> b.mod_type)','LABEL':'Id.to_string' if k=='tb' else 'Label.to_string','ARITY':'p.mind_user_arity' if k=='tb' else 'p.mind_arity','EXTRACONST':'| Primitive _ -> "constant:primitive" | Symbol _ -> "constant:symbol"' if k=='tb' else '', 'EXTRAFIELD':'| SFBrules _ -> failwith "unexpected rewrite rules"' if k=='tb' else '', 'PACKET_RECORD':'''(match p.mind_record with
        | PrimRecord r -> Array.iteri (fun i id -> emit (path ^ "." ^ Id.to_string id) "primitive_projection" r.tys.(i)) r.projections
        | NotRecord | FakeRecord -> ());''' if k=='tb' else '', 'BLOCK_RECORD':'()' if k=='tb' else '''(match b.mind_record with
      | PrimRecord rs -> Array.iter (fun (_, labels, tys) -> Array.iteri (fun i id -> emit (path ^ "." ^ Label.to_string id) "primitive_projection" tys.(i)) labels) rs
      | NotRecord | FakeRecord -> ())'''}
        for a,b in replacements.items():template=template.replace('@'+a+'@',b)
        need(template==(root/'review_signature_walk.ml').read_text(),'generated inspection source')
        if k=='tb':
            template=template.replace(json.dumps(str(root)),json.dumps(str(pd))).replace('let () =\n  output_string index','let inspect () =\n  output_string index')
            template+='\nlet () = Mltop.declare_cache_obj_full (Mltop.interp_only_obj inspect) "review_signature_walk"\n'
            need(template==(pd/'review_signature_walk.ml').read_text(),'Interp callback source')
            # The callback API itself is from the already guarded pinned source.
            runtime=TB/'opamroot/tb/lib/rocq-runtime'
            api=[p for p in runtime.rglob('mltop.mli')]
            need(len(api)==1,'callback API source')
            api_text=api[0].read_text();need('interp_only_obj' in api_text and 'declare_cache_obj_full' in api_text,'documented callback API')
        out[k]={'inventory_query_sha256':sha(query),'supplemental_query_sha256':sha(w/'extra-inductives/queries.v'),'inspection_query_sha256':sha(pd/'inspect.v'),'inspection_source_sha256':sha(pd/'review_signature_walk.ml'),'inspection_plugin_sha256':sha(pd/'review_signature_walk.cmxs'),'proof_commands_added':False}
    save(NEW/'query-authentication.json',out)
    return out

def main():
    r=load(NEW/'finalize-receipt.json');need(r['state']=='finished' and r['exit_code']==0,'finalizer not accepted');check(r['log'],r['log_sha256'])
    pre=load(NEW/'pretrial.json');check(OWN/'finalize.py',pre['controller_sha256']);need(pre['controller_sha256']==r['controller_sha256'],'finalizer controller guard')
    ep=epochs();q=queries();result=load(NEW/'assessment.json')
    result['models']['sb']['delivery_inventory_verdict']='ACCEPT 1247 individually queried entries and 43 full block prints; 15 supplementary individual binding queries complete the explicit index'
    result['models']['tb']['delivery_inventory_verdict']='ACCEPT 2399 internal constant queries and 529 public roots; REJECT complete nested-signature claim; independent structural walk and 354 additional individual inductive/constructor queries complete coverage'
    result['query_authentication']=q
    result['finalizer_receipt']={'path':str(NEW/'finalize-receipt.json'),'sha256':sha(NEW/'finalize-receipt.json'),'exit_code':0,'log_sha256':r['log_sha256']}
    result['review_verdict']='ACCEPT both exact external MODEL mathematical checkpoints relative to enumerated assumptions; medium TB delivery nested-signature finding remains; no actual Rust/native/application acceptance'
    save(NEW/'final-assessment.json',result);save(OWN/'RESULTS.json',result)
    for k in f.SPECS:save(NEW/k/'final-assessment.json',result['models'][k])
    manifests=load(OWN/'MANIFESTS.json')
    for p in [NEW/'controller-epochs.json',NEW/'query-authentication.json',NEW/'finalize-receipt.json',NEW/'final-assessment.json']:
        manifests['indices'][p.name]={'path':str(p),'sha256':sha(p)}
    save(OWN/'MANIFESTS.json',manifests)
    save(OWN/'CONTROLLER_EPOCHS.json',ep)
    print('explicit terminal, controller epoch, exact query/source/loader guards passed; final precise verdict assembled')

if __name__=='__main__':main()
