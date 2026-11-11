"""Freeze from own committed records; never imports historical executable code."""
import ast
import hashlib
import subprocess
from pathlib import Path
ROOT = Path(__file__).resolve().parents[4]
BASE = '913044c4'
PATH = 'source/tools/research/nonzero_ordering_goal/run.py'
DECL_SHA = '0107890f5ffee6b94a4179c0351a4ff121e0e61b9886067e94370a65391e9abd'
def need(ok, message):
    if not ok: raise RuntimeError(message)
def declaration():
    tree = ast.parse(subprocess.check_output(['git','show',BASE+':'+PATH],cwd=ROOT))
    values = [ast.literal_eval(n.value) for n in tree.body if isinstance(n,ast.Assign)
              and any(isinstance(t,ast.Name) and t.id=='DECL' for t in n.targets)]
    need(len(values)==1,'unique original declaration')
    d=values[0]
    need(hashlib.sha256(d.encode()).hexdigest()==DECL_SHA,'declaration SHA256')
    return d
PRE = 'use vstd::prelude::*;\nuse std::num::NonZero;\nuse core::cmp::Ordering;\n'
REL = '''
impl BorTag {
 pub open spec fn __verus_infer_derived_cmp(&self)->bool {true}
 pub open spec fn __verus_infer_derived_partial_cmp(&self)->bool {true}
}
impl vstd::std_specs::cmp::OrdSpecImpl for BorTag {
 open spec fn obeys_cmp_spec()->bool {true}
 closed spec fn cmp_spec(&self,rhs:&Self)->Ordering {
  if self.0@ < rhs.0@ {Ordering::Less}
  else if self.0@ > rhs.0@ {Ordering::Greater} else {Ordering::Equal}
 }
}
impl vstd::std_specs::cmp::PartialOrdSpecImpl for BorTag {
 open spec fn obeys_partial_cmp_spec()->bool {true}
 closed spec fn partial_cmp_spec(&self,rhs:&Self)->Option<Ordering> {
  Some(vstd::std_specs::cmp::OrdSpec::cmp_spec(&self.0@,&rhs.0@))
 }
}
'''
def source(mode):
    need(mode in ['original','request','authored'],'fixture mode')
    attr = '#[verus_verify(infer_derive(PartialEq,PartialOrd,Ord))]' if mode=='request' else '#[verus_verify]'
    clients = '''
pub closed spec fn value(a:&BorTag)->u64 {a.0@}
pub fn total(a:&BorTag,b:&BorTag)->(r:Ordering)
 ensures r == vstd::std_specs::cmp::OrdSpec::cmp_spec(&value(a),&value(b)),
{a.cmp(b)}
pub fn partial(a:&BorTag,b:&BorTag)->(r:Option<Ordering>)
 ensures r == Some(vstd::std_specs::cmp::OrdSpec::cmp_spec(&value(a),&value(b))),
{a.partial_cmp(b)}
pub fn same_reference(a:&BorTag)
{let c=a.cmp(a);let p=a.partial_cmp(a);assert(c==Ordering::Equal);assert(p==Some(Ordering::Equal));}
'''
    for name,op in [('less','<'),('less_equal','<='),('greater','>'),('greater_equal','>=')]:
        clients+=f'pub fn {name}(a:&BorTag,b:&BorTag)->(r:bool) ensures r==(value(a){op}value(b)),{{a{op}b}}\n'
    return PRE+attr+'\n'+declaration()+'\nverus! {\n'+(REL if mode=='authored' else '')+clients+'}\n'
