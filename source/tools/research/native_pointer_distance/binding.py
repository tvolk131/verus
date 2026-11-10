"""Structured operand/control eligibility. Not a Rust semantics theorem."""
import copy, hashlib, json
from pathlib import Path
HERE=Path(__file__).resolve().parent
RAW=Path('/tmp/native-effect-soundness-ced1eb20/source-v1/extraction/original.json')
def need(c,m):
    if not c: raise ValueError(m)
def place(p,local,ty,projection=None):
    need(p == dict(local=local,ty=ty,projection=projection or []),'typed operand/place')
def operand(o,local,ty,mode='copy',projection=None):
    need(o['op']==mode,'operand mode');place(o['place'],local,ty,projection)
def constant(o,n,ty='usize'):
    need(o['op']=='constant' and o['evaluation_ok'] is True and o['bits']==str(n) and o['ty']==ty,'CTFE/constant')
def switch(t,local,zero,other):
    need(t['op']=='switch' and t['cases']==[dict(target=zero,value='0')] and t['otherwise']==other,'branch target')
    operand(t['operand'],local,'bool','move')
def assignments(b):
    need(not b['cleanup'],'unexpected cleanup')
    need(all(s['ir']['op'] in ['assign','storage_live','storage_dead'] for s in b['statements']),'unsupported operation')
    return [s['ir'] for s in b['statements'] if s['ir']['op']=='assign']
def assigned(s,n,ty):
    need(s['op']=='assign','assignment');place(s['destination'],n,ty);return s['value']
def call(b,target_id,args,dest,ty,target):
    t=b['term_ir'];need(t['op']=='call' and b['call']['resolved']==target_id,'resolved edge')
    need(t['target']==target,'return edge');place(t['destination'],dest,ty)
    need(len(t['arguments'])==len(args),'call arity')
    for o,(n,typ) in zip(t['arguments'],args):operand(o,n,typ,'move')
    need(t['unwind']==dict(action='unreachable'),'source exceptional edge changed')
def check_hint(d,pre,intr,mutable):
    blocks={b['bb']:b for b in d['blocks']};need(set(blocks)==set(range(11)),'block coverage')
    raw='*mut u8' if mutable else '*const u8'; nn='std::ptr::NonNull<u8>'
    for l in d['locals']:
        if l['ty'] in ['usize',raw,'*const u8','*const ()',nn]:
            need(l['layout']['size']==8 and l['layout']['align']==8,'native width/layout')
    a=assignments(blocks[0]);need(a==[],'CTFE statements')
    t=blocks[0]['term_ir'];constant(t['operand'],0,'bool')
    need(t['op']=='switch' and t['cases']==[dict(target=2,value='0')] and t['otherwise']==1,'u8 branch')
    a=assignments(blocks[2]);need(len(a)==6,'pointer chain count')
    for s,dst,src,typ,to,field in [
        (a[0],5,1,raw,raw,1),(a[1],4,5,raw,nn,None),
        (a[2],6,1,nn,nn,0),(a[3],9,4,nn,'*mut u8',None),
        (a[4],10,6,nn,'*const u8',None),(a[5],11,4,nn,'*const u8',None)]:
        v=assigned(s,dst,to)
        if field is not None:
            need(v['op']=='use' and v['retag']=='Yes','field copy')
            operand(v['operand'],src,typ,projection=[dict(op='deref'),dict(op='field',field=field,ty=typ)])
        else:
            need(v['op']=='cast' and v['kind']=='Transmute' and v['to']==to,'transparent cast')
            operand(v['operand'],src,typ)
    t=blocks[2]['term_ir'];need(t['op']=='switch' and t['operand']['op']=='runtime_checks' and t['operand']['debug']=='UbChecks','UbChecks')
    need(t['cases']==[dict(target=7,value='0')] and t['otherwise']==5,'UbChecks edges')
    a=assignments(blocks[5]);need(len(a)==2,'check casts')
    for s,dst,src in [(a[0],13,4),(a[1],14,6)]:
        v=assigned(s,dst,'*const ()');need(v['op']=='cast' and v['kind']=='Transmute' and v['to']=='*const ()','check cast')
        operand(v['operand'],src,nn)
    call(blocks[5],pre,[(13,'*const ()'),(14,'*const ()')],12,'()',6)
    for bb,target in [(6,7),(4,3)]:need(blocks[bb]['term_ir']==dict(op='goto',target=target),'goto edge')
    for bb,dst,op,left,right,other in [(7,15,'Lt',0,1,8),(8,16,'Le',1,9223372036854775807,9)]:
        a=assignments(blocks[bb]);need(len(a)==1,'assert operation count');v=assigned(a[0],dst,'bool')
        need(v['op']=='binary' and v['kind']==op,'size comparison');constant(v['left'],left);constant(v['right'],right)
        switch(blocks[bb]['term_ir'],dst,10,other)
    need(assignments(blocks[9])==[],'intrinsic statements')
    call(blocks[9],intr,[(11,'*const u8'),(10,'*const u8')],2,'usize',4)
    a=assignments(blocks[3]);need(len(a)==3,'tuple count')
    v=assigned(a[0],7,'usize');need(v['op']=='use','tuple lower');operand(v['operand'],2,'usize')
    v=assigned(a[1],8,'std::option::Option<usize>')
    need(v['op']=='aggregate' and v['kind']['definition']=='std::option::Option' and v['kind']['variant']==1,'Some constructor')
    need(len(v['operands'])==1,'Some arity');operand(v['operands'][0],2,'usize','move')
    v=assigned(a[2],0,'(usize, std::option::Option<usize>)');need(v['op']=='aggregate' and v['kind']==dict(op='tuple'),'tuple constructor')
    need(len(v['operands'])==2,'tuple arity');operand(v['operands'][0],7,'usize','move');operand(v['operands'][1],8,'std::option::Option<usize>','move')
    need(blocks[3]['term_ir']==dict(op='return'),'return')
    # Failure and ZST blocks are retained, excluded only by the conditional Coq lemmas.
    return dict(id=d['id'],field_start=0,field_end=1,actual_intrinsic_arguments=[11,10],
                check_arguments=[13,14],distance_local=2,returned_tuple=[7,8],
                selected_branch_blocks=[0,2,5,6,7,8,9,4,3],retained_failure_blocks=[1,10],
                source_correspondence_proved=False)
def validate(d):
    need(d['pointer_bits']==64 and len(d['instances'])==32 and d['operations']==408,'frozen full graph')
    xs=d['instances'];need(xs[27]['kind']=='intrinsic' and 'ptr_offset_from_unsigned' in xs[27]['id'],'intrinsic identity')
    pre=xs[26];bb=pre['blocks'][0];a=assignments(bb);need(len(a)==1,'order check count')
    v=assigned(a[0],3,'bool');need(v['op']=='binary' and v['kind']=='Ge','order comparison')
    operand(v['left'],1,'*const ()');operand(v['right'],2,'*const ()');switch(bb['term_ir'],3,2,1)
    need(pre['blocks'][1]['term_ir']==dict(op='return'),'check return')
    hints=[check_hint(xs[i],pre['id'],xs[27]['id'],i==21) for i in [21,22]]
    for i,h in zip([15,16],[21,22]):
        x=xs[i];need(x['blocks'][0]['call']['resolved']==xs[h]['id'],'size call edge')
        t=x['blocks'][0]['term_ir'];operand(t['arguments'][0],1,x['locals'][1]['ty'],'move')
        a=assignments(x['blocks'][1]);need(len(a)==1,'size projection count')
        v=assigned(a[0],0,'usize');need(v['op']=='use','size .0');operand(v['operand'],2,'usize',projection=[dict(op='field',field=0,ty='usize')])
    # Specialized Zip receives both projected sizes and records index=0/len=min.
    z=xs[11];zb={b['bb']:b for b in z['blocks']}
    for bb,which,dest,ret in [(0,15,4,1),(1,16,6,2)]:
        t=zb[bb]['term_ir'];need(zb[bb]['call']['resolved']==xs[which]['id'],'Zip size origin')
        place(t['destination'],dest,'usize');need(t['target']==ret and t['unwind']==dict(action='cleanup',target=3),'Zip size edges')
    t=zb[2]['term_ir'];need(zb[2]['call']['resolved']==xs[17]['id'],'Zip min target')
    need(len(t['arguments'])==2,'min arity');operand(t['arguments'][0],4,'usize','move');operand(t['arguments'][1],6,'usize','move')
    place(t['destination'],3,'usize');need(t['target']==6 and t['unwind']==dict(action='cleanup',target=3),'Zip min edges')
    aa=assignments(zb[6]);need(len(aa)==3,'Zip aggregate count')
    for stmt,dest,src,typ in [(aa[0],8,1,z['locals'][1]['ty']),(aa[1],9,2,z['locals'][2]['ty'])]:
        v=assigned(stmt,dest,typ);need(v['op']=='use','Zip inner preservation');operand(v['operand'],src,typ,'move')
    v=assigned(aa[2],0,z['locals'][0]['ty']);need(v['op']=='aggregate' and v['kind']['definition']=='std::iter::Zip','Zip aggregate')
    oo=v['operands'];need(len(oo)==4,'Zip field count')
    operand(oo[0],8,z['locals'][1]['ty'],'move');operand(oo[1],9,z['locals'][2]['ty'],'move');constant(oo[2],0);operand(oo[3],3,'usize')
    # Actual min chooses b when b<a; its drop/cleanup transitions remain unsupported.
    mn=xs[17];mb={b['bb']:b for b in mn['blocks']}
    aa=assignments(mb[0]);need(len(aa)==3,'min input count')
    for stmt,dst,src in [(aa[1],4,2),(aa[2],5,1)]:
        v=assigned(stmt,dst,'&usize');need(v['op']=='ref' and v['kind']=='Shared','min argument borrow');place(v['place'],src,'usize')
    t=mb[0]['term_ir'];need(mb[0]['call']['resolved']==xs[23]['id'],'min comparison edge')
    operand(t['arguments'][0],4,'&usize','move');operand(t['arguments'][1],5,'&usize','move');place(t['destination'],3,'bool')
    switch(mb[1]['term_ir'],3,3,2)
    for bb,src in [(2,2),(3,1)]:
        v=assigned(assignments(mb[bb])[-1],0,'usize');need(v['op']=='use','min selected return');operand(v['operand'],src,'usize','move')
    lt=assignments(xs[23]['blocks'][0]);need(len(lt)==3,'lt count')
    for stmt,dst,src in [(lt[0],3,1),(lt[1],4,2)]:
        v=assigned(stmt,dst,'usize');need(v['op']=='use','lt load');operand(v['operand'],src,'usize',projection=[dict(op='deref')])
    v=assigned(lt[2],0,'bool');need(v['op']=='binary' and v['kind']=='Lt','min comparison meaning')
    operand(v['left'],3,'usize','move');operand(v['right'],4,'usize','move')
    # Compact coverage retains every exceptional/cleanup edge without declaring
    # those blocks simulated. Full structured raw inputs stay private and pinned.
    summaries=[]
    for i in [11,15,16,17,21,22,23,26,27]:
        x=xs[i];summaries.append(dict(index=i,id=x['id'],blocks=[dict(bb=b['bb'],cleanup=b['cleanup'],
            operation_count=len(b['statements'])+1,terminator=b['term_ir']['op'],
            successors=b.get('successors'),call=b.get('call'),unwind=b['term_ir'].get('unwind'),
            structured_sha256=hashlib.sha256(json.dumps(dict(statements=[s['ir'] for s in b['statements']],
                term_ir=b['term_ir']),sort_keys=True,separators=(',',':')).encode()).hexdigest())
            for b in x.get('blocks',[])]))
    return dict(schema=1,structural_eligibility_only=True,source_correspondence_proved=False,hints=hints,
                preserved_graph_coverage=summaries,publication=False,native_correspondences=0)

def check_source(): return validate(json.loads(RAW.read_bytes()))
def controls():
    original=json.loads(RAW.read_bytes());out={}
    def run(name,mut):
        d=copy.deepcopy(original);mut(d)
        try:validate(d)
        except (ValueError,KeyError) as e:out[name]=dict(rejected=True,reason=str(e));return
        raise ValueError('control accepted: '+name)
    run('wrong-distance-order',lambda d:d['instances'][21]['blocks'][9]['term_ir']['arguments'].reverse())
    def origin(d):
        a=[s['ir'] for s in d['instances'][21]['blocks'][2]['statements'] if s['ir']['op']=='assign'];a[2]['value']['operand']['place']['projection'][1]['field']=1
    run('wrong-origin-field',origin)
    run('wrong-width',lambda d:d.update(pointer_bits=32))
    run('wrong-branch',lambda d:d['instances'][21]['blocks'][7]['term_ir'].update(otherwise=10))
    run('wrong-ctfe',lambda d:d['instances'][22]['blocks'][0]['term_ir']['operand'].update(bits='1'))
    run('wrong-address-order',lambda d:next(s['ir'] for s in d['instances'][26]['blocks'][0]['statements'] if s['ir']['op']=='assign')['value'].update(kind='Le'))
    def wrong_min(d):
        a=[s['ir'] for s in d['instances'][11]['blocks'][-1]['statements'] if s['ir']['op']=='assign']
        a[-1]['value']['operands'][-1]['place']['local']=6
    run('wrong-Zip-length',wrong_min)
    def wrong_min_cmp(d):
        a=[s['ir'] for s in d['instances'][23]['blocks'][0]['statements'] if s['ir']['op']=='assign']
        a[-1]['value']['kind']='Le'
    run('wrong-min-comparison',wrong_min_cmp)
    run('changed-exception-edge',lambda d:d['instances'][21]['blocks'][9]['term_ir'].update(unwind=dict(action='continue')))
    return out
