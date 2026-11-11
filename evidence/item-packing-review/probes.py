"""Fresh finite-width controls. No false frontend/setup rejection counts as proof evidence."""
import review as t
import json
R=t.root()
PRE='use vstd::prelude::*; verus! {\nfn flag(b:bool)->(r:u64) ensures r==if b {1u64} else {0u64} {u64::from(b)}\n'
def check(name,s,expected):
 p=R/'probes-source'/f'{name}.rs';p.parent.mkdir(exist_ok=True);d=R/'probes'/name
 if (d/'receipt.json').exists():
  t.need(p.read_text()==s,'immutable resumed fixture '+name);j=json.loads((d/'receipt.json').read_text());f=json.loads((d/'freeze.json').read_text());shipping=json.loads((R/'full/cold/freeze.json').read_text())
  t.need(j['guards'] and f['source']==shipping['source'] and f['tools']==shipping['tools'],'resumed shipping identity')
  for n,key in [('freeze.json','freeze_sha256'),('stdout','stdout_sha256'),('stderr','stderr_sha256')]:t.need(t.sha(d/n)==j[key],'resumed receipt '+name)
 else:
  t.need(not p.exists(),'new immutable fixture '+name);p.write_text(s);j=t.trial('probes/'+name,p)
 v=j.get('verification',{});err=(R/'probes'/name/'stderr').read_text()
 t.need(v.get('is-verifying-entire-crate') and ((j['exit']==0 and v.get('errors',0)==0) if expected=='pass' else (j['exit']!=0 and v.get('errors',0)>0)),'verification outcome '+name)
 if expected in ['false','conservative']:t.need('postcondition not satisfied' in err or 'precondition not satisfied' in err or 'assertion failed' in err or 'possible bit shift' in err or 'overflow' in err,'semantic diagnostic '+name)
 return dict(receipt=j,expected=expected)
def main():
 out={}
 for w in [8,16,32,64]:
  ty=f'u{w}';H=w-1;M=(1<<w)-1
  core=f'''const NEST:{ty}=((3{ty} << 2u64) | 1{ty}) ^ 1{ty};
fn zero(x:{ty})->(r:{ty}) ensures r==x {{ (x << 0u64) >> 0u64 }}
fn edge(x:{ty})->(r:{ty}) requires x<2, ensures r==x*{1<<H}, {{x << {H}u64}}
fn raw(low:{ty},mid:{ty},upper:{ty})->(r:{ty})
 requires low<2, mid<{1<<(w-2)},
 ensures r & 1{ty}==low, (r & {M-1-(1<<H)}{ty}) >> 1u64 == mid,
 {{low | (mid << 1u64) | (upper << {H}u64)}}
fn nested(x:{ty})->(r:{ty}) ensures r<=3 {{(x & NEST) >> 2u64}}
fn reversed(x:{ty})->(r:{ty}) ensures r<=3 {{(12{ty} & x) >> 2u64}}
'''
  out[f'valid-conservative-{w}']=check(f'valid-conservative-{w}',PRE+core+'}', 'conservative')
  supported='\n'.join(line for line in core.splitlines() if not line.startswith('fn edge(') and not line.startswith('fn reversed('))
  out[f'supported-{w}']=check(f'supported-{w}',PRE+supported+'}', 'pass')
  # These all have concrete semantic counterexamples; separate source fixtures identify each failure.
  controls={
   'relation':f'fn f(x:{ty})->(r:{ty}) ensures r!=x {{x << 0u64}}',
   'range':f'fn f(low:{ty},high:{ty})->(r:{ty}) requires low<8, high<2, ensures r & 3{ty}==low {{low | (high << 2u64)}}',
   'field':f'fn f(x:{ty})->(r:{ty}) ensures r<3 {{(x & 12{ty}) >> 2u64}}',
   'flag':f'fn f(low:{ty},b:bool)->(r:{ty}) requires low<4, ensures (r & 4{ty}>0)==!b {{low | ((u64::from(b) as {ty}) << 2u64)}}',
   'shift-safety':f'fn f(x:{ty})->{ty} {{ x << {w}u64 }}',
   'signed':f'fn f(x:i{w})->(r:i{w}) requires x<4, ensures r==x {{x & 3i{w}}}',
   'noncontiguous':f'fn f(x:{ty})->(r:{ty}) ensures r<=3 {{(x & 20{ty}) >> 2u64}}',
   'reversed':f'fn f(x:{ty})->(r:{ty}) ensures r<3 {{(12{ty} & x) >> 2u64}}',
   'nested-constant':f'const N:{ty}=(3{ty} << 2u64) | 16{ty}; fn f(x:{ty})->(r:{ty}) ensures r<=3 {{(x & N) >> 2u64}}',
   'xor-overlap':f'fn f(low:{ty},high:{ty})->(r:{ty}) requires low<8, high<2, ensures r & 7{ty}==low {{low ^ (high << 2u64)}}',
  }
  for n,s in controls.items():out[f'{n}-{w}']=check(f'{n}-{w}',PRE+s+'}', 'false')
 full=(R/'full.rs').read_text()
 controls={
 'full-false-relation':full.replace('r.word() & (u64::MAX >> 3)==tag.value()','r.word() & (u64::MAX >> 3)!=tag.value()'),
 'full-range':full.replace('requires tag.value() <= (u64::MAX >> 3),',''),
 'full-permission-shift':full.replace('perm.to_bits() << PERM_SHIFT','perm.to_bits() << (PERM_SHIFT - 1)'),
 'full-protector':full.replace('u64::from(protected)','u64::from(!protected)'),
 }
 for n,s in controls.items():t.need(s!=full,'effective '+n);out[n]=check(n,s,'false')
 t.write(t.HERE/'PROBES.json',out)
if __name__=='__main__':main()
