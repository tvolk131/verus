"""Untrusted normalized-state witnesses; every edge must pass ordinary Verus."""
import sys
sys.dont_write_bytecode=True

def optext(s):return 'Op::'+s['op']+'('+','.join(str(x) for x in s['args'])+')'
def val(v):
 k=v[0]
 if k in ('Dead','Unit'):return 'Val::'+k
 if k=='Num':return f'Val::Num({v[1]})'
 if k=='Ref':return f'Val::Ref{{p:Ptr{{origin,loan,offset:{v[1]}}},len:{v[2]},slice:{str(v[3]).lower()}}}'
 if k in ('Raw','NonNull'):return f'Val::{k}{{p:Ptr{{origin,loan,offset:{v[1]}}},meta:'+('None' if v[2] is None else f'Some({v[2]})')+'}'
 if k=='Iter':return f'Val::Iter{{start:Ptr{{origin,loan,offset:{v[1]}}},end:Ptr{{origin,loan,offset:{v[2]}}}}}'
 raise ValueError(k)
def make(plan,kernel):
 regs={1:('Ref',0,128,False)};custodian=0;stack=[];states=[]
 def state():
  m='Map::<int,Val>::empty()'+''.join(f'.insert({r},{val(v)})' for r,v in sorted(regs.items()))
  return f'State{{bytes,origin,loan,regs:{m},custodian:{custodian},stack:seq!['+','.join(str(x) for x in stack)+'],ok:true}'
 states.append(state())
 for s in plan['steps']:
  k=s['op'];a=s['args'];d=a[0]
  if k=='Clear':regs[d]=('Dead',)
  elif k in ('Unit','Num'):regs[d]=(k,*a[1:])
  elif k in ('Copy','Move','Reborrow'):
   regs[d]=regs[a[1]]
   if k=='Move':regs[a[1]]=('Dead',)
  elif k=='Unsize':
   regs[d]=(*regs[a[1]][:3],True);regs[a[1]]=('Dead',)
  elif k=='Metadata':regs[d]=('Num',regs[a[1]][2])
  elif k=='Raw':regs[d]=('Raw',regs[a[1]][1],regs[a[1]][2])
  elif k in ('Wrap','Unwrap'):regs[d]=('NonNull' if k=='Wrap' else 'Raw',*regs[a[1]][1:])
  elif k=='Thin':regs[d]=('Raw',regs[a[1]][1],None)
  elif k=='Offset':regs[d]=('Raw',regs[a[1]][1]+regs[a[2]][1],None)
  elif k=='Iter':regs[d]=('Iter',regs[a[1]][1],regs[a[2]][1])
  elif k=='Enter':stack.append(custodian);custodian=d
  elif k=='Leave':custodian=stack.pop()
  elif k in ('Test','Select','Return','DropReturn'):pass
  else:raise ValueError('witness unsupported '+k)
  states.append(state())
 lines=['use vstd::prelude::*;',f'#[path = "{kernel}"] mod kernel;','use kernel::*;','verus! {']
 for n,st in enumerate(states):lines.append(f'pub open spec fn state_{n}(bytes:Seq<u8>,origin:int,loan:int) -> State {{ {st} }}')
 for n,s in enumerate(plan['steps']):
  lines.extend([f'pub proof fn edge_{n}(bytes:Seq<u8>,origin:int,loan:int)', ' requires bytes.len()==128,',
   f' ensures step(state_{n}(bytes,origin,loan),{optext(s)}) == state_{n+1}(bytes,origin,loan),',f' wf(state_{n+1}(bytes,origin,loan)),', '{',f' assert(step(state_{n}(bytes,origin,loan),{optext(s)}).regs =~= state_{n+1}(bytes,origin,loan).regs);',f' assert(step(state_{n}(bytes,origin,loan),{optext(s)}).stack =~= state_{n+1}(bytes,origin,loan).stack);','}'])
 lines.extend(['pub proof fn start(bytes:Seq<u8>,origin:int,loan:int) requires bytes.len()==128, ensures initial(bytes,origin,loan)==state_0(bytes,origin,loan), {}',f'pub proof fn finish(bytes:Seq<u8>,origin:int,loan:int) ensures state_{len(states)-1}(bytes,origin,loan).bytes==bytes, state_{len(states)-1}(bytes,origin,loan).custodian==0, state_{len(states)-1}(bytes,origin,loan).stack.len()==0, {{}}'])
 # Closed suffix definitions keep proof stitching linear and independently checked.
 N=len(plan['steps'])
 lines.append(f'pub closed spec fn tail_{N}() -> Seq<Op> {{ seq![] }}')
 lines.append(f'pub proof fn path_{N}(bytes:Seq<u8>,origin:int,loan:int) ensures run(state_{N}(bytes,origin,loan),tail_{N}())==state_{N}(bytes,origin,loan), {{ reveal(tail_{N}); }}')
 for n in reversed(range(N)):
  lines.append(f'pub closed spec fn tail_{n}() -> Seq<Op> {{ seq![{optext(plan["steps"][n])}]+tail_{n+1}() }}')
  lines.extend([f'pub proof fn path_{n}(bytes:Seq<u8>,origin:int,loan:int)', ' requires bytes.len()==128,',f' ensures run(state_{n}(bytes,origin,loan),tail_{n}())==state_{N}(bytes,origin,loan),','{',' hide(step);',' hide(wf);'])
  for m in sorted({n,n+1,N}):lines.append(f' hide(state_{m});')
  lines.extend([f' reveal(tail_{n});',f' edge_{n}(bytes,origin,loan);',f' path_{n+1}(bytes,origin,loan);',f' run_cons(state_{n}(bytes,origin,loan),{optext(plan["steps"][n])},tail_{n+1}());','}'])
 lines.extend(['pub proof fn checked_drop_trace(bytes:Seq<u8>,origin:int,loan:int)',' requires bytes.len()==128,',f' ensures run(initial(bytes,origin,loan),tail_0()) == state_{N}(bytes,origin,loan),',f' state_{N}(bytes,origin,loan).bytes==bytes,',f' state_{N}(bytes,origin,loan).custodian==0,',f' state_{N}(bytes,origin,loan).stack.len()==0,','{',' hide(step);',f' hide(state_0);',f' hide(state_{N});',' start(bytes,origin,loan);',' finish(bytes,origin,loan);',' path_0(bytes,origin,loan);','}',f'pub proof fn checked_drop_in_window(bytes:Seq<u8>,origin:int,loan:int,w:Window) requires bytes.len()==128,window_valid(w), ensures run(initial(bytes,origin,loan),tail_0()).bytes==bytes,offset_safe(w,128), {{ checked_drop_trace(bytes,origin,loan); array_offset_safe(w,128); }}','}'])
 return '\n'.join(lines)+'\n'
