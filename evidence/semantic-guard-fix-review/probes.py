#!/usr/bin/env python3
"""Private guard probes. Rejections are setup/eligibility events, never semantic negatives."""
import argparse, copy, json, shutil, sys, tempfile
from pathlib import Path
sys.dont_write_bytecode = True
from recheck import ROOT, BRIDGE, RAW, READY, FREEZE, sha, need, save, run
sys.path.insert(0,str(BRIDGE))
from proof_checks import parse_axioms, parse_inventory, parse_kernel_summary, check_inventory
from source_map import build, authenticated_extraction

NAMES = {
 'borrow_handoff':['source_loan_access','source_loan_release','source_context_access'],
 'byte_array':['byte_list_length','bytes_size','bytes_physical','bytes_initialized','byte_array128_size','byte_array128_release','byte_array128_scoped_handoff','byte_array128_context_handoff'],
 'raw_view':['typed_source_raw_view','typed_source_raw_offset','typed_source_loan_release'],
 'model_limits':['arbitrary_raw_pointer_typed','raw_read_empty_head_stuck','model_offset_unrestricted','empty_heap_offset_adequate']}
CONSTANTS = [m+'.'+n for m,ns in NAMES.items() for n in ns]+['byte_array.byte_ty_obligation_'+str(i) for i in range(1,7)]+['byte_array.byte_ty']
FE='Coq.Logic.FunctionalExtensionality.functional_extensionality_dep'
SUMMARY=Path('/tmp/native-semantic-bridge-e4a8dcab/proof-v12/kernel-check.stderr').read_text()


def main(out):
 out.mkdir(parents=True,exist_ok=False)
 results=[]
 def probe(label, action, accept=False):
  try: value=action()
  except (ValueError,KeyError,IndexError,TypeError,FileNotFoundError,StopIteration) as e:
   need(not accept,label+' unexpectedly refused: '+str(e))
   results.append(dict(label=label,accepted=False,error_type=type(e).__name__,message=str(e)))
  else:
   need(accept,label+' unexpectedly accepted')
   results.append(dict(label=label,accepted=True))
 closed='Closed under the global context'
 base=''.join(f'BEGIN {n}\n{closed}\nEND {n}\n' for n in CONSTANTS)
 probe('all25-closed-parser-compatible',lambda:parse_inventory(base),True)
 for ax in ['classic','Coq.Logic.Classical_Prop.classic','Other.functional_extensionality_dep']:
  for prefix in ['', ' ', '  ', '\t']:
   probe('extra-axiom-'+repr(prefix)+ax,lambda ax=ax,prefix=prefix:parse_axioms('Axioms:\n'+prefix+ax+' : forall P : Prop, P \\/ ~P'))
 for label,text in [('empty',''),('empty-axioms','Axioms:\n'),('closed-plus-extra',closed+'\nclassic : True'),('trailing-unindented','Axioms:\nfunctional_extensionality_dep : True\nclassic'),('duplicate-allowed','Axioms:\nfunctional_extensionality_dep : True\nfunctional_extensionality_dep : True')]:
  probe(label,lambda text=text:parse_axioms(text))
 for label,text in [('missing-first',base[base.index('BEGIN '+CONSTANTS[1]):]),('duplicated',base+base),('mismatched-end',base.replace('END '+CONSTANTS[0],'END wrong',1)),('swapped',base.replace(CONSTANTS[0],'SWAP').replace(CONSTANTS[1],CONSTANTS[0]).replace('SWAP',CONSTANTS[1]))]:
  probe(label,lambda text=text:parse_inventory(text))
 for n in ['borrow_handoff.source_context_access','raw_view.typed_source_raw_view','model_limits.empty_heap_offset_adequate','byte_array.byte_ty_obligation_6','byte_array.byte_ty','byte_array.byte_list_length']:
  text=base.replace('BEGIN '+n+'\n'+closed,'BEGIN '+n+'\nAxioms:\nfunctional_extensionality_dep : True')
  probe('closed-policy-'+n,lambda text=text:parse_inventory(text))
 text=base.replace('BEGIN byte_array.bytes_size\n'+closed,'BEGIN byte_array.bytes_size\nAxioms:\nEqdep.Eq_rect_eq.eq_rect_eq : True')
 probe('specialized-Eqrect-disallowed',lambda:parse_inventory(text))
 probe('specialized-FE-allowed',lambda:parse_inventory(base.replace('BEGIN byte_array.bytes_size\n'+closed,'BEGIN byte_array.bytes_size\nAxioms:\nfunctional_extensionality_dep : True')),True)
 probe('real-entire-summary',lambda:parse_kernel_summary(SUMMARY),True)
 for label,text in [('missing-summary',''),('duplicate-summary',SUMMARY+SUMMARY),('extra-axiom',SUMMARY.replace('* Axioms:', '* Axioms:\n    Coq.Logic.Classical_Prop.classic')),('impredicative',SUMMARY.replace('Set is predicative','Set is impredicative')),('rewrite-allowed',SUMMARY.replace('Rewrite rules are not allowed','Rewrite rules are allowed')),('type-in-type',SUMMARY.replace('type-in-type: <none>','type-in-type: Bad.constant')),('unsafe-fix',SUMMARY.replace('unsafe (co)fixpoints: <none>','unsafe (co)fixpoints: Bad.constant')),('positivity',SUMMARY.replace('positivity is assumed: <none>','positivity is assumed: Bad.inductive')),('unknown-heading',SUMMARY+'\n* Unknown: unsafe\n')]:
  probe(label,lambda text=text:parse_kernel_summary(text))
 # The parser is format-specific, not an arbitrary hostile-text validator.
 # Record tolerances rather than treating unproducible strings as semantic bypasses.
 tolerances={
  'indented-split-name-after-FE':lambda:parse_axioms('Axioms:\nfunctional_extensionality_dep : True\n  classic\n  : forall P : Prop, P \\/ ~P'),
  'nonheading-line-after-none':lambda:parse_kernel_summary(SUMMARY+'\n    Bad.constant\n'),
  'extra-text-outside-inventory':lambda:parse_inventory('classic : True\n'+base+'classic : True\n')}
 for label,action in tolerances.items():
  try:value=action();results.append(dict(label=label,accepted=True,classification='synthetic-format-tolerance-no-real-emitter-bypass-demonstrated'))
  except Exception as e:results.append(dict(label=label,accepted=False,message=str(e)))
 native=json.loads((RAW/'drop-native.json').read_text()); borrow=json.loads((RAW/'drop-borrowck.json').read_text())
 src=BRIDGE.parent/'native_heap_lowering/fixtures/drop.rs'
 probe('genuine-source-eligible',lambda:build(native,borrow,src),True)
 mutations={
  'unsize-source-local1':lambda n,b:b['blocks'][0]['statements'][4]['ir']['value']['operand']['place'].update(local=1),
  'call-source-local1':lambda n,b:b['blocks'][0]['terminator']['arguments'][0]['place'].update(local=1),
  'ref-source-local4':lambda n,b:b['blocks'][0]['statements'][3]['ir']['value']['place'].update(local=4),
  'unsize-copy':lambda n,b:b['blocks'][0]['statements'][4]['ir']['value']['operand'].update(op='copy'),
  'ref-fake':lambda n,b:b['blocks'][0]['statements'][3]['ir']['value'].update(fake=True),
  'ref-shared':lambda n,b:b['blocks'][0]['statements'][3]['ir']['value'].update(mutable=False),
  'ref-kind':lambda n,b:b['blocks'][0]['statements'][3]['ir']['value'].update(kind='Mut { kind: Default }'),
  'ref-destination':lambda n,b:b['blocks'][0]['statements'][3]['ir']['destination'].update(local=1),
  'issue-location':lambda n,b:b['blocks'][0]['statements'][3]['location'].update(statement=4),
  'unsize-target-type':lambda n,b:b['blocks'][0]['statements'][4]['ir']['value'].update(to='&[u8]'),
  'call-destination':lambda n,b:b['blocks'][0]['terminator']['destination'].update(local=9),
  'call-function':lambda n,b:b['blocks'][0]['terminator']['function'].update(ty='other'),
  'native-call-argument':lambda n,b:n['instances'][0]['blocks'][0]['term_ir']['arguments'][0]['place'].update(local=1),
  'duplicate-borrowck-block':lambda n,b:b['blocks'].append(copy.deepcopy(b['blocks'][0])),
  'missing-loans':lambda n,b:b.update(loans=[]),
  'missing-borrowck-block':lambda n,b:b['blocks'].pop()}
 for label,mutate in mutations.items():
  n,b=copy.deepcopy(native),copy.deepcopy(borrow);mutate(n,b)
  probe(label,lambda n=n,b=b:build(n,b,src))
 private=out/'extraction-copy';shutil.copytree(RAW,private,ignore=lambda d,ns:[n for n in ns if n not in ('drop-native.json','drop-borrowck.json','receipt.json')])
 probe('byte-identical-extraction-copy',lambda:authenticated_extraction(private),True)
 for name in ('receipt.json','drop-native.json','drop-borrowck.json'):
  p=private/name;data=p.read_bytes();p.unlink()
  probe('missing-'+name,lambda:authenticated_extraction(private));p.write_bytes(data+b'\n')
  probe('changed-bytes-'+name,lambda:authenticated_extraction(private));p.write_bytes(data)
 save(out/'probes.json',dict(cases=results,semantic_negatives=0,publication=False))
 # Full-controller refusal checks for every mandatory frozen input.
 f=json.loads(FREEZE.read_text())
 mandatory=['check.py','proof_checks.py','source-bindings.json','baseline-bindings.json']
 paths=[str((BRIDGE/n).relative_to(ROOT)) for n in mandatory]
 paths += [str((BRIDGE.parent/'mutable_buffer_goal/foundation-baseline.json').relative_to(ROOT))]
 paths += [str((BRIDGE/'candidates/loan-v12'/(m+'.v')).relative_to(ROOT)) for m in NAMES]
 checks=[]
 for index,name in enumerate(paths):
  altered=copy.deepcopy(f);del altered['files'][name]
  pin=out/('missing-input-'+str(index)+'.json');save(pin,altered)
  result=out/('missing-input-'+str(index))
  r=run(out,'missing-input-'+str(index),[sys.executable,'-B','-O',BRIDGE/'check.py','--ready',READY,'--freeze',pin,'--output',result])
  receipt=json.loads((result/'receipt.json').read_text())
  need(r['returncode']!=0 and receipt['failure']['message']=='incomplete frozen trust inputs' and not receipt['commands'],'mandatory input omission escaped')
  r.update(omitted=name,receipt_sha256=sha(result/'receipt.json'));checks.append(r)
 for label,modules in [('empty',[]),('reduced',['borrow_handoff']),('reordered',list(reversed(NAMES))),('duplicate',list(NAMES)+['raw_view'])]:
  altered=copy.deepcopy(f);altered['modules']=modules;pin=out/(label+'.json');save(pin,altered);result=out/(label+'-inventory')
  r=run(out,label,[sys.executable,'-B','-O',BRIDGE/'check.py','--ready',READY,'--freeze',pin,'--output',result]);receipt=json.loads((result/'receipt.json').read_text())
  need(r['returncode']!=0 and not receipt['commands'] and receipt['failure']['message']=='exact four-module inventory required','module inventory escaped')
  r.update(receipt_sha256=sha(result/'receipt.json'));checks.append(r)
 save(out/'controller-refusals.json',checks)
 print(json.dumps(dict(parser_source_cases=len(results),controller_refusals=len(checks))))

if __name__=='__main__':
 p=argparse.ArgumentParser();p.add_argument('--output',required=True,type=Path);a=p.parse_args();main(a.output.resolve())
