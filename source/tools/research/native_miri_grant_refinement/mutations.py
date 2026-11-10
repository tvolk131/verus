"""Executable semantic mutations, isolated alongside identical healthy originals."""
import json, pathlib, re, tempfile
from replay_support import *
from fidelity import strict_check, block_end

def extract(text,name):
    start=text.index('fn '+name+'(');brace=text.index('\n{',start)+1
    return text[start:block_end(text,brace)]

def main():
    strict_check(HERE/'component.rs');base=(HERE/'component.rs').read_text()
    out=pathlib.Path(tempfile.mkdtemp(prefix='mutations-',dir=private_root()))
    edits=[('boundary','find_first_write_incompatible','mutated_boundary','granting + 1\n            }','granting\n            }','Stack'),
           ('permission','from_bits','mutated_permission','Self::UNIQUE => Permission::Unique,','Self::UNIQUE => Permission::SharedReadWrite,','Permission'),
           ('bitfield','set_permission','mutated_bitfield','self.0 |= perm.to_bits() << PERM_SHIFT;','self.0 |= perm.to_bits() << 60;','Item')]
    rows={}
    for label,original,mutant,before,after,carrier in edits:
        body=extract(base,original);require(body.count(before)==1,'ambiguous mutation')
        changed=body.replace('fn '+original+'(', 'fn '+mutant+'(',1).replace(before,after,1)
        src=out/(label+'.rs');src.write_text(base.rstrip()[:-1]+'\nimpl '+carrier+' {\n'+changed+'\n}\n}\n')
        rows[label]={'source':str(src),'sha256':digest(src),'symbol':carrier+'::'+mutant,'original':original,'before':before,'after':after,
                     'original_function_sha256':__import__('hashlib').sha256(body.encode()).hexdigest(), 'original_prefix_identical':src.read_text().startswith(base.rstrip()[:-1])}
    freeze,bundle,files=authenticate();rec={'output':str(out),'commands':[],'mutations':rows,'classification':'isolated_executable_semantic_negatives'}
    (out/'input-freeze.json').write_text(json.dumps(rec,indent=2)+'\n')
    with replay_receipt(out,rec):
        guard=ArtifactGuard(files+[pathlib.Path(v['source']) for v in rows.values()]);rec['artifacts']=guard.initial
        run=LoggedCommands(out,out,rec['commands'],timeout=180,guard=guard,status_key='returncode')
        for label,row in rows.items():
            rc=run(label,[str(bundle/'verus'),row['source'],'--crate-type=lib','--no-cheating','--output-json','--multiple-errors','20','--infer-goals'],environment(bundle,out))
            stdout=(out/(label+'.stdout')).read_text();stderr=(out/(label+'.stderr')).read_text();data=json.loads(stdout)
            validate_run({'returncode':rc,'stdout':stdout,'stderr':stderr},False)
            v=data['verification-results'];inf=data['verus-infer'];scope=inf['verification_scope']
            require(v['is-verifying-entire-crate'] is True and scope['native_collection_completed'] is True)
            require(v['verified']>=22 and v['errors']>=1,'original obligations unhealthy')
            selected={b['rust_function']:b for b in scope['native_functions'] if b['rust_function'] in EXEC|{row['symbol']}}
            require(set(selected)==EXEC|{row['symbol']} and all(b['selected'] for b in selected.values()),'mutation body coverage')
            require(inf['functions']==[] and inf['goal_assistance']==[] and inf['interface_written'] is False)
            diagnostics=[line for line in stderr.splitlines() if line.startswith('error:') and not line.startswith('error: aborting')]
            require(diagnostics and all(d=='error: postcondition not satisfied' for d in diagnostics),'mixed diagnostic failure')
            spans=re.findall(r'error: postcondition not satisfied\n\s*--> .*?\.rs:(\d+):',stderr)
            src=pathlib.Path(row['source']).read_text();start=src[:src.index('fn '+row['symbol'].split('::')[1])].count('\n')+1
            require(len(spans)==len(diagnostics) and all(int(n)>=start for n in spans),'failure outside mutated function')
            row.update(returncode=rc,verification=v,diagnostics=diagnostics,error_lines=spans,stdout_sha256=digest(out/(label+'.stdout')),stderr_sha256=digest(out/(label+'.stderr')),scope=scope)
        guard.check();rec['artifacts_after']=guard.snapshot()
    print(out)
if __name__=='__main__':main()
