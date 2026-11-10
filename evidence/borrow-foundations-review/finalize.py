#!/usr/bin/env python3
"""Read-only reauthentication and final collection of terminal predecessor replay.

Never executes old controllers, formal commands, or historical checkout paths.
All gates are explicit exceptions and remain active under Python -O.
"""
import collections, hashlib, json, os, pathlib, re, sys, tarfile, time
P = pathlib.Path
OWN = P(__file__).resolve().parent
DEL = OWN.parent.parent/'source/tools/research'
OLD = P('/tmp/borrow-foundations-review-20261110-ba6890b6')
NEW = P('/tmp/borrow-foundations-review-resume-20261110-e2ece9be')
SB = P('/tmp/stbor-checked-foundation-20261110')
TB = P('/tmp/tree-borrows-checked-foundation-20261110')
SPECS = {'sb': (SB, SB/'src/stacked-borrows', DEL/'stbor_checked_foundation', 'opamroot/legacy', 'stbor'),
         'tb': (TB, TB/'simuliris', DEL/'tree_borrows_checked_foundation', 'opamroot/tb', 'simuliris')}
VERIFIED = {}

def need(ok, detail):
    if not ok:
        raise RuntimeError(detail)

def sha(p):
    with P(p).open('rb') as f:
        return hashlib.file_digest(f, 'sha256').hexdigest()

def load(p):
    return json.loads(P(p).read_text())

def save(p, d):
    p = P(p)
    need(p.is_relative_to(NEW) or p.is_relative_to(OWN), ('write scope', str(p)))
    p.parent.mkdir(parents=True, exist_ok=True)
    p.write_text(json.dumps(d, indent=2, sort_keys=True)+'\n')

def allowed(p):
    p = P(p)
    need(any(p.is_relative_to(r) for r in [OLD, NEW, SB, TB, OWN, DEL]), ('read scope', str(p)))
    return p

def check(p, expected):
    p = P(p)
    actual = 'symlink:'+str(p.readlink()) if expected.startswith('symlink:') else sha(p)
    need(actual == expected, ('identity mismatch', str(p), expected, actual))
    VERIFIED[str(p)] = expected
    return expected

def guard(root, files, exact=False, tb=False):
    root = P(root)
    if exact:
        actual = {str(p.relative_to(root)) for p in root.rglob('*')
                  if (p.is_file() or (tb and p.is_symlink())) and (not tb or '.git' not in p.relative_to(root).parts)}
        need(actual == set(files), ('file set mismatch', str(root), sorted(actual ^ set(files))))
    for p, h in files.items():
        check(root/p, h)
    return {'count':len(files), 'map_sha256':hashlib.sha256(json.dumps(files, sort_keys=True).encode()).hexdigest()}

def linked(d):
    if isinstance(d, dict):
        p = d.get('path') or d.get('receipt')
        if p and d.get('sha256'):
            check(allowed(p), d['sha256'])
        for v in d.values():
            linked(v)
    elif isinstance(d, list):
        for v in d:
            linked(v)

def terminal(r, path):
    need(r['state'] == 'finished' and 'exit_code' in r and r['finished_unix'] >= r['started_unix'], ('not terminal', str(path)))
    proc = P('/proc')/str(r['pid'])/'stat'
    state = 'absent'
    if proc.exists():
        fields = proc.read_text().split(') ', 1)[1].split()
        need(fields[19] != str(r['startticks']) or fields[0] in ['Z', 'X'], ('job still live', str(path)))
        state = 'pid_reused' if fields[19] != str(r['startticks']) else 'terminal_'+fields[0]
    check(allowed(r['log']), r['log_sha256'])
    return state

def authenticate():
    out = {}
    for k, (base, src, delivery, prefix, logical) in SPECS.items():
        ready = load(base/'baseline-ready.json')
        check(base/'baseline-ready.json', {'sb':'9259b8f303e52e38f1b47709429f2be23a81a4e44ed557b6058184a95e653e51', 'tb':'f56490bb8441dbd73c37fe46fe970dc008bb8b2b67ee2b651a04c34d2b1ca523'}[k])
        linked(ready)
        if k == 'sb':
            need(ready['fully_ready'] is True and ready['normal_compilation_succeeded'] is True and ready['recursive_kernel_check_succeeded'] is True, 'SB not ready')
            need(ready['upstream_commit']=='d1471b10fe4cb4f506371b2e3acdd927d0a0c076', 'SB commit')
            check(base/'completed-build-receipt.json', '0d85e9ec05f29413973137f03bc840ccb1e22170d50169b2837331e58770f649')
            complete = load(base/'completed-build-receipt.json'); linked(complete)
            receipts = {n:load(e['path']) for n,e in complete['build_and_check_receipts'].items()}
            f = load(base/'manifests/upstream-tool-freeze.json')
            guards = {n:guard(b,m) for n,b,m in [('installed',base/prefix,f['installed_files']), ('initial_sources',base/'preflight-sources',f['dependency_initial_sources']), ('build_snapshots',base/'source-snapshots',f['dependency_build_sources']), ('downloads',base/'downloads',f['downloads']), ('own_controller',delivery,f['controller'])]}
            guards['platform'] = guard('/', load(base/'manifests/platform-toolchain.json')['files'])
            check(base/'opamroot/config', f['opam_config_sha256'])
            source = load(delivery/'upstream.json')['tracked_source_sha256']
            for p,e in load(base/'manifests/kernel-loaded-objects.json').items():
                check(allowed(p),e['object_sha256']);check(allowed(e['source']),e['source_sha256'])
            for e in load(base/'manifests/declaration-inventory.json').values():
                for s in ['type','assumptions']+(['declaration'] if e.get('print_full_declaration') else []):
                    check(base/f'declaration-output/{e["index"]}-{s}.out',e[s+'_sha256'])
            for filename in ['supplemental-controller-freeze.json','object-controller-freeze.json']:
                for label,h in load(base/'manifests'/filename)['files'].items():
                    marker='/source/tools/research/stbor_checked_foundation/'
                    p=delivery/label.split(marker,1)[1] if marker in label else allowed(label)
                    check(p,h)
            graph=load(base/'manifests/object-dependency-map.json')
            check(allowed(graph['reader']),graph['reader_sha256']);check(base/'object-reader-v3/vo_summary.ml',graph['reader_source_sha256'])
            need(len(graph['modules'])==518, 'SB object map size')
            for e in graph['modules'].values():
                need(set(e['recorded_compiled_dependencies'])<=set(graph['modules']), 'SB dependency outside closure')
            for name,e in load(base/'manifests/release-authentication.json').items():
                archive=base/'downloads'/({'coq':'coq-V8.9.1.tar.gz','equations':'equations-v1.2.1-8.9.tar.gz'}[name]);check(archive,e['archive_sha256']);seen=set()
                with tarfile.open(archive) as tar:
                    for member in tar:
                        if member.isfile():
                            p='/'.join(member.name.split('/')[1:]);b=tar.extractfile(member).read()
                            need(p in e['tracked_source_sha256'] and hashlib.sha256(b).hexdigest()==e['tracked_source_sha256'][p], ('release payload',name,p));seen.add(p)
                need(sorted(set(e['tracked_source_sha256'])-seen)==e['export_omitted'], 'archive omitted files')
                need(not any(p.endswith(('.v','.ml','.mli','.c','.h')) for p in e['export_omitted']), 'omitted program source')
        else:
            need(ready['ready'] is True and ready['normal_compilation'] is True and ready['full_recursive_kernel_check'] is True, 'TB not ready')
            need(ready['simuliris_commit']=='e3abfcdc301f92250d4227fc03c00463bf57b982', 'TB commit')
            receipts = {n:e['details'] for n,e in ready['receipts'].items()}
            for file,h in ready['hashes'].items():
                check(base/file if (base/file).is_file() else delivery/file,h)
            guards={p:guard(p,m) for p,m in load(base/'installed-pretrial.json').items()}
            guards['own_controller']=guard(delivery,ready['pretrial_versions_and_loadpaths']['support_files'])
            source=load(delivery/'upstream-source.json')['files']
            for p,h in load(base/'model-objects.json').items():check(src/p,h)
            linked(load(delivery/'INVENTORY.json'))
            for pkg,e in load(delivery/'dependencies.json').items():
                guard(allowed(e['metadata_root']),e['metadata_files'])
                if 'source_root' in e:
                    manifest=base/'source-manifests'/(pkg+'.json');check(manifest,e['source_manifest_sha256']);guard(allowed(e['source_root']),load(manifest))
            for name,e in load(delivery/'repositories.json').items():
                m=base/(name+'-tracked.json');check(m,e['manifest_sha256']);guard(base/'repositories'/name,load(m))
            guard('/',load(base/'loaded-objects.json'))
            for name,e in load(delivery/'SUPPORT.json').items():check(delivery/name,e['sha256'])
            classes=load(delivery/'ASSUMPTIONS.json')['individual_constant_classes']
            for e in load(base/'inventory-queries.json')['internal_constants']:check(allowed(e['assumptions']),classes[e['name']])
        for n,r in receipts.items():
            need(r.get('exit_status',r.get('exit_code'))==0, ('baseline receipt failed',k,n))
            check(allowed(r['log']),r['log_sha256'])
        guards['upstream']=guard(src,source)
        need(guards==load(OLD/'authentication.json')[k]['guards'], ('original authentication guard differs',k))
        need(load(OLD/k/'source.json')==source, ('fresh source map',k))
        out[k]={'ready_sha256':sha(base/'baseline-ready.json'),'guards':guards,'successful_receipts':list(receipts)}
        print(k,'baseline identity/provenance reauthenticated',flush=True)
    save(NEW/'authentication.json',out)

def receipts_first():
    out={}; failures={'sb/inventory-rejected','sb/nested-round-1','sb/nested-v2-round-2','tb/signature-api-query','tb/signature-api-query-rocq'}
    for k in SPECS:
        for p in sorted((OLD/k).glob('*.json')):
            d=load(p)
            if not isinstance(d,dict) or 'argv' not in d:continue
            name=k+'/'+p.stem; state=terminal(d,p)
            need((d['exit_code']!=0)==(name in failures), ('unexpected exit',name,d['exit_code']))
            check(OWN/'review.py',d['controller_sha256'])
            need(d['cwd']==str(OLD/k/'src'),'receipt cwd')
            need(d['environment']=={'LANG':'C.UTF-8','PATH':str(SPECS[k][0]/SPECS[k][3]/'bin')+':/usr/bin:/bin','TMPDIR':str(OLD/k/'tmp'),'XDG_CACHE_HOME':str(OLD/k/'cache'),'PYTHONDONTWRITEBYTECODE':'1','OPAMJOBS':'2',**({'LD_LIBRARY_PATH':str(TB/'deps/lib')} if k=='tb' else {})},('environment',name))
            out[name]={**d,'receipt':str(p),'receipt_sha256':sha(p),'observed_process_state':state,'classification':'query_setup_failure' if d['exit_code'] else 'successful_command'}
    save(NEW/'receipts.json',out)
    print('terminal states/logs authenticated for',len(out),'receipts',flush=True)
    return out

def collect():
    out={};tools={}
    for k,(base,oldsrc,delivery,prefix,logical) in SPECS.items():
        w=OLD/k;inv=load(w/'inventory.json');sig=load(w/'signature-api-inventory.json');pre=load(w/'pretrial.json')
        check(OWN/'review.py',pre['controller_sha256'])
        check(OWN/'signatures.py',sig['controller_sha256']);check(OWN/'signature_walk.template',sig['template_sha256'])
        ipre=load(w/'signature-api-pretrial.json')
        need(ipre['inventory_sha256']==sha(w/'inventory.json') and ipre['normal_kernel_receipt_sha256']==sha(w/'kernel.json'),'signature original input binding')
        need(ipre['controller_sha256']==sig['controller_sha256'] and ipre['template_sha256']==sig['template_sha256'],'signature original support binding')
        pluginroot=w/('signature-api-v1' if k=='sb' else 'signature-api-v2')
        check(pluginroot/'review_signature_walk.ml',sig['source_sha256']);check(pluginroot/'review_signature_walk.cmxs',sig['plugin_sha256'])
        check(sig['index'],sig['index_sha256']);check(sig['types'],sig['types_sha256'])
        if k=='tb':
            # Rehome this historical controller label; never open its old checkout.
            check(OWN/'rocq_signature_v2.py',sig['loader_sha256'])
            vp=load(w/'signature-api-v2-pretrial.json')
            need(vp['kernel_receipt_sha256']==sha(w/'kernel.json') and vp['source_sha256']==sig['source_sha256'],'v2 input binding')
            check(pluginroot/'inspect.v',vp['query_sha256']);check(pluginroot/'findlib/review_signature_walk/META',vp['descriptor_sha256'])
            check(w/'signature-api-v1/review_signature_walk.ml',vp['original_generated_source_sha256'])
        else:need(ipre['generated_source_sha256']==sig['source_sha256'],'SB plugin source input binding')
        check(w/'signature-api-v1/review_signature_walk.ml',ipre['generated_source_sha256'])
        need((pluginroot/'inspect.v').read_text().startswith('Require '+' '.join(inv['libraries'])+'.\n'),'signature imports')
        rows=P(sig['index']).read_text().splitlines();need(rows[0]=='name\tkind\toffset\tbytes','index header')
        counts=collections.Counter();found={};offset=0;payload_bytes=P(sig['types']).stat().st_size
        for row in rows[1:]:
            n,kind,o,l=row.split('\t');o=int(o);l=int(l)
            need(o==offset and l>0, ('signature payload range',k,n));offset+=l;counts[kind]+=1
            if n.startswith(logical+'.') and '/$' not in n and (kind.startswith('constant:') or kind in ['inductive','constructor','primitive_projection']):
                need(n not in found, ('duplicate binding',k,n));found[n]=kind
        need(offset==payload_bytes and len(rows)-1==sig['rows'] and counts==sig['counts'],'signature exact coverage/counts')
        need(sig['libraries']==inv['libraries'],'signature library set')
        ind=load(w/'inductive-pretrial.json');check(OWN/'inductives.py',ind['controller_sha256'])
        need(ind['signature_inventory_sha256']==sha(w/'signature-api-inventory.json') and ind['found']==found,'supplemental input binding')
        fresh=inv['declarations']|load(w/'extra-declarations.json')
        need(set(fresh)==set(found),'complete individual binding set')
        need(ind['additional_queries']==sorted(set(found)-set(inv['declarations'])),'supplemental query set')
        classes={};classcounts=collections.Counter();displaydiff={'type':[],'assumptions':[],'declaration':[]};declrows=[]
        delivered=load(base/'manifests/declaration-inventory.json') if k=='sb' else {e['name']:e for e in load(base/'inventory-queries.json')['internal_constants']}
        need(set(delivered)<=set(fresh),'missing delivered binding')
        for n,e in sorted(fresh.items()):
            for s in ['type','assumptions']+(['declaration'] if 'declaration_path' in e else []):
                check(allowed(e[s+'_path']),e[s+'_sha256'])
                text=P(e[s+'_path']).read_text();need(text.strip() and 'Error:' not in text,('query output',k,n,s))
                if n in delivered:
                    old=delivered[n]
                    if s=='declaration' and (k=='tb' or not old.get('print_full_declaration')):continue
                    oldp=base/f'declaration-output/{old["index"]}-{s}.out' if k=='sb' else P(old[s])
                    if sha(oldp)!=e[s+'_sha256']:displaydiff[s].append(n)
            a=P(e['assumptions_path']).read_text();h=e['assumptions_sha256']
            need(a=='Closed under the global context\n' or a.startswith('Axioms:\n'),('assumption output',k,n))
            classes[h]=a;classcounts[h]+=1
            declrows.append('\t'.join([n,found[n],h,e['type_sha256'],e.get('declaration_sha256','-')]))
        for name,e in inv['signatures'].items():check(allowed(e['path']),e['sha256'])
        (OWN/(k+'-declarations.tsv')).write_text('name\tkind\tassumptions_sha256\ttype_sha256\tfull_declaration_sha256\n'+'\n'.join(declrows)+'\n')
        save(OWN/(k+'-assumptions.json'),{h:{'count':classcounts[h],'verbatim':a} for h,a in classes.items()})
        klog=(w/'kernel.log').read_text()
        need('Modules were successfully checked' in klog and 'Admitting library:' not in klog and 'CONTEXT SUMMARY' in klog,'full kernel log')
        context='CONTEXT SUMMARY'+klog.split('CONTEXT SUMMARY',1)[1]
        original=('CONTEXT SUMMARY'+load(base/'completed-build-receipt.json')['kernel_context_summary']) if k=='sb' else load(base/'baseline-ready.json')['kernel_context_verbatim']
        need(context.strip()==original.strip() and 'Set is predicative' in context,'kernel exact context')
        if k=='tb':
            for item in ['Rewrite rules are not allowed','Constants/Inductives relying on type-in-type: <none>','Constants/Inductives relying on unsafe (co)fixpoints: <none>','Inductives whose positivity is assumed: <none>']:need(item in context,('kernel safety mode',item))
        libraries=sorted(set(re.findall(r'^Checking library: (\S+)',klog,re.M)))
        constants=sorted(set(re.findall(r'checking cst:(\S+)',klog)))
        need(libraries==inv['libraries'] and constants==inv['constants'],'kernel discovery')
        oldlibs=load(base/'manifests/kernel-coverage.json')['checked_modules'] if k=='sb' else load(base/'inventory-queries.json')['all_checked_libraries']
        need(libraries==sorted(oldlibs),'baseline library equality')
        loads={}
        for label in re.findall(r'\[intern (.*?) \.\.\.',klog):
            p=P(label);p=p if p.is_absolute() else w/'src'/p
            need(p.is_relative_to(w/'src') or p.is_relative_to(base/prefix),'loaded namespace')
            s=p.with_suffix('.v');loads[str(p)]={'object_sha256':sha(p),'source_sha256':sha(s) if s.exists() else None}
        need(len(loads)==len(libraries),'loaded object closure')
        save(NEW/k/'loaded-objects.json',loads)
        objects=load(w/'objects.json');need(len(objects)==(44 if k=='sb' else 71),'project object count')
        need({str(p.relative_to(w/'src')) for p in (w/'src').rglob('*.vo')}==set(objects),'object file set')
        for p,h in objects.items():check(w/'src'/p,h);check(oldsrc/p,h)
        final=load(w/'final-guards.json');check(OWN/'review.py',final['controller_sha256'])
        sources=load(w/'source.json');sg=guard(w/'src',sources)
        need(sg==pre['source_guard']==final['source'],'source guard')
        expected={p for p in sources if p.endswith('.v')};actual={str(p.relative_to(w/'src')) for p in (w/'src').rglob('*.v')}
        need(expected==actual,'proof source file set')
        installed={b:guard(b,m,exact=True,tb=k=='tb') for b,m in load(w/'installed-before.json').items()}
        need(installed==pre['installed_guard']==final['installed'],'installed before/after identity')
        need(guard(w/'src',objects)==final['objects'],'object final guard')
        post=load(OLD/'postguards.json');check(OWN/'postguards.py',post['controller_sha256'])
        need(post['guards'][k]=={'installed_file_sets_and_content':installed,'upstream_proof_sources_exact_set':len(expected),'proof_source_guard':sg,'objects':final['objects']},'postguard comparison')
        # Exact build/kernel vectors are independently reconstructed, with no exclusions.
        bin=base/prefix/'bin';roots=pre['roots']
        expected_build=['/usr/bin/make','-j2'] if k=='sb' else ['/usr/bin/make','-f','Makefile.rocq','-j2']+['theories/'+m.removeprefix('simuliris.').replace('.','/')+'.vo' for m in roots]
        need(load(w/'build.json')['argv']==expected_build,'normal build argv')
        if k=='sb':
            lib=base/prefix/'lib/coq';deps=sorted('.'.join(p.relative_to(lib/'user-contrib').with_suffix('').parts) for p in (lib/'user-contrib').rglob('*.vo'))
            expected_kernel=[str(bin/'coqchk'),'-coqlib',str(lib),'-Q',str(w/'src/theories'),logical,'-o']+sorted(set(roots+deps))
            need(len(roots)==44 and len(deps)==250,'SB normal root closure')
        else:expected_kernel=[str(bin/'rocq'),'check','-Q',str(w/'src/theories'),logical,'-o']+roots
        need(load(w/'kernel.json')['argv']==expected_kernel,'full recursive kernel argv')
        versions=[]
        for p in sorted(w.glob('tool-version-*.json')):
            r=load(p);tool=P(r['argv'][0]);versions.append({'argv':r['argv'],'output':P(r['log']).read_text(),'binary_sha256':sha(tool)})
        tools[k]={'versions':versions,'loadpath':(w/'loadpath.log').read_text()}
        out[k]={'verdict':'ACCEPT exact external MODEL mathematical checkpoint relative to reported assumptions', 'delivery_inventory_verdict':'REJECT complete individual binding claim; supplemented independently', 'normal_build':True,'full_recursive_kernel':True,'exact_binary_object_equality':len(objects),'libraries':len(libraries),'all_kernel_constants':len(constants),'individual_bindings':len(fresh),'delivered_individual_bindings':len(delivered),'prior_independent_bindings':len(inv['declarations']),'supplemental_bindings':ind['additional_queries'],'additional_vs_delivery':sorted(set(fresh)-set(delivered)),'assumption_classes':dict(classcounts),'display_differences':displaydiff,'structural_signature_fields':len(rows)-1,'structural_signature_counts':dict(counts),'kernel_context':context,'installed_final_guards':installed,'source_final_guard':sg,'objects_final_guard':final['objects'],'signature_inspection_only':True}
        save(NEW/k/'assessment.json',out[k]);save(OWN/(k+'-signatures.json'),sig)
        print(k,'complete model acceptance assembled:',len(fresh),'bindings;',len(rows)-1,'signature fields;', {a:len(v) for a,v in displaydiff.items()},flush=True)
    save(OWN/'TOOLS.json',tools)
    return out

def main():
    need(NEW.is_dir(),'launch must create fresh private root and pretrial freeze')
    receipts=receipts_first()  # Terminal state established before using old semantic results.
    authenticate()
    out=collect()
    # All private predecessor output bytes, including rejected setup products, are
    # indexed read-only; directories and installed maps are separately guarded.
    private={str(p.relative_to(OLD)):sha(p) for p in sorted(OLD.rglob('*')) if p.is_file()}
    save(NEW/'predecessor-files.json',private)
    save(NEW/'verified-files.json',VERIFIED)
    save(NEW/'support.json',{p.name:sha(p) for p in sorted(OWN.iterdir()) if p.is_file()})
    result={'scope':'independent exact external MODEL only','accepted':True,'models':out,'fresh_builds':'authenticated terminal predecessor independent builds; no unnecessary rebuild on resumption','private_root':str(NEW),'predecessor_private_root':str(OLD),'native_correspondences':0,'minimal_open':44,'phase_open':3,'writer_effects_open':101,'canonical':'inherited 6/0; every u64/arbitrary initialized exclusive [u8;10]; unchanged body/domain/predicate; no application replay here','HMAC':'inherited 1/1 UNPROVED; independently frozen bitcoin_hashes0.14 actual pad-XOR; arbitrary initialized [u8;128]; EVERY valid key length including0/>128; XOR prefix AND unchanged suffix; no application replay here','publication':False,'goal_complete':False}
    save(NEW/'assessment.json',result)
    save(OWN/'RESULTS.json',result)
    save(OWN/'COMMANDS.json',receipts)
    save(OWN/'MANIFESTS.json',{'private_root':str(NEW),'indices':{p.name:{'path':str(p),'sha256':sha(p)} for p in [NEW/'authentication.json',NEW/'receipts.json',NEW/'predecessor-files.json',NEW/'verified-files.json',NEW/'support.json',NEW/'assessment.json',NEW/'pretrial.json']},'notice':'Full private indices contain every predecessor file and every rehashed source/tool/dependency/receipt identity. Historical checkout strings are labels, never followed.'})
    print('FINAL: both exact MODEL checkpoints accepted; delivery completeness findings retained; native=0, publication=false, goal_complete=false',flush=True)

if __name__=='__main__':main()
