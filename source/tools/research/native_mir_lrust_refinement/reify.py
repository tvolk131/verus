#!/usr/bin/env python3
"""Untrusted structured MIR subset selector. No native semantic certification."""
import copy, hashlib, json
from pathlib import Path
HERE = Path(__file__).resolve().parent
SOURCE = Path('/tmp/native-semantic-bridge-e4a8dcab/source-v1')

def need(ok, why):
    if not ok: raise ValueError(why)

def place(n, ty): return dict(local=n, projection=[], ty=ty)
def operand(n, ty): return dict(op='copy', place=place(n,ty))
def assign(n,ty,value): return dict(op='assign',destination=place(n,ty),value=value)
def cast(n,ty,src,src_ty,kind):
    return assign(n,ty,dict(op='cast',kind=kind,operand=operand(src,src_ty),to=ty))
SELECTED = {
 (0,1): ('Metadata',assign(2,'usize',dict(op='unary',kind='PtrMetadata',operand=operand(1,'&mut [u8]')))),
 (0,6): ('WrapWide',cast(4,'std::ptr::NonNull<[u8]>',7,'*mut [u8]','Transmute')),
 (0,10): ('UnwrapWide',cast(9,'*mut [u8]',4,'std::ptr::NonNull<[u8]>','Transmute')),
 (0,11): ('Thin',cast(8,'*mut u8',9,'*mut [u8]','PtrToPtr')),
 (0,13): ('WrapThin',cast(3,'std::ptr::NonNull<u8>',8,'*mut u8','Transmute')),
 (2,1): ('UnwrapThin',cast(6,'*mut u8',3,'std::ptr::NonNull<u8>','Transmute')),
 (2,2): ('ByteOffset',assign(5,'*mut u8',dict(op='binary',kind='Offset',left=operand(6,'*mut u8'),right=operand(2,'usize')))),
}
def select(inst):
    blocks={b['bb']:b for b in inst['blocks']}
    need(set(blocks)=={0,1,2,3} and all(not b['cleanup'] for b in blocks.values()), 'unsupported control/cleanup')
    for (bb,i),(op,want) in SELECTED.items():
        need(blocks[bb]['statements'][i]['ir']==want, 'unsupported operand/operation '+str((bb,i)))
    branch=blocks[0]['term_ir']; v=branch['operand']
    need(branch['op']=='switch' and branch['cases']==[dict(target=2,value='0')] and branch['otherwise']==1,
         'unsupported branch targets')
    need(v['op']=='constant' and v['evaluation_ok'] is True and v['bits']=='0' and v['ty']=='bool'
         and v['zero_sized'] is False, 'CTFE false required')
    locals={l['local']:l for l in inst['locals']}
    expected={1:('&mut [u8]',16,8,[0,8]),2:('usize',8,8,[]),
      3:('std::ptr::NonNull<u8>',8,8,[0]),4:('std::ptr::NonNull<[u8]>',16,8,[0]),
      5:('*mut u8',8,8,[]),6:('*mut u8',8,8,[]),7:('*mut [u8]',16,8,[0,8]),
      8:('*mut u8',8,8,[]),9:('*mut [u8]',16,8,[0,8])}
    for n,(ty,size,align,offsets) in expected.items():
        l=locals[n]; layout=l['layout']
        need(l['ty']==ty and (layout['size'],layout['align'],layout['field_offsets'])==(size,align,offsets), 'unsupported layout '+str(n))
        if n in [3,4]:
            shape=l['shape']
            need(shape['definition']=='std::ptr::NonNull' and shape['kind']=='adt'
              and shape['repr_transparent'] is True and shape['has_destructor'] is False
              and shape['variants']==[dict(name='NonNull',fields=[dict(name='pointer',ty='(*const '+('u8' if n==3 else '[u8]')+') is !null')])], 'unsupported NonNull validity layout')
    return [op for op,_ in SELECTED.values()]+['U8ZstBranch']

def run_reifier(output=None):
    pin=json.loads((HERE.parent/'native_semantic_bridge/source-bindings.json').read_text())
    blobs={}
    for name,sha in {'receipt.json':pin['receipt_sha256'],**pin['outputs']}.items():
        blobs[name]=(SOURCE/name).read_bytes()
        need(hashlib.sha256(blobs[name]).hexdigest()==sha,'unauthenticated '+name)
    receipt=json.loads(blobs['receipt.json']);need(receipt['success'] is True,'unsuccessful source receipt')
    d=json.loads(blobs['drop-native.json']); borrow=json.loads(blobs['drop-borrowck.json'])
    need(d['pointer_bits']==64 and d['operations']==44 and len(d['instances'])==6
         and d['extraction_blockers']==[] and d['pending_instances']==[], 'unexpected native graph')
    inst=[i for i in d['instances'] if '::new)' in i['display']]
    need(len(inst)==1,'constructor ambiguity')
    ops=select(inst[0]); coverage=[]
    for i in d['instances']:
        for b in i['blocks']:
            for j,_ in enumerate(b['statements']+[dict(ir=b['term_ir'])]):
                term=j==len(b['statements']); key=(b['bb'],j)
                status='unsupported'
                if i['id']==inst[0]['id']:
                    if b['bb']==1: status='CTFE-unreachable'
                    elif key in SELECTED and not term: status='supported-fragment'
                    elif b['bb']==0 and term: status='supported-fragment'
                coverage.append(dict(instance=i['id'],bb=b['bb'],index='term' if term else j,status=status))
    need(len(coverage)==44,'coverage incomplete')
    result=dict(schema=1,producer_trusted=False,source_semantics_link_proved=False,publication=False,
      supported_operations=ops,source_constructor=inst[0]['id'],coverage=coverage,
      counts={s:sum(c['status']==s for c in coverage) for s in ['supported-fragment','CTFE-unreachable','unsupported']},
      complete_native_translation=False,raw_reference_creation='unsupported',
      borrowck_authenticated=True,borrowck_supplies_permission=False)
    if output: Path(output).write_text(json.dumps(result,indent=2)+'\n')
    return result

def controls():
    d=json.loads((SOURCE/'drop-native.json').read_text()); inst=next(i for i in d['instances'] if '::new)' in i['display'])
    cases={}
    for label,mutate in [
      ('retag',lambda x:x['blocks'][0]['statements'][1]['ir'].update(op='retag')),
      ('wrong-original-operand',lambda x:x['blocks'][0]['statements'][1]['ir']['value']['operand']['place'].update(local=7)),
      ('wrong-offset',lambda x:x['blocks'][2]['statements'][2]['ir']['value'].update(kind='Add')),
      ('cleanup',lambda x:x['blocks'][2].update(cleanup=True)),
      ('ctfe-true',lambda x:x['blocks'][0]['term_ir']['operand'].update(bits='1')),
      ('layout',lambda x:x['locals'][3]['layout'].update(size=16)),
      ('nonnull-valid-range',lambda x:x['locals'][3]['shape']['variants'][0]['fields'][0].update(ty='*const u8')),
    ]:
        x=copy.deepcopy(inst);mutate(x)
        try: select(x)
        except ValueError as e: cases[label]=dict(rejected=True,reason=str(e))
        else: raise ValueError('mutation accepted '+label)
    return cases
def concrete_edges():
    pairs=[('SliceDesc p 128','Usize 128'), ('RawWide p 128','NonNullWide p 128'),
           ('NonNullWide p 128','RawWide p 128'), ('RawWide p 128','RawThin p'),
           ('RawThin p','NonNullThin p'), ('NonNullThin p','RawThin p'),
           ('RawThin p; Usize 128','RawThin (shifted p 128)'), ('Boolean false','Usize 2')]
    ops=[op for op,_ in SELECTED.values()]+['U8ZstBranch']
    text='Definition selected_edges w start : list (operation * list descriptor * list descriptor) :=\n'+\
      ' let p := Pointer w start in [\n'+\
      ';\n'.join(' ('+op+', ['+ins+'], ['+outs+'])' for op,(ins,outs) in zip(ops,pairs))+'].\n'
    text += """
Lemma selected_edges_checked w start σ :
  window_valid w → 0 < start → start + 128 < window_extent w →
  Forall (λ edge, source_step edge.1.1 edge.1.2 σ edge.2 σ) (selected_edges w start).
Proof.
  intros W S E.
  destruct (interior128_offset w start W S E) as [Off _].
  assert (PV : pointer_valid (Pointer w start)) by exact (proj1 Off).
  assert (SL : slice_valid (Pointer w start) 128).
  { split; first done. split; [lia|exact (proj2 (proj2 Off))]. }
  unfold selected_edges.
"""
    for op in ops:
        proof='constructor' if op=='U8ZstBranch' else 'apply Step'+op+'. exact '+('Off' if op=='ByteOffset' else 'PV' if op in ['WrapThin','UnwrapThin'] else 'SL')
        text+='  constructor. { '+proof+'. }\n'
    return text+'  constructor.\nQed.\n'+"""
Theorem selected_edges_head_simulation w start σ :
  window_valid w → 0 < start → start + 128 < window_extent w →
  Forall (λ edge, ∃ es,
    translate edge.1.1 (input_words edge.1.2) = Some es ∧
    Forall2 (λ e v, head_step e σ [] (of_val v) σ [])
      es (input_words edge.2)) (selected_edges w start).
Proof.
  intros W S E.
  pose proof (selected_edges_checked w start σ W S E) as Steps.
  eapply Forall_impl; first exact Steps.
  intros [[op ins] outs] Step.
  destruct (supported_operation_head_simulation op ins σ outs σ Step)
    as [_ [es [Tr Sim]]]. exists es. split; first done.
  eapply Forall2_impl; first exact Sim. intros e v [Head _]. exact Head.
Qed.
"""

def proof_source(result):
    ops=result['supported_operations']
    return ('From NativeMirLrust Require Export simulation.\nOpen Scope Z_scope.\n'+
      'Definition selected_operations : list operation := ['+'; '.join(ops)+'].\n'+
      'Lemma selected_operations_exact : selected_operations =\n ['+'; '.join(ops)+'].\nProof. reflexivity. Qed.\n'+
      'Lemma selected_operations_supported : Forall (λ op, ∀ why, op ≠ Unsupported why) selected_operations.\n'+
      'Proof. repeat constructor; intros why H; discriminate. Qed.\n'+concrete_edges())

if __name__=='__main__':
    import sys
    result=run_reifier()
    if '--controls' in sys.argv: print(json.dumps(controls(),indent=2))
    else:
        (HERE/'source-coverage.json').write_text(json.dumps(result,indent=2)+'\n')
        (HERE/'candidates/reified.v').write_text(proof_source(result))
