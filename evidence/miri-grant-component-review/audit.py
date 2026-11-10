"""Independent saved-output, source, scope, and assumption acceptance.

Imports no delivered replay/acceptance code. Explicit checks survive Python -O.
This is a bounded reviewer, not a Rust equivalence checker or native bridge.
"""
import ast
import hashlib
import json
from pathlib import Path
import re
import sys

sys.dont_write_bytecode = True
HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[1]
RESEARCH = ROOT/'source/tools/research'
PRIVATE = Path('/tmp/miri-grant-component-review-t3-8295fb02')
A = RESEARCH/'native_miri_grant_refinement'
B = RESEARCH/'native_miri_permission_grants'
C = RESEARCH/'native_entry_type_binding'
EXEC = {'Item::perm','Item::protected','Item::set_permission','Permission::from_bits',
        'Permission::to_bits','Stack::get','Stack::find_first_write_incompatible',
        '<Item as std::clone::Clone>::clone','<Permission as std::clone::Clone>::clone'}
SPEC = {'decode','bits','Item::permission'}

def need(ok, msg):
    if not ok:
        raise ValueError(msg)

def digest(p):
    return hashlib.sha256(Path(p).read_bytes()).hexdigest()

def read(p):
    return json.loads(Path(p).read_text())

def write(p, data):
    Path(p).write_text(json.dumps(data, indent=2, sort_keys=True)+'\n')

def guard(mapping):
    for p,h in mapping.items():
        need(digest(p) == h, 'input drift '+p)

def receipt(p):
    d=read(p)
    need(d['success'] is True, 'controller failure '+str(p))
    for key,end in [('artifacts','artifacts_after'),('inventory_inputs','inventory_inputs_after')]:
        if end in d:
            need(d[key] == d[end], 'before/after artifact mismatch')
            for path,info in d[key].items():
                need(digest(path) == info['sha256'], 'current receipt input mismatch '+path)
    for cmd in d['commands']:
        rc=cmd.get('returncode',cmd.get('exit_code'))
        need(type(rc) is int and not cmd.get('timed_out') and not cmd.get('interrupted'), 'nonterminal command')
        for suffix in ['stdout','stderr']:
            need((Path(p).parent/(cmd['label']+'.'+suffix)).is_file(), 'missing raw log')
    return d

def one(pattern):
    paths=list((PRIVATE/'grant').glob(pattern))
    need(len(paths)==1, 'ambiguous own epoch '+pattern)
    return paths[0]

def clean_errors(stderr):
    need(not any(s in stderr.lower() for s in ['panicked at','resource limit','timed out','out of memory']), 'resource failure')
    return re.findall(r'^error: (?!aborting)(.+)$',stderr,re.M)

def verify(out, label, expected_exec, expected_specs, verified, failed=(), kind=None):
    d=read(out/(label+'.stdout')); err=(out/(label+'.stderr')).read_text()
    v=d['verification-results']; inf=d['verus-infer']; s=inf['verification_scope']
    positive=not failed
    need(v['is-verifying-entire-crate'] is True and v['encountered-vir-error'] is False, 'ordinary whole crate')
    need(v['verified']==verified and v['errors']==len(failed) and v['success'] is positive, 'query counts')
    need(inf['crate_success'] is positive and inf['functions']==[] and inf['requested_functions']==0 and inf['generated_functions']==0, 'unexpected inference')
    need(inf['interface_written'] is False, 'publication')
    for k,value in inf.items():
        if k.endswith('_assistance') and isinstance(value,list):
            need(value==[], 'unexpected assistance '+k)
    need(s['native_collection_completed'] is True and s['complete_selected_scope_verified'] is positive, 'scope flags')
    selected={x['rust_function']:x for x in s['native_functions'] if x['selected']}
    bodies={n:x for n,x in selected.items() if x['body_status'] not in ['specification_definition_or_declaration','selected_item_without_translated_body']}
    need(set(bodies)==expected_exec and len(bodies)==len(expected_exec), 'complete exec inventory')
    need({n for n,x in selected.items() if x['body_status']=='specification_definition_or_declaration'}==expected_specs,'spec inventory')
    need(s['checked_proof_bodies']==0, 'unexpected proof function')
    status='checked_exec_body' if positive else 'selected_body_not_certified_by_successful_full_run'
    need(all(x['body_status']==status and x['ordinary_contract'] is True for x in bodies.values()), 'body/contract coverage')
    need(s['checked_exec_bodies']==(len(expected_exec) if positive else 0), 'certified body count')
    target=[bodies[name]['function'] for name in failed]
    need(inf['failed_functions']==target, 'exact failed_functions')
    errors=clean_errors(err)
    if positive:
        need(errors==[], 'positive diagnostic')
    else:
        need(errors and all(e==kind for e in errors), 'mixed failure kind')
    spans=re.findall(r'^error: (?!aborting)([^\n]+)\n\s*--> (.+?\.rs):(\d+):(\d+)',err,re.M)
    need(len(spans)==len(errors),'unlocated diagnostic')
    return {'verification':v, 'failed_functions':target, 'diagnostics':[{'kind':k,'source':p,'line':int(l),'column':int(c)} for k,p,l,c in spans],
            'exec_inventory':sorted(bodies), 'spec_inventory':sorted(expected_specs),
            'stdout_sha256':digest(out/(label+'.stdout')),'stderr_sha256':digest(out/(label+'.stderr')),
            'source_sha256':digest(out/('component.rs' if label=='verify' else label+'.rs')),
            'native_inventory':s['native_functions'] if positive else None}

def tokens(text):
    # Selected snippets contain no raw strings or comment-like string content.
    return re.findall(r'"(?:[^"\\]|\\.)*"|\w+|[^\s]', re.sub(r'//[^\n]*','',text))

def balanced(text,start):
    depth=0
    for i in range(start,len(text)):
        depth+=(text[i]=='{')-(text[i]=='}')
        if depth==0:
            return text[start:i+1]
    raise ValueError('unterminated selected block')

def source_audit():
    f=read(A/'FREEZE.json'); selected=read(A/'selected.json'); base=(A/'component.rs').read_text(); rows={}
    for name,extracted in selected.items():
        file='mod.rs' if name=='find_first_write_incompatible' else ('stack.rs' if name=='get' else 'item.rs')
        full=Path(f['sources'][file]['path']).read_text()
        need(extracted in full, 'not literal source extraction '+name)
        start=base.index('\n{',base.index('fn '+name+'('))+1
        body=balanced(base,start)
        while (m:=re.search(r'\bproof\s*\{',body)):
            b=body.index('{',m.start()); block=balanced(body,b)
            body=body[:m.start()]+body[b+len(block):]
        if name=='find_first_write_incompatible':
            body=re.sub(r'(while let Some\(item\) = self.get\(idx\))\s+invariant\b.*?\n\s*\{',r'\1 {',body,flags=re.S)
        original=balanced(extracted,extracted.index('{'))
        need(tokens(body)==tokens(original), 'independent token mismatch '+name)
        rows[name]={'upstream':file,'tokens_identical':True,'upstream_body_sha256':hashlib.sha256(original.encode()).hexdigest()}
    need(digest(A/'component.rs')=='23880c5435a385228b5bcd35da75a0862bc56a3a2755fa4aa1684a2c18d59092','complete template')
    need(len(re.findall(r'\bproof\s*\{',base))==5,'manual block count')
    need(not re.search(r'\b(?:assume|admit|external_body|assume_specification|exec_allows_no_decreases_clause)\b',base),'forbidden body')
    # Check source-cut primary bytes only through the own committed pin map.
    pin=read(RESEARCH/'native_source_reborrow/pins.json')
    place=next(x for x in pin['rust_sources'] if x['upstream'].endswith('/interpret/place.rs'))
    need(digest(place['path'])==place['sha256'],'copy source pin')
    text=Path(place['path']).read_text()
    lines={s:text[:text.index(s)].count('\n')+1 for s in ['fn copy_op_no_validate','let src_has_padding =','BackendRepr::Scalar','fn copy_op_inner','fn write_immediate_no_validate']}
    return {'grant_selected':rows,'manual_inline_blocks':5,'authored_specs':sorted(SPEC),'manual_library_calls':['vstd::std_specs::vec::axiom_spec_len'],
            'copy_source':dict(place, inspected_lines=lines),'bridge_source_sha256':digest(A/'GrantBridge.v')}

def audit():
    frozen=read(HERE/'pretrial.json'); guard(frozen['inputs'])
    audit_frozen=read(HERE/'audit-freeze.json'); guard(audit_frozen['inputs'])
    result={'schema':1,'classification':'bounded_independent_component_acceptance','publication':False,'goal_complete':False,'native_correspondences':0,'terminals':{},'grant':{},'permission':{},'carrier':{},'coq':{}}
    for label in ['grant-suite','grant-mutations','coq','permission','carrier']:
        p=PRIVATE/(label+'-terminal.json'); d=read(p)
        need(d['status']=='finished' and d['returncode']==0 and d['final_guards'] is True,'top terminal')
        need(d['before']==digest(HERE/'pretrial.json'),'pretrial binding')
        for ext,h in d['logs'].items():
            need(digest(PRIVATE/(label+'.'+ext))==h,'terminal log')
        result['terminals'][label]=dict(d,receipt=str(p),sha256=digest(p))
    result['source']=source_audit()
    suite=receipt(one('suite-*/receipt.json')); base=(A/'component.rs').read_text().rstrip()[:-1]
    grant=[]; excluded=[]
    for p in sorted((PRIVATE/'grant').glob('*/receipt.json')):
        d=receipt(p)
        if 'input' not in d:
            continue
        out=p.parent; kind=d['input']['kind']
        if kind in ['frontend','no_manual']:
            data=read(out/'verify.stdout'); err=(out/'verify.stderr').read_text()
            need(d['returncode']==1 and data['verification-results']['success'] is False,'excluded failure state')
            if kind=='frontend':
                need(data['verification-results']['encountered-vir-error'] is True and 'loop must have a decreases clause' in err,'frontend class')
            else:
                need(data['verification-results']['verified']==14 and data['verification-results']['errors']==3,'causality failure counts')
            excluded.append({'receipt':str(p),'sha256':digest(p),'classification':kind+'_not_semantic_control','verification':data['verification-results'],'diagnostic_kinds':clean_errors(err)})
            continue
        name=d['input']['extra']; failure=[name] if name else []
        row=verify(out,'verify',EXEC|set(failure),SPEC,22,failure,'assertion failed' if name else None)
        if name:
            text=(out/'component.rs').read_text(); start=text[:text.index('fn '+name+'(')].count('\n')+1
            need(text.startswith(base) and all(x['line']>=start and Path(x['source'])==out/'component.rs' for x in row['diagnostics']),'property failure escaped appended control')
        else:
            need(row['source_sha256']==digest(A/'component.rs'),'positive template')
        row.update(receipt=str(p),receipt_sha256=digest(p),assistance=d['input']['assistance'],kind=kind)
        grant.append(row)
    need(len(grant)==5 and len(excluded)==3,'suite epoch inventory')
    mutations=receipt(one('mutations-*/receipt.json')); mutants={}
    for label,n in [('boundary',23),('permission',22),('bitfield',25)]:
        item=mutations['mutations'][label]; out=Path(mutations['output']); symbol=item['symbol']
        row=verify(out,label,EXEC|{symbol},SPEC,n,[symbol],'postcondition not satisfied')
        text=Path(item['source']).read_text(); start=text[:text.index('fn '+symbol.split('::')[-1]+'(')].count('\n')+1
        need(text.startswith(base) and all(x['line']>=start and x['source']==item['source'] for x in row['diagnostics']),'mutant diagnostic escaped own function')
        row['source_mutation']={k:item[k] for k in ['symbol','original','before','after','original_function_sha256']}
        mutants[label]=row
    result['grant']={'accepted':grant,'semantic_mutations':mutants,'excluded':excluded,'suite_receipt_sha256':digest(one('suite-*/receipt.json'))}
    native=receipt(one('native-*/receipt.json'))
    need(native['build_exits']==[0,0] and native['run_exits']==[0,0] and native['outputs_equal'] is True and native['observation']=='cases=77394 checksum=4520926185573745023\n','finite support')
    result['grant']['finite_support']={k:native[k] for k in ['observation','binary_hashes','build_exits','run_exits','classification']}
    bp=PRIVATE/'permission'; b=receipt(bp/'receipt.json'); permission=[]
    for r in b['runs']:
        row=verify(bp,r['name'],{'Permission::grants','<Permission as std::clone::Clone>::clone','<AccessKind as std::clone::Clone>::clone'}, {'permission_access_table'},3 if r['expected'] else 2, [] if r['expected'] else ['Permission::grants'],None if r['expected'] else 'postcondition not satisfied')
        need(len(row['diagnostics'])==(0 if r['expected'] else 1),'permission single diagnostic')
        for diag in row['diagnostics']:
            need(diag['line']==33 and Path(diag['source'])==bp/(r['name']+'.rs'),'permission diagnostic span')
        row.update(name=r['name'],enabled=r['enabled']);permission.append(row)
    need(len(permission)==6 and len(b['canonical_support_key_mapping'])==2,'permission six cases/mapping')
    result['permission']={'accepted':permission,'canonical_support_key_mapping':b['canonical_support_key_mapping'],'receipt_sha256':digest(bp/'receipt.json'),'R1':'resolved','R2':'resolved'}
    coq_path=one('coq-*/receipt.json'); coq=receipt(coq_path); out=coq_path.parent
    old=read('/tmp/native-miri-grant-resumed-pa16x1fp/coq-f8wu9d37/receipt.json')
    need(coq['compile_returncode']==coq['check_returncode']==0,'kernel exits')
    source=(out/'GrantBridge.v').read_text(); names=re.findall(r'^(?:Definition|Lemma|Theorem) (\w+)',source,re.M)
    glob=re.findall(r'^(?:def|prf) \S+ <> (\w+)$',(out/'GrantBridge.glob').read_text(),re.M)
    kernel=(out/'coqchk.stderr').read_text()
    need(len(names)==7 and set(names)==set(glob)==set(coq['declarations']),'complete own glob inventory')
    need(set(re.findall(r'checking cst:GrantRefinement.GrantBridge\.(\S+)',kernel))==set(names),'kernel own declarations')
    need('Modules were successfully checked' in kernel,'kernel marker')
    need(not re.search(r'\b(?:Axiom|Admitted|admit|Parameter|Hypothesis)\b',source),'own assumption declaration')
    declarations={}
    for name,d in coq['declarations'].items():
        row={k:d[k] for k in ['kind','source_sha256','object_sha256','type_text','type_sha256','term_sha256','assumptions_sha256']}
        for kind in ['type','term','assumptions']:
            need(digest(d[kind+'_path'])==d[kind+'_sha256']==old['declarations'][name][kind+'_sha256'],'fresh/historical exact term '+name)
        need(Path(d['assumptions_path']).read_text().strip()=='Closed under the global context','own assumption closure')
        row['closed']=True;declarations[name]=row
    counts={k:len(coq[k]) for k in ['kernel_libraries','kernel_constants','kernel_inductive_blocks']}
    need(list(counts.values())==[263,41987,274],'recursive closure inventory')
    result['coq']={'receipt':str(coq_path),'receipt_sha256':digest(coq_path),'declarations':declarations,'counts':counts,'source_sha256':coq['source_sha256'],'object_sha256':coq['object_sha256'],'inherited_context':'263-library superset recursively checked; seven own items CLOSED. Broader historical foundation includes proof irrelevance, dependent functional extensionality, classical EM, Eq_rect_eq, Equations funext and admitted FixWf_unfold; no new assumption closure follows for other exports.'}
    cp=PRIVATE/'carrier'; c=receipt(cp/'receipt.json')
    need(len(c['commands'])==9 and [x['exit_code'] for x in c['commands']]==[0,0,0,0,0,0,1,1,1],'nine carrier outcomes')
    queries={n:read(cp/(n+'.json')) for n in ['canonical','writer','probes']}
    for n,q in queries.items():
        need(q==c['queries'][n] and q['pointer_bits']==64 and q['publication'] is False,'query product binding')
    need(queries['canonical']['records']==read(C/'carrier-results.json')['actual_parameters']['canonical'] and queries['writer']['records']==read(C/'carrier-results.json')['actual_parameters']['writer'],'actual parameters exact')
    for cmd in c['commands'][-3:]:
        err=(cp/(cmd['label']+'.stderr')).read_text()
        need('error[E0277]' in err and not any(s in err for s in ['error[E0463]','panicked at','aborting due to 2']),'ordinary trait negative')
    result['carrier']={'receipt_sha256':digest(cp/'receipt.json'),'commands':c['commands'],'queries':queries,'ordinary_rejections':c['ordinary_rejections'],'scope':'compiler parameter/subject type traits and layouts only; source-cut inspection only'}
    guard(frozen['inputs']);guard(audit_frozen['inputs'])
    # Full raw product inventory stays private; only its identity is committed.
    products={str(p.relative_to(PRIVATE)):digest(p) for p in sorted(PRIVATE.rglob('*')) if p.is_file() and p.name!='product-hashes.json'}
    write(PRIVATE/'product-hashes.json',products)
    result['private_products']={'path':str(PRIVATE/'product-hashes.json'),'sha256':digest(PRIVATE/'product-hashes.json'),'count':len(products)}
    result['guards']={'pretrial_sha256':digest(HERE/'pretrial.json'),'audit_freeze_sha256':digest(HERE/'audit-freeze.json'),'all_final_guards_passed':True,'python_optimized':bool(sys.flags.optimize)}
    write(HERE/'results.json',result)
    print('Independent audit accepted bounded A/B/C; no whole-goal acceptance')

if __name__=='__main__':
    audit()
