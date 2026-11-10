#!/usr/bin/env python3
"""Untrusted typed candidate producer. The generated ordinary proof checks each step.
No debug string is interpreted; this is NOT the Rust/reference semantic link.
"""
import argparse,hashlib,json,sys
from pathlib import Path
sys.dont_write_bytecode=True
HERE=Path(__file__).resolve().parent
sys.path.insert(0,str(HERE.parents[1]))
from infer_experiment import require

def sha(v): return hashlib.sha256(json.dumps(v,sort_keys=True,separators=(',',':')).encode()).hexdigest()
class Lower:
 def __init__(self,d,b,source_hash):
  require(d['schema']==2 and d['input_sha256']==source_hash and b['source_sha256']==source_hash,'source_binding')
  require(d['pointer_bits']==64 and not d['pending_instances'] and not d['extraction_blockers'],'incomplete_native_inventory')
  require(b['normal_analysis_completed'] is True and b['arg_count']==1,'normal_analysis')
  require(b['input_shapes']==[dict(kind='array_borrow',exclusive=True,element='u8',length=128,size=128,align=1)],'input_type')
  require(len(b['loans'])==1 and b['loans'][0]['kind']=='exclusive','single_source_loan')
  loan=b['loans'][0]
  require(loan['borrowed']['local']==1 and loan['borrowed']['projection']==[dict(op='deref')],'original_array_loan')
  require(any(r==loan['region'] and l==loan['loan'] for r,l,p in b['borrowck_facts']['loan_issued_at']),'loan_issued_fact')
  self.d=d;self.b=b;self.instances={x['id']:x for x in d['instances']};require(len(self.instances)==len(d['instances']),'duplicate_instance')
  self.steps=[];self.coverage={};self.frames=0;self.temp=10000;self.call_depth=0;self.excluded=[]
  self.loan=loan
  self.validate_types()
 def validate_types(self):
  for i in self.d['instances']:
   require(i['kind']=='mir','unavailable_native_body')
   for n,l in enumerate(i['locals']):
    require(l['local']==n,'local_identity');s=l['shape'];k=s['kind'];layout=l['layout'];expected=None
    if k=='tuple' and s['fields']==[]:expected=(0,1,[])
    elif k=='scalar_or_function' and s['ty']=='usize':expected=(8,8,[])
    elif k in ('ref','raw_ptr'):
     require(s['mutable'] is True,'pointer_mutability')
     if s['pointee']=='[u8]':expected=(16,8,[0,8])
     elif s['pointee'] in ('u8','[u8; 128]',"std::slice::IterMut<'_, u8>",'std::ptr::NonNull<u8>','std::ptr::NonNull<[u8]>'):expected=(8,8,[])
    elif k=='adt':
     require(not s['has_destructor'] and len(s['variants'])==1,'native_destructor_or_variants')
     fields=s['variants'][0]['fields']
     if s['definition']=='std::slice::IterMut':
      require(fields==[dict(name='ptr',ty='std::ptr::NonNull<u8>'),dict(name='end_or_len',ty='*mut u8'),dict(name='_marker',ty='std::marker::PhantomData<&mut u8>')],'iter_field_types')
      expected=(16,8,[0,8,16])
     elif s['definition']=='std::ptr::NonNull':
      require(s['repr_transparent'] is True,'nonnull_repr')
      if fields==[dict(name='pointer',ty='(*const u8) is !null')]:expected=(8,8,[0])
      elif fields==[dict(name='pointer',ty='(*const [u8]) is !null')]:expected=(16,8,[0])
    require(expected is not None,'unsupported_local_type')
    require((layout['size'],layout['align'],layout['field_offsets'])==expected,'local_type_layout')
    require(l['needs_drop'] is False,'local_drop_effect')
 def emit(self,op,*args):
  self.steps.append(dict(op=op,args=list(args)))
 def reg(self,f,p,deref=False):
  require(p['projection']==([dict(op='deref')] if deref else []),'nonlocal_place_or_store')
  require(type(p['local']) is int and 0<=p['local']<100,'local_budget')
  return f*100+p['local']
 def mark(self,id,bb,loc,ir,rule):
  key=(id,bb,str(loc));require(key not in self.coverage,'duplicate_operation')
  self.coverage[key]=dict(instance=id,bb=bb,location=loc,payload_sha256=sha(ir),rule=rule)
 def operand(self,f,o):
  if o['op'] in ('copy','move'):return self.reg(f,o['place'])
  require(o['op']=='constant' and o['evaluation_ok'] is True,'constant_ctfe')
  self.temp+=1;r=self.temp
  if o['bits'] is not None:
   require(o['ty'] in ('usize','bool'),'constant_type');self.emit('Num',r,int(o['bits']))
  else:
   require(o['zero_sized'] is True,'constant_non_scalar');self.emit('Unit',r)
  return r
 def local(self,i,p):
  require(not p['projection'],'type_projection'); ls=i['locals'];require(p['local']<len(ls),'local_missing')
  l=ls[p['local']];require(l['local']==p['local'] and l['ty']==p['ty'],'operand_type');return l
 def assign(self,i,f,ir):
  p=ir['destination'];dst=self.reg(f,p);target=self.local(i,p);v=ir['value'];op=v['op']
  if op=='use':
   o=v['operand'];r=self.operand(f,o)
   require(v['retag'] in ('Yes','No'),'retag_unknown')
   self.emit('Move' if o['op']=='move' else 'Copy',dst,r)
  elif op=='ref':
   require(v['mutable'] and not v['fake'],'borrow_kind')
   r=self.reg(f,v['place'],True);require(target['shape']['kind']=='ref','reference_type');require(f==0 and v['place']==self.loan['borrowed'] and p==self.loan['assigned'],'original_reborrow_operand');self.emit('Reborrow',dst,r)
  elif op=='raw_ptr':
   require(v['kind']=='Mut' and target['shape']['kind']=='raw_ptr','raw_kind')
   self.emit('Raw',dst,self.reg(f,v['place'],True))
  elif op=='unary':
   require(v['kind']=='PtrMetadata' and v['operand']['op']=='copy','metadata_kind');self.emit('Metadata',dst,self.operand(f,v['operand']))
  elif op=='binary':
   require(v['kind']=='Offset' and target['shape']==dict(kind='raw_ptr',pointee='u8',mutable=True),'offset_type')
   require(v['left']['op']=='copy' and v['right']['op']=='copy','offset_operand')
   self.emit('Offset',dst,self.operand(f,v['left']),self.operand(f,v['right']))
  elif op=='cast':
   o=v['operand'];r=self.operand(f,o);src=self.local(i,o['place']);a=src['shape'];b=target['shape'];k=v['kind']
   require(v['to']==target['ty'],'cast_destination')
   if k=='Unsize':
    require(f==0 and o['op']=='move' and a==dict(kind='ref',pointee='[u8; 128]',mutable=True) and b==dict(kind='ref',pointee='[u8]',mutable=True),'unsize_type')
    require(o['place']['local']==self.loan['assigned']['local'] and self.loan['reserve']==dict(bb=0,statement=3),'borrow_chain_binding')
    self.emit('Unsize',dst,r)
   elif k=='PtrToPtr':
    require(o['op']=='copy' and a==dict(kind='raw_ptr',pointee='[u8]',mutable=True) and b==dict(kind='raw_ptr',pointee='u8',mutable=True),'pointer_cast_type');self.emit('Thin',dst,r)
   elif k=='Transmute':
    require(o['op']=='copy' and src['layout']['size']==target['layout']['size'] and src['layout']['align']==target['layout']['align'],'transmute_layout')
    raw=a if a['kind']=='raw_ptr' else b
    wrapper=b if a['kind']=='raw_ptr' else a
    require(raw['kind']=='raw_ptr' and raw['mutable'] is True and raw['pointee'] in ('u8','[u8]'),'transmute_raw_type')
    require(wrapper['kind']=='adt' and wrapper['definition']=='std::ptr::NonNull' and wrapper['repr_transparent'] is True and not wrapper['has_destructor'],'transparent_pointer_type')
    fields=wrapper['variants'];require(len(fields)==1 and len(fields[0]['fields'])==1,'transparent_fields')
    require(fields[0]['fields'][0]['ty'] in ('(*const u8) is !null','(*const [u8]) is !null'),'transparent_pointee')
    self.emit('Wrap' if a['kind']=='raw_ptr' else 'Unwrap',dst,r)
   else: raise ValueError('unsupported_cast')
  elif op=='aggregate':
   require(v['kind']['op']=='adt' and v['kind']['definition']=='std::slice::IterMut' and v['kind']['variant']==0,'aggregate_type')
   require(target['shape']['definition']=='std::slice::IterMut' and not target['shape']['has_destructor'],'iter_type')
   fields=target['shape']['variants'][0]['fields'];require([x['name'] for x in fields]==['ptr','end_or_len','_marker'],'iter_fields')
   require(target['layout']['size']==16 and target['layout']['field_offsets']==[0,8,16],'iter_layout')
   require(len(v['operands'])==3 and v['operands'][2]['evaluation_ok'] and v['operands'][2]['zero_sized'],'iter_marker')
   require(all(o['op']=='copy' for o in v['operands'][:2]),'iter_operands')
   self.emit('Iter',dst,self.operand(f,v['operands'][0]),self.operand(f,v['operands'][1]))
  else: raise ValueError('unsupported_rvalue:'+op)
 def walk(self,id,f):
  require(self.call_depth<8,'call_depth_budget');self.call_depth+=1
  i=self.instances[id];require(i['kind']=='mir','missing_call_body');bb=0;seen=set();blocks={b['bb']:b for b in i['blocks']}
  while True:
   require(bb not in seen,'loop_requires_induction');seen.add(bb);b=blocks[bb];require(not b['cleanup'],'cleanup_requires_semantics')
   for n,s in enumerate(b['statements']):
    ir=s['ir'];op=ir['op'];self.mark(id,bb,n,ir,op)
    if op in ('storage_live','storage_dead'):self.emit('Clear',f*100+ir['local'])
    elif op=='assign':self.assign(i,f,ir)
    else:raise ValueError('unsupported_statement:'+op)
   t=b['term_ir'];op=t['op'];self.mark(id,bb,'term',t,op)
   if op=='return':self.emit('Return',f);break
   if op=='goto':bb=t['target']
   elif op=='switch':
    o=t['operand'];require(o['op']=='constant' and o['evaluation_ok'] and o['bits'] is not None,'nonconstant_branch')
    r=self.operand(f,o);n=int(o['bits']);require(len(t['cases'])==1,'switch_case_fragment')
    bb=next((c['target'] for c in t['cases'] if int(c['value'])==n),t['otherwise'])
    self.emit('Select',r,int(t['cases'][0]['value']),bb,t['cases'][0]['target'],t['otherwise'])
   elif op=='call':
    require(t['unwind']['action'] in ('continue','unreachable'),'unwind_cleanup')
    require(b['call'] and b['call']['resolved'] in self.instances and t['target'] is not None,'missing_call')
    callee=b['call']['resolved'];c=self.instances[callee];require(c['arg_count']==len(t['arguments'])==1,'call_arity')
    require(t['function']['op']=='constant' and t['function']['evaluation_ok'] and t['function']['zero_sized'],'indirect_call')
    arg=t['arguments'][0];require(arg['op']=='move','native_loan_operand')
    r=self.operand(f,arg);self.frames+=1;new=self.frames;self.emit('Move',new*100+1,r);self.emit('Enter',new,new*100+1)
    self.walk(callee,new);self.emit('Leave',new,new*100);self.emit('Move',self.reg(f,t['destination']),new*100);bb=t['target']
   else:raise ValueError('unsupported_terminator:'+op)
  # A constant branch's excluded block is recorded, not interpreted as safe.
  for b in i['blocks']:
   if b['bb'] not in seen:
    self.excluded.append(dict(instance=id,bb=b['bb'],reason='CTFE_unreachable',payload_sha256=sha(b)))
  self.call_depth-=1
 def build(self):
  self.walk(self.d['root'],0)
  # The same specialized glue is checked for every queried native local type.
  glues=set(l['drop_query']['instance'] for i in self.d['instances'] for l in i.get('locals',[]) if l.get('drop_query'))
  reached=set(k[0] for k in self.coverage)
  for id in sorted(glues-reached):
   require(id in self.instances,'missing_drop_body');i=self.instances[id];require(i['kind']=='mir' and len(i['blocks'])==1,'drop_effect_requires_semantics')
   b=i['blocks'][0];require(not b['cleanup'] and b['statements']==[] and b['term_ir']==dict(op='return'),'drop_effect_requires_semantics')
   self.mark(id,0,'term',b['term_ir'],'empty_drop_return');self.emit('DropReturn',0)
  covered=set(k[0] for k in self.coverage);require(covered==set(self.instances),'unexamined_helper')
  total=sum(len(i['blocks'][j]['statements'])+1 for i in self.instances.values() for j in range(len(i['blocks'])))
  excluded=sum(len(self.instances[x['instance']]['blocks'][x['bb']]['statements'])+1 for x in self.excluded)
  require(len(self.coverage)+excluded==total==self.d['operations'],'operation_exit_coverage')
  return dict(schema=1,source_sha256=self.d['input_sha256'],native_ir_sha256=sha(self.d),borrowck_sha256=sha(self.b),coverage=list(self.coverage.values()),excluded=self.excluded,steps=self.steps,operation_count=total,publication=False,unproved=['Rust_reference_embedding','MIR_primitive_simulation_and_retags','call_and_empty_drop_loan_interpretation','final_original_array_observation'])

if __name__=='__main__':
 p=argparse.ArgumentParser();p.add_argument('--native',type=Path,required=True);p.add_argument('--borrowck',type=Path,required=True);p.add_argument('--source',type=Path,required=True);p.add_argument('--output',type=Path,required=True);p.add_argument('--kernel',default='../kernel-v2/kernel.rs');a=p.parse_args()
 a.output.mkdir(parents=True,exist_ok=False)
 plan=Lower(json.loads(a.native.read_text()),json.loads(a.borrowck.read_text()),hashlib.sha256(a.source.read_bytes()).hexdigest()).build()
 (a.output/'plan.json').write_text(json.dumps(plan,indent=2)+'\n');(a.output/'trace.rs').write_text(__import__('witness').make(plan,a.kernel))
 (a.output/'case.json').write_text(json.dumps(dict(file=str((a.output/'trace.rs').resolve().relative_to(HERE)),positive=True,proof_bodies=['embedding','one_step_frame','trace_frame','step_preserves_typing','run_cons','unissued_suffix','reborrow_unsize_metadata','array_offset_safe','pointer_window','subregion_example','checked_drop_in_window','start','finish','checked_drop_trace']+['edge_'+str(n) for n in range(len(plan['steps']))]+['path_'+str(n) for n in range(len(plan['steps'])+1)]))+'\n')
 print(json.dumps(dict(steps=len(plan['steps']),operations=plan['operation_count'],publication=False)))
