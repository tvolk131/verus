"""Immutable private diagnostic epochs. No production edits, Cargo, or network.
Run python3 -B; never count frontend/setup failures as semantic negatives.
"""
import hashlib, importlib.util, json, os, pathlib, shutil, stat, subprocess, sys, tempfile
HERE=pathlib.Path(__file__).resolve().parent
ROOT=HERE.parents[3]
TC=pathlib.Path('/home/dev/.rustup/toolchains/1.99.1-x86_64-unknown-linux-gnu')
B=pathlib.Path('/tmp/verus-infer-buffer-cycle-20261109/parent-combined74-bundle-d3b2aa6e')
Z=pathlib.Path('/tmp/local-nonzero-ord-09t1tshb/bundle/z3')
ITEM=pathlib.Path('/tmp/local-nonzero-ord-09t1tshb/source-1adb8b6c2ee64cc1.rs')
def need(b,m):
    if not b: raise RuntimeError(m)
def sha(p): return hashlib.sha256(pathlib.Path(p).read_bytes()).hexdigest()
def write(p,j): p.write_text(json.dumps(j,indent=2)+'\n')
def main():
    need(sys.flags.dont_write_bytecode,'use -B')
    r=pathlib.Path(tempfile.mkdtemp(prefix='rev-nominal-definition-')); r.chmod(0o700)
    print(r,flush=True)
    result={'schema':1,'private_root':str(r),'diagnostic_only':True,'trials':{},'publication':False,'goal_complete':False,'native_correspondences':0,'production_fact_installed':False}
    env=dict(PATH=str(TC/'bin')+':/home/dev/.cargo/bin:/usr/bin:/bin',RUSTUP_HOME='/home/dev/.rustup',RUSTUP_TOOLCHAIN='1.99.1',RUSTC_BOOTSTRAP='1',LD_LIBRARY_PATH=str(TC/'lib'),CARGO_BUILD_JOBS='2',CARGO_NET_OFFLINE='true')
    for n in ['home','cargo','target','tmp','cache']:(r/n).mkdir()
    env.update(HOME=str(r/'home'),CARGO_HOME=str(r/'cargo'),CARGO_TARGET_DIR=str(r/'target'),TMPDIR=str(r/'tmp'),XDG_CACHE_HOME=str(r/'cache'))
    def run(label,args):
        d=r/label;d.mkdir(); args=list(map(str,args))
        with (d/'stdout').open('wb') as out,(d/'stderr').open('wb') as err:
            p=subprocess.run(args,cwd=ROOT,env=env,stdout=out,stderr=err)
        j={'argv':args,'exit':p.returncode,'stdout':{'path':str(d/'stdout'),'sha256':sha(d/'stdout')},'stderr':{'path':str(d/'stderr'),'sha256':sha(d/'stderr')}}
        try:j['verification']=json.loads((d/'stdout').read_text())['verification-results']
        except (ValueError,KeyError):pass
        write(d/'receipt.json',j);result['trials'][label]=j
        print(label,p.returncode,j.get('verification',''),flush=True)
        return j
    try:
        freeze=json.loads((HERE/'FREEZE.json').read_text())
        need(sha(HERE/'INTERFACE.json')==freeze['interface_sha256'],'preregistered interface')
        for p,h in freeze['inputs'].items():need(sha(ROOT/p)==h,'input '+p)
        for p,h in freeze['toolchain'].items():need(sha(p)==h,'pinned toolchain '+p)
        need(sha(ITEM)==freeze['allowed_item']['sha256'],'genuine Item fixture')
        need(sha(B/'receipt.json')=='5e7e79cf6b6cef0509226cf6134c8fce0a8c47ed608fc4c5a7520b5023aa4c45','parent receipt')
        receipt=json.loads((B/'receipt.json').read_text())
        names={'verus','rust_verify','z3','libverus_builtin.rlib','libverus_builtin_macros.so','libverus_state_machines_macros.so','libvstd.rlib','vstd.vir'}
        need(set(receipt['tools'])==names,'eight frozen tools')
        bundle=r/'bundle';bundle.mkdir()
        for n,h in receipt['tools'].items():
            src=Z if n=='z3' else B/n
            need(sha(B/n)==h and sha(src)==h,'input tool '+n)
            shutil.copy2(src,bundle/n);need(sha(bundle/n)==h,'copied tool '+n)
        need(sha(Z)=='e583c4186a45e72411fa2cb2048401eed03f0f8e5f24694676a8f6271a50b765','exact solver')
        need(os.access(bundle/'z3',os.X_OK),'solver executable')
        env['VERUS_Z3_PATH']=str(bundle/'z3')
        production_paths=subprocess.check_output(['git','ls-tree','-r','--name-only',freeze['parent_head'],'source'],cwd=ROOT,text=True).splitlines()
        production={p:sha(ROOT/p) for p in production_paths if not p.startswith(('source/tools/research/','source/docs/'))}
        controllers={str(p.relative_to(ROOT)):sha(p) for p in sorted(HERE.iterdir()) if p.is_file() and p.suffix in ['.py','.rs','.md','.json'] and p.name not in ['RESULTS.json','ATTEMPTS.json','REPORT.md','VALIDATION.json','DECISION.json','IDENTITIES.json','INVENTORY.json']}
        f={'tools':receipt['tools'],'toolchain':freeze['toolchain'],'inputs':freeze['inputs'],'production':production,'controllers':controllers,'environment':env,'item_sha256':sha(ITEM),'solver_mode':oct(stat.S_IMODE((bundle/'z3').stat().st_mode)),'receipt_sha256':sha(B/'receipt.json')}
        write(r/'freeze.json',f)
        result.update(tools=f['tools'],toolchain=f['toolchain'])
        run('rustc-version',[TC/'bin/rustc','-vV'])
        if len(sys.argv)==3 and sys.argv[1]=='--reuse-inspector':
            previous=pathlib.Path(sys.argv[2]); old=json.loads(previous.read_text())
            oldroot=pathlib.Path(old['private_root']); of=json.loads((oldroot/'freeze.json').read_text())
            need(of['controllers'][str((HERE/'inspect.rs').relative_to(ROOT))]==sha(HERE/'inspect.rs'),'reused inspector source')
            need(of['toolchain']==f['toolchain'],'reused inspector toolchain')
            need(old['trials']['driver-build']['exit']==0,'completed own inspector build')
            need(sha(oldroot/'inspect')==old['products'][str(oldroot/'inspect')],'reused inspector binary')
            shutil.copy2(oldroot/'inspect',r/'inspect')
            result['reused_inspector']={'results':{'path':str(previous),'sha256':sha(previous)},'binary':{'path':str(oldroot/'inspect'),'sha256':sha(oldroot/'inspect')}}
        else:
            need(len(sys.argv)==1,'arguments')
            j=run('driver-build',[TC/'bin/rustc',HERE/'inspect.rs','--edition=2024','-o',r/'inspect','-L',TC/'lib','-C','rpath=yes'])
            need(j['exit']==0,'driver build failure')
        common=['--crate-name','rev_item_witness','--crate-type=lib','--edition=2024','--extern','vstd='+str(bundle/'libvstd.rlib'),'-L','dependency='+str(bundle)]
        witness=ITEM.read_text()+'''\n// Original genuine Item declaration and runtime bodies above are byte-preserved.
pub fn actual_rev<'a>(e: core::iter::Enumerate<core::slice::Iter<'a,Item>>) -> core::iter::Rev<core::iter::Enumerate<core::slice::Iter<'a,Item>>> { e.rev() }
pub fn actual_next<'a>(r: &mut core::iter::Rev<core::iter::Enumerate<core::slice::Iter<'a,Item>>>) -> Option<(usize,&'a Item)> { r.next() }
pub fn actual_back<'a>(r: &mut core::iter::Rev<core::iter::Enumerate<core::slice::Iter<'a,Item>>>) -> Option<(usize,&'a Item)> { r.next_back() }
pub fn original<'a>(borrows:&'a Vec<Item>) { for (_,_) in borrows.iter().enumerate().rev() {} }
'''
        p=r/'genuine_item.rs';p.write_text(witness)
        j=run('item-native-compile',[TC/'bin/rustc',p,*common,'-o',r/'item.rlib'])
        need(j['exit']==0,'ordinary genuine Item Rust compile failure')
        j=run('item-mir',[r/'inspect',p,*common,'--sysroot',TC,'-C','opt-level=0','-Z','mir-opt-level=0','--emit=metadata','-o',r/'item.rmeta'])
        need(j['exit']==0,'actual Item diagnostic extraction failure')
        # Unchanged baselines only: no changed compiler/model or mixed artifacts.
        spec=importlib.util.spec_from_file_location('old_probes',HERE.parent/'checked_rev_bridge_design/probes.py')
        probes=importlib.util.module_from_spec(spec);spec.loader.exec_module(probes)
        for name in probes.BODIES:
            p=r/(name+'.rs');p.write_text(probes.source(name))
            for infer in [False,True]:
                label=name+('-on' if infer else '-off')
                j=run(label,[bundle/'verus',p,'--crate-name','rev_probe','--crate-type=lib','--no-cheating','--compile','-o',r/(label+'.rlib'),'--output-json','--multiple-errors','10',*(['--infer-goals'] if infer else [])])
                need('verification' in j and not j['verification']['encountered-vir-error'],'baseline frontend/setup failure '+label)
        p=r/'compatibility.rs';shutil.copyfile(HERE/'compatibility.rs',p)
        j=run('compatibility',[bundle/'verus',p,'--crate-type=lib','--no-cheating','--compile','-o',r/'compatibility.rlib','--output-json','--multiple-errors','10'])
        need('verification' in j and j['verification']['success'],'compatibility not checked')
        for kind,claim in [('false-inner','f(c(a))==b'),('false-law','law(c(a))'),('false-index','Seq::new(s.len(), |j:int| ((n as int+j) as usize,s[j])).reverse()[0].0==n')]:
            args={'false-inner':('a:u64,b:u64','a!=b'),'false-law':('a:State<u64>','!a.lawful'),'false-index':('s:Seq<u64>, n:usize','s.len()>1, n as int+s.len()<=usize::MAX')}[kind]
            extra='\nverus! { pub proof fn negative('+args[0]+') requires '+args[1]+', { assert('+claim+'); } }\n'
            q=r/(kind.replace('-','_')+'.rs');q.write_text(p.read_text()+extra)
            j=run(kind,[bundle/'verus',q,'--crate-type=lib','--no-cheating','--compile','-o',r/(kind+'.rlib'),'--output-json','--multiple-errors','10'])
            need('verification' in j and not j['verification']['encountered-vir-error'] and j['verification']['errors']==1,'not a healthy assertion negative '+kind)
        # This is the precise undischarged native field-frame premise, tested
        # only as a logical non-entailment, never a native counterexample.
        q=r/'missing_frame.rs'
        q.write_text(p.read_text()+"""
verus! {
        pub proof fn missing_frame(old:Carrier<State<u64>>, final_outer:Carrier<State<u64>>,
            before:State<u64>, final_field:State<u64>, out:Option<u64>)
            requires f(old)==before, before.lawful,
                back_law(before,final_field,out),
                front_law(reversed(before),reversed(f(final_outer)),out),
            { assert(f(final_outer)==final_field); }
        }
""")
        j=run('missing-frame',[bundle/'verus',q,'--crate-type=lib','--no-cheating','--compile','-o',r/'missing_frame.rlib','--output-json','--multiple-errors','10'])
        need('verification' in j and not j['verification']['encountered-vir-error'] and j['verification']['errors']==1,'missing frame must fail at assertion')
        # Controls for actual source ownership/privacy. These are Rust diagnostics,
        # not semantic assertion negatives or a source-to-native proof.
        for label,extra in [
            ('rust-private','pub fn forbidden(r:core::iter::Rev<core::iter::Enumerate<core::slice::Iter<Item>>>) { let _=r.iter; }'),
            ('rust-move','pub fn forbidden(e:core::iter::Enumerate<core::slice::Iter<Item>>) { let _=e.rev(); let _=e.rev(); }'),
            ('rust-borrow','pub fn forbidden(v:&mut Vec<Item>) { let mut r=v.iter().enumerate().rev(); v.clear(); let _=r.next(); }')]:
            q=r/(label.replace('-','_')+'.rs');q.write_text(witness+'\n'+extra+'\n')
            run(label,[TC/'bin/rustc',q,*common,'-o',r/(label+'.rlib')])
        need(all(sha(ROOT/p)==h for p,h in production.items()),'production changed')
        need(all(sha(ROOT/p)==h for p,h in controllers.items()),'controller changed')
        need(all(sha(p)==h for p,h in freeze['toolchain'].items()),'toolchain changed')
        need(all(sha(bundle/n)==h for n,h in f['tools'].items()),'tools changed')
        result['final_guards']=True
        result['status']='DIAGNOSTIC_COMPATIBILITY_ONLY_NATIVE_COHERENCE_UNDISCHARGED'
    except Exception as e:
        result['status']='EXCLUDED_SETUP_OR_DIAGNOSTIC_FAILURE'
        result['failure']=str(e)
    result['products']={str(p):sha(p) for p in r.iterdir() if p.is_file() and p.name!='RESULTS.json'}
    write(r/'RESULTS.json',result)
    # Immutable epochs retained, with no overwrite of any previous run.
    print('RESULT',r/'RESULTS.json',result['status'],flush=True)
    if 'failure' in result: raise RuntimeError(result['failure'])
if __name__=='__main__':main()
