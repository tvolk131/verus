#!/usr/bin/env python3
"""Independent bounded review. Historical checkout paths are never opened."""
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import sys
import time

sys.dont_write_bytecode = True
HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[1]
RESEARCH = ROOT/'source/tools/research'
RAW = Path('/tmp/source-replay-exit-review-bc2a0b37-002')
LANES = {
    'constructor': ('native_constructor_erasure', 'integration001-freeze.json', 'NativeConstructorErasure', ['Interface','Graph','Composition','Boundary']),
    'pointer': ('native_pointer_distance', 'integration003-freeze.json', 'NativePointerDistance', ['distance','branches']),
    'exit': ('native_original_reference_exit', 'integration001-freeze.json', 'NativeOriginalReferenceExit', ['frontend','resources','source']),
}

def require(ok, message):
    if not ok:
        raise ValueError(message)


def sha(p):
    return hashlib.sha256(Path(p).read_bytes()).hexdigest()


def read(p):
    return json.loads(Path(p).read_text())


def write(p, value):
    Path(p).write_text(json.dumps(value, indent=2, sort_keys=True)+'\n')


def pin(p):
    return {'path':str(p), 'sha256':sha(p)}


def guard():
    for p,h in read(RAW/'pretrial.json')['files'].items():
        require(sha(p)==h, 'retained input changed '+p)


def run(label, argv, cwd=ROOT):
    guard()
    start=time.monotonic()
    env=dict(os.environ, PYTHONDONTWRITEBYTECODE='1')
    for k in ['COQPATH','COQLIB','COQCORELIB','OCAMLPATH','CAML_LD_LIBRARY_PATH','LD_PRELOAD','LD_LIBRARY_PATH']:
        env.pop(k,None)
    with (RAW/(label+'.stdout')).open('w') as out, (RAW/(label+'.stderr')).open('w') as err:
        p=subprocess.run(argv,cwd=cwd,env=env,stdout=out,stderr=err,timeout=1800)
    result={'argv':list(map(str,argv)),'cwd':str(cwd),'exit_code':p.returncode,
            'seconds':time.monotonic()-start, 'stdout':pin(RAW/(label+'.stdout')),
            'stderr':pin(RAW/(label+'.stderr'))}
    write(RAW/(label+'-command.json'),result)
    guard()
    require(p.returncode==0, label+' failed; see durable command receipt')
    return result


def prepare():
    RAW.mkdir(exist_ok=False)
    files={}
    def retain(p, expected=None):
        p=Path(p).resolve()
        actual=sha(p)
        if expected is not None: require(actual==expected,'authentication mismatch '+str(p))
        files[str(p)]=actual
    # Current sources and all README integration entrypoints are independently pinned.
    for lane,freeze,_,_ in LANES.values():
        for p in (RESEARCH/lane).rglob('*'):
            if p.is_file(): retain(p)
    for p in [HERE/'review.py',HERE/'INTERFACE.md',RESEARCH/'mutable_buffer_goal/TARGET.md',
              RESEARCH/'mutable_buffer_goal/goal_scope.py',RESEARCH/'mutable_buffer_goal/test_goal_scope.py']:
        retain(p)
    for p in (ROOT/'evidence/native-components-model-review').iterdir():
        if p.is_file(): retain(p)
    foundation=read(RESEARCH/'mutable_buffer_goal/foundation-baseline.json')
    for k in ['readiness','receipt']:
        retain(foundation[k]['path'],foundation[k]['sha256'])
    ready=read(foundation['readiness']['path'])
    base=read(foundation['receipt']['path'])
    require(ready['receipt']==foundation['receipt']['path'],'wrong durable receipt')
    require('durable-build-2/receipt.json' in ready['receipt'],'not durable-build-2')
    require(ready['baseline_kernel_checked'] is True and base['success'] is True,'unchecked baseline')
    require(ready['source_commit']=='115a618dd35ff8337112cf2081a7f5391d3973d5','wrong foundation')
    require({'prover-version','build-baseline','kernel-check-baseline'} <=
            {c['label'] for c in base['commands'] if c['returncode']==0},'missing successful build stage')
    gmp=foundation['toolchain']['GMP_packages']
    require(gmp['sha256']=='332763a92d52a35f1623c7383970e6cd290b5010c426ca5bb15b21efc7f0bb37','wrong GMP pin')
    retain(gmp['path'],gmp['sha256'])
    authenticated={}
    for short,(lane,freeze,namespace,modules) in LANES.items():
        path=RESEARCH/lane
        index=read(path/'integration-results.json')
        historical=read(path/'results.json')
        require(index['delivered_receipt']['sha256']==historical['receipt']['sha256'],'historical receipt mismatch')
        receipts={}
        for k in ['delivered_receipt','integration_receipt','manager_receipt']:
            entry=index[k];retain(entry['path'],entry['sha256'])
            r=read(entry['path']);require(r['success'] is True,'unsuccessful '+k)
            receipts[k]=entry
            # Only explicitly delivered /tmp products are read from original absolute paths.
            for p,facts in r.get('proof_artifacts',{}).items():
                require(p.startswith('/tmp/'),'non-private product')
                retain(p,facts['sha256'])
        delivered=read(index['delivered_receipt']['path'])
        integrated=read(index['integration_receipt']['path'])
        manager=read(index['manager_receipt']['path'])
        require(manager['receipt_sha256']==index['integration_receipt']['sha256'],'manager chain mismatch')
        current=read(path/freeze)
        historical_freeze=read(path/'final-freeze.json')
        # Original proof sources/objects and current candidates must agree at all three points.
        comparisons={}
        for module in modules:
            src=path/'candidates'/(module+'.v')
            key=str(src.relative_to(ROOT))
            require(current['files'][key]==historical_freeze['files'][key]==sha(src),'mathematical source changed')
            for suffix in ['.v','.vo']:
                name=module+suffix
                a={Path(p).name:f['sha256'] for p,f in delivered['proof_artifacts'].items()}
                b={Path(p).name:f['sha256'] for p,f in integrated['proof_artifacts'].items()}
                require(a[name]==b[name],'integration math object differs '+name)
                if suffix=='.v': require(a[name]==sha(src),'delivered source differs')
                comparisons[name]=a[name]
        # Pin current dependencies/tools/support using authenticated integration receipt.
        # Historical /home/... checkout labels are remapped; never traversed.
        remapped={}
        for p,facts in integrated['inputs'].items():
            if not p.startswith('/tmp/') and re.search(r'/(source|evidence)/',p):
                m=re.search(r'/(source|evidence)/',p)
                require(m is not None,'unrecognized historical checkout label')
                current_path=ROOT/p[m.start()+1:]
                if current_path.name=='TARGET.md':
                    remapped[p]={'current':str(current_path),'historical_sha256':facts['sha256'],
                                 'current_sha256':sha(current_path),'progress_drift_allowed':True}
                    retain(current_path)
                else: retain(current_path,facts['sha256'])
            else:
                retain(p,facts['sha256'])
        authenticated[short]={'receipts':receipts,'unchanged_math':comparisons,'target_remap':remapped}
    write(RAW/'pretrial.json',{'base':'bc2a0b37','files':files,'authentication':authenticated,
                              'foundation_readiness':foundation['readiness'], 'foundation_receipt':foundation['receipt'],
                              'GMP_manifest':gmp})
    print(json.dumps({'raw':str(RAW),'pinned_files':len(files),'authentication':'passed'}))


def replay(short):
    lane,freeze,_,_=LANES[short]
    # Exact current README vector with only a fresh private output path substituted.
    run('replay-'+short,[sys.executable,'-B','-O',str(RESEARCH/lane/'check.py'),
        '--freeze',str(RESEARCH/lane/freeze),'--output',str(RAW/short)])
    receipt=read(RAW/short/'receipt.json')
    require(receipt['success'] is True,'replay receipt failed')
    original=read(RAW/'pretrial.json')['authentication'][short]['unchanged_math']
    fresh={Path(p).name:f['sha256'] for p,f in receipt['proof_artifacts'].items()}
    require(all(fresh[n]==h for n,h in original.items()),'fresh mathematical object changed')
    print(short+': replay success; all original mathematical sources/objects identical')


def discover(text):
    # Independent compiler-text parser. Parse declaration blocks first, then
    # constructor arms/record fields; no source-controller expected-name parser.
    result=[]; constants=[]
    heads=list(re.finditer(r'^     (Definition|Parameter|Inductive|Record)\s+(\w+)\b',text,re.M))
    for i,m in enumerate(heads):
        kind,name=m.groups();result.append(name)
        if kind in ['Definition','Parameter']: constants.append(name)
        body=text[m.end():heads[i+1].start() if i+1<len(heads) else len(text)]
        if kind=='Inductive':
            rhs=body.split(':=',1)[1]
            result.extend(re.findall(r'(?:^\s*|\|\s*)(\w+)\s*:',rhs,re.M))
        elif kind=='Record':
            rhs=body.split(':=',1)[1]
            constructor=re.match(r'\s*(\w+)',rhs)
            require(constructor is not None,'record constructor absent')
            result.append(constructor[1])
    require(len(result)==len(set(result)),'duplicate compiler discovery')
    return result,constants


def supplement(short):
    guard()
    lane,_,namespace,modules=LANES[short]
    receipt=read(RAW/short/'receipt.json')
    cmd=next(c['argv'] for c in receipt['commands'] if c['label'].startswith('compile-'))
    args=cmd[3:-1] # coqc -q followed by exact pinned -Q/-R pairs
    require(cmd[2]=='-q' and len(args)%3==0,'unexpected compile vector')
    work=RAW/(short+'-independent');work.mkdir(exist_ok=False)
    # Retain the fresh proof path loadpath; add only a new review module namespace.
    args=[*args,'-Q',str(work),'Review']
    products=[p for p in (RAW/short/'proofs').rglob('*') if p.is_file() and p.suffix in ['.v','.vo','.out']]
    product_pins={str(p):sha(p) for p in products}
    extra={'constructor':['Assumptions'], 'pointer':['Assumptions','Inspection'],
           'exit':['Assumptions','ProofTerms']}[short]
    inspect=work/'Discover.v'
    inspect.write_text('From '+namespace+' Require Import '+' '.join(modules+extra)+'.\n'+
        ''.join(f'Redirect "module-{m}" Print Module {namespace}.{m}.\n' for m in modules+extra))
    run(short+'-discover',[*cmd[:3],'-output-directory',str(work),*args,str(inspect)],work)
    inventories={};constants=[];names=[]
    for m in modules+extra:
        discovered,csts=discover((work/('module-'+m+'.out')).read_text())
        inventories[m]=discovered
        names += [m+'.'+n for n in discovered]
        constants += [m+'.'+n for n in csts]
    require(len(names)=={'constructor':96,'pointer':231,'exit':85}[short],'independent declaration count differs')
    if short=='pointer':
        require(inventories['Inspection']==[],'Inspection carries declarations')
        require(len(inventories['Assumptions'])==154 and len(constants)==203,'pointer helper/constant count differs')
    elif short=='constructor': require(inventories['Assumptions']==[],'constructor inspection carries declarations')
    else:
        require(len(inventories['Assumptions'])==len(inventories['ProofTerms'])==34,'exit helper count differs')
    inventory=work/'Individual.v'
    inventory.write_text('From '+namespace+' Require Import '+' '.join(modules+extra)+'.\n'+
        ''.join(f'Redirect "item-{i}.assumptions" Print Assumptions {namespace}.{name}.\n'
                f'Redirect "item-{i}.statement" Check {namespace}.{name}.\n'
                for i,name in enumerate(names)))
    run(short+'-individual',[*cmd[:3],'-output-directory',str(work),*args,str(inventory)],work)
    assumptions={}
    allowed={'Coq.Logic.FunctionalExtensionality.functional_extensionality_dep',
             'Coq.Logic.Eqdep.Eq_rect_eq.eq_rect_eq'} if short=='exit' else set()
    for i,name in enumerate(names):
        text=(work/f'item-{i}.assumptions.out').read_text().strip()
        if text=='Closed under the global context': axioms=[]
        else:
            require(text.startswith('Axioms:\n'),'unknown assumption format '+text)
            axioms=re.findall(r'^(Coq\.\S+)',text,re.M)
            require(axioms and set(axioms)<=allowed,'unexpected axioms '+text)
        assumptions[name]=axioms
    for m in extra:
        require(all(assumptions[m+'.'+n]==[] for n in inventories[m]),'generated helpers not closed')
    # Independently replay every original module, inspection module and the two
    # declaration-free review modules recursively, with kernel constants exposed.
    run(short+'-kernel-independent',[cmd[0],str(Path(cmd[1]).with_name('coqchk')),'-o',*args,
        *[namespace+'.'+m for m in modules+extra],'Review.Discover','Review.Individual'],work)
    kt=(RAW/(short+'-kernel-independent.stderr')).read_text()
    checked=re.findall(r'checking cst:'+namespace+r'\.(\w+\.\w+)',kt)
    require(len(checked)==len(constants) and set(checked)==set(constants),'independent kernel constant coverage differs')
    require(not re.findall(r'checking cst:Review\.',kt),'review inspection generated constants')
    require(kt.count('CONTEXT SUMMARY')==1,'missing unique kernel summary')
    summary=kt.split('CONTEXT SUMMARY')[1]
    for heading in ['Constants/Inductives relying on type-in-type', 'Constants/Inductives relying on unsafe (co)fixpoints',
                    'Inductives whose positivity is assumed']:
        require('* '+heading+': <none>' in summary,'unsafe kernel summary')
    require('Set is predicative' in summary and 'Rewrite rules are not allowed' in summary,'unsafe kernel theory')
    kernel_axioms=re.findall(r'^  (Coq\.\S+)',summary,re.M)
    require(set(kernel_axioms)<=allowed,'unexpected recursive kernel assumptions')
    if short!='exit': require('* Axioms: <none>' in summary,'nonempty recursive axioms')
    for p,h in product_pins.items(): require(sha(p)==h,'fresh replay products changed')
    guard()
    write(RAW/(short+'-independent.json'),{'discovered_modules':inventories,'assumptions':assumptions,
        'declaration_count':len(names),'constant_count':len(constants), 'kernel_constants':checked,
        'kernel_summary':summary,'kernel_axioms':kernel_axioms,
        'all_individual_statements_checked':True,'all_fresh_product_guards_passed':True,
        'products':{str(p):sha(p) for p in work.rglob('*') if p.is_file() and p.suffix in ['.v','.vo','.out']}})
    print(short+': independent complete discovery, individual assumptions and recursive kernel replay passed')


def tests():
    for mode in [[],['-O']]:
        run('goal-scope'+('-O' if mode else '-normal'),[sys.executable,'-B',*mode,'-m','unittest','-v','test_goal_scope.py'],RESEARCH/'mutable_buffer_goal')
    print('goal scope: seven tests passed in each mode')


if __name__=='__main__':
    action=sys.argv[1]
    if action=='prepare': prepare()
    elif action=='tests': tests()
    elif action=='replay': replay(sys.argv[2])
    elif action=='supplement': supplement(sys.argv[2])
    else: raise ValueError('unknown action')
