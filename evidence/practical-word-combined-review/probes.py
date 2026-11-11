"""Bounded independent source probes; existing execution and scope gates only."""
import os,json,sys
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2];O=Path('/tmp/practical-word-combined-review-5ecea5fe');D=O/'independent-probes';D.mkdir(exist_ok=False)
sys.path[:0]=[str(ROOT/'source/tools'),str(ROOT/'source/tools/research/practical_redb_fedimint/common_set')]
from infer_experiment import LoggedCommands,replay_receipt,validate_run,require_exact,require
from infer_eval_support import ArtifactGuard,digest
from replay_s1 import require_native_scope
B=O/'bundle';C=ROOT/'source/tools/research/practical_redb_fedimint';env=dict(os.environ,**json.loads((O/'env.json').read_text()),PYTHONDONTWRITEBYTECODE='1',VERUS_INFER_CACHE=str(D/'cache'))
manual=(C/'redb_tree/manual.rs').read_text();s2=(C/'word_frame/s2-reduced.rs').read_text()
ret='        group == u64::MAX\n'
assert manual.count(ret)==s2.count(ret)==1
helper='''fn return_full(&mut self, group: u64) -> (r: bool)
    ensures *final(self) == *old(self), r == (group == u64::MAX),
{ group == u64::MAX }
'''
call=manual.replace(ret,'        self.return_full(group)\n').replace('impl U64GroupedBitmap {','impl U64GroupedBitmap {\n'+helper)
# Build ordinary valid compatibility controls before stripping the one authored
# writer scaffold for the independent expected-skip probe.
missing=call.replace('        proof { word_frame(before, *self, bit, true); }\n','')
# Only remove this snapshot in the word set body (other snapshots remain exact).
pos=missing.index('fn set(&mut self, bit: u32)');prefix=missing[:pos];tail=missing[pos:];assert tail.count('        let ghost before = *self;')==2
tail=tail.replace('        let ghost before = *self;\n','',1);missing=prefix+tail
nested=s2.replace(ret,'        { self.data[index] = group; group == u64::MAX }\n')
field=manual.replace('    data: Vec<u64>,','    data: Vec<u64>,\n    audit: u32,').replace(ret,'        self.audit = self.audit;\n'+ret)
# Probe a wrong final frame postcondition while preserving the real implementation.
receipt={'commands':[],'outcomes':[]}
with replay_receipt(D,receipt):
 run=LoggedCommands(D,D,receipt['commands'],timeout=180,guard=ArtifactGuard([B/n for n in json.loads((B/'receipt.json').read_text())['tools']]))
 def check(label,text,goals,failed,expected_frames):
  src=D/(label+'.rs');src.write_text(text);g=ArtifactGuard([src])
  cmd=[str(B/'verus'),str(src),'--crate-type=lib','--crate-name',label,'--edition=2024','--no-cheating','--output-json','--time','--multiple-errors','20']+(['--infer-goals'] if goals else [])
  code=run(label,cmd,env);g.check();stdout=(D/(label+'.stdout')).read_text();stderr=(D/(label+'.stderr')).read_text();data=validate_run(dict(returncode=code,stdout=stdout,stderr=stderr),not failed)
  inf=data['verus-infer'];scope=inf['verification_scope'];require_exact(inf.get('functions',[]));require(not inf.get('functions',[]),'helpers empty')
  ex=['BtreeBitmap::'+n for n in ['clear','get_height','get_level','get_level_mut','set','update_to_root']]+['U64GroupedBitmap::'+n for n in ['clear','data_index_of','get','len','select_mask','set']]
  if 'fn return_full' in text:ex+=['U64GroupedBitmap::return_full']
  proof=['layer_edit','leaf_edit','clear_not_full','finish_mutation','propagation','ancestor_next']+(['word_bits','word_frame'] if 'proof fn word_bits' in text else [])
  require_native_scope(data,src,ex,proof,failed,{'finish_mutation':20} if goals else {})
  native={x['function']:x['rust_function'] for x in scope['native_functions']}
  require(sorted(native[x['function']] for x in inf.get('word_sequence_assistance',[]))==sorted(expected_frames),'exact frame selection')
  if failed:require('postcondition not satisfied' in stderr and not any(x in stderr for x in ['error[E','assertion failed','possible arithmetic underflow/overflow','precondition not satisfied']),'healthy exact postcondition negative')
  receipt['outcomes'].append(dict(label=label,verification=data['verification-results'],failed=failed,scope=scope,frames=inf.get('word_sequence_assistance',[]),pairs=inf.get('set_pair_assistance',[]),times=data['times-ms'],source_sha256=digest(src),stdout_sha256=digest(D/(label+'.stdout')),stderr_sha256=digest(D/(label+'.stderr'))))
 check('effectful_tail_valid_off',call,False,[],[])
 check('effectful_tail_valid_on',call,True,[],['U64GroupedBitmap::clear'])
 check('effectful_tail_missing_proof',missing,True,['U64GroupedBitmap::set'],['U64GroupedBitmap::clear'])
 check('nested_tail_write',nested,True,[],['U64GroupedBitmap::clear','U64GroupedBitmap::set'])
 check('unrelated_field_write',field,True,[],['U64GroupedBitmap::clear'])
print('INDEPENDENT SOURCE PROBES COMPLETE')
