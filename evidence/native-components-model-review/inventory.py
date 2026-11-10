#!/usr/bin/env python3
"""Compiler-discovered inventory, including anonymous inspection helpers."""
import argparse, json, os, re, subprocess, sys, time
from pathlib import Path
sys.dont_write_bytecode=True
from review import RAW, ROOT, HERE, SPECS, need, read, write, sha, guards
SCOPES={
 'caller':('ModelCallerOrdinaryWriter',['interface','caller','Assumptions','ProofTerms']),
 'constructor':('NativeConstructorErasure',['Interface','Graph','Composition','Boundary','Assumptions']),
 'distance':('NativePointerDistance',['distance','branches','Assumptions'])}
ALLOWED={'Coq.Logic.FunctionalExtensionality.functional_extensionality_dep','Coq.Logic.Eqdep.Eq_rect_eq.eq_rect_eq'}
ALIASES={'functional_extensionality_dep':'Coq.Logic.FunctionalExtensionality.functional_extensionality_dep','Eqdep.Eq_rect_eq.eq_rect_eq':'Coq.Logic.Eqdep.Eq_rect_eq.eq_rect_eq'}
def printed(text):
 names=re.findall(r'^     (?:Definition|Parameter|Record|Inductive)\s+(\w+)',text,re.M)
 for block in re.finditer(r'^     Inductive\s+\w+(.*?)(?=^     (?:Definition|Inductive|Record|Parameter)|^   End)',text,re.M|re.S):
  names+=re.findall(r'(?:^\s*|\|\s*)(\w+)\s*:',block[1].split(':=',1)[1])
 names+=re.findall(r'^     Record\s+\w+.*?:=\s*(\w+)',text,re.M)
 need(len(names)==len(set(names)), 'duplicate discovered declaration')
 return names

def main(tag):
 guards()
 frozen=read(HERE/'inventory-freeze.json')
 need(sha(HERE/'inventory.py')==frozen['script_sha256'],'inventory harness changed')
 out=RAW/(tag+'-inventory');out.mkdir(exist_ok=False)
 scope,freeze,*_=SPECS[tag]; ns,modules=SCOPES[tag]
 r=read(RAW/tag/'receipt.json');need(r['success'] is True,'replay incomplete')
 # Fully authenticate the unchanged replay before importing objects.
 pins={str(RAW/tag/'receipt.json'):sha(RAW/tag/'receipt.json')}
 for n,f in r['inputs'].items():need(sha(n)==f['sha256'],'fresh input changed '+n);pins[n]=f['sha256']
 for n,f in r['proof_artifacts'].items():need(sha(n)==f['sha256'],'fresh product changed '+n);pins[n]=f['sha256']
 argv=r['commands'][0]['argv'][:-1]
 need(Path(argv[1]).name=='coqc' and argv[2]=='-q','not normal coqc')
 argv+=['-Q',str(out),'IndependentReview']
 env=dict(os.environ)
 for k in ['COQPATH','COQLIB','COQCORELIB','OCAMLPATH','CAML_LD_LIBRARY_PATH','LD_PRELOAD','LD_LIBRARY_PATH']:env.pop(k,None)
 records=[]
 def keep(p):pins[str(p)]=sha(p)
 def check():
  guards()
  for n,h in pins.items():need(sha(n)==h,'inventory guard changed '+n)
 def run(label,cmd):
  check();t=time.monotonic()
  with (out/(label+'.stdout')).open('wb') as a,(out/(label+'.stderr')).open('wb') as b:
   p=subprocess.run(cmd,cwd=out,env=env,stdout=a,stderr=b,timeout=1800)
  records.append({'label':label,'argv':cmd,'exit_code':p.returncode,'seconds':time.monotonic()-t})
  keep(out/(label+'.stdout'));keep(out/(label+'.stderr'))
  need(p.returncode==0,'inventory command failed '+label);check()
 imports='From '+ns+' Require Import '+' '.join(modules)+'.\n'
 src=out/'Modules.v';src.write_text(imports+''.join(f'Redirect "{m}.module" Print Module {ns}.{m}.\n' for m in modules));keep(src)
 run('modules',argv+[str(src)]);keep(src.with_suffix('.vo'))
 inventory={m:printed((out/(m+'.module.out')).read_text()) for m in modules}
 for m in modules:keep(out/(m+'.module.out'))
 f=read(Path(read(HERE/'pretrial-freeze.json')['scopes'][tag]['replay_root'])/'source/tools/research'/scope/freeze)
 for m,expected in f['constants'].items():need(set(inventory[m])==set(expected),'compiler inventory mismatch '+m)
 if tag=='caller':
  for m in ['Assumptions','ProofTerms']:need(set(m+'.'+n for n in inventory[m])==set(r['inventory_helper_theorems'][m]),'caller helper set')
 # Record actual helper names instead of assuming Goal True is anonymous/discarded.
 inspect=out/'AllAssumptions.v'
 names=[m+'.'+n for m in modules for n in inventory[m]]
 inspect.write_text(imports+''.join(f'Redirect "{n}.assumptions" Print Assumptions {ns}.{n}.\nRedirect "{n}.statement" Check {ns}.{n}.\n' for n in names));keep(inspect)
 run('all-assumptions',argv+[str(inspect)]);keep(inspect.with_suffix('.vo'))
 assumptions={}
 for n in names:
  p=out/(n+'.assumptions.out');text=p.read_text().strip();keep(p);keep(out/(n+'.statement.out'))
  if text=='Closed under the global context': axioms=[]
  else:
   need(text.startswith('Axioms:\n'),'unknown assumptions output')
   axioms=[ALIASES.get(a,a) for a in re.findall(r'^([^\s:]+)\s*:',text.split('\n',1)[1],re.M)]
   need(axioms and set(axioms)<=ALLOWED,'new/unparsed axiom')
  need(tag=='caller' or not axioms,'nonclosed native fragment')
  if n.split('.',1)[0] in ['Assumptions','ProofTerms']:need(not axioms,'nonclosed inventory helper')
  assumptions[n]=axioms
 kernel=[argv[0],str(Path(argv[1]).with_name('coqchk')),'-o',*argv[3:],*[ns+'.'+m for m in modules],'IndependentReview.Modules','IndependentReview.AllAssumptions']
 run('kernel',kernel)
 text=(out/'kernel.stderr').read_text();need(text.count('CONTEXT SUMMARY')==1,'missing summary')
 summary=text.split('CONTEXT SUMMARY')[1]
 expected={'Theory: Set is predicative','Theory: Rewrite rules are not allowed','Constants/Inductives relying on type-in-type: <none>','Constants/Inductives relying on unsafe (co)fixpoints: <none>','Inductives whose positivity is assumed: <none>'}
 lines=re.findall(r'^\* (.+?)\s*$',summary,re.M)
 axheading='Axioms:' if tag=='caller' else 'Axioms: <none>'
 need(len(lines)==6 and set(lines)==expected|{axheading},'unsafe/unrecognized summary')
 kernel_axioms=re.search(r'^\* Axioms:(.*?)(?=^\* |\Z)',summary,re.M|re.S)[1].strip().splitlines()
 kernel_axioms=[a.strip() for a in kernel_axioms if a.strip() and a.strip()!='<none>']
 need(set(kernel_axioms)==set(r['kernel_axioms']),'kernel axiom set differs')
 # Print Module is the complete inventory for inductives/constructors as well as csts.
 # Match every kernel-checked cst against that independently discovered namespace.
 csts=re.findall(r'checking cst:'+re.escape(ns)+r'\.(\w+\.\w+)',text)
 need(len(csts)==len(set(csts)) and set(csts)<=set(names),'unexpected/duplicate checked cst')
 need(all(n in csts for n in names if n.split('.',1)[0] in ['Assumptions','ProofTerms']),'unchecked helper')
 check()
 result={'success':True,'all_final_guards_before_success':True,'inventory':inventory,'assumptions':assumptions,'kernel_axioms':kernel_axioms,'kernel_csts':csts,'commands':records,'products':pins,'source_count':sum(len(inventory[m]) for m in f['constants']),'inspection_helpers':sum(len(inventory[m]) for m in modules if m not in f['constants']),'program_obligations':[]}
 write(out/'receipt.json',result)
 print(tag,'independent inventory passed',result['source_count'],result['inspection_helpers'],flush=True)
if __name__=='__main__':
 a=argparse.ArgumentParser();a.add_argument('scope',choices=SCOPES);main(a.parse_args().scope)
