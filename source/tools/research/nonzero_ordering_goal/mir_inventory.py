"""Read-only rustc-driver MIR inventory of the exact local total/partial call closure.
This diagnostic is not a verifier, translation rule, certificate, or native theorem.
"""
from run import *
INSPECTOR = r'''#![feature(rustc_private)]
extern crate rustc_driver;
extern crate rustc_interface;
extern crate rustc_middle;
extern crate rustc_hir;
extern crate rustc_data_structures;
use rustc_middle::{ty::{self, TyCtxt}, mir};
use rustc_driver::{Callbacks, Compilation};
struct Inspect;
impl Callbacks for Inspect {
 fn after_analysis<'tcx>(&mut self, _: &rustc_interface::interface::Compiler, tcx: TyCtxt<'tcx>) -> Compilation {
  let mut queue = vec![];
  for id in tcx.hir_body_owners() {
   if matches!(tcx.item_name(id.to_def_id()).as_str(), "total" | "partial") {
    queue.push((ty::Instance::mono(tcx,id.to_def_id()),0));
   }
  }
  let mut seen = std::collections::HashSet::new();
  let mut cursor = 0;
  while cursor < queue.len() {
   let (instance, depth) = queue[cursor]; cursor += 1;
   if !seen.insert(instance) {continue;}
   println!("BEGIN_INSTANCE {:?}",instance);
   println!("PATH {}",tcx.def_path_str(instance.def_id()));
   println!("DEPTH {depth}");
   if depth > 6 || !tcx.is_mir_available(instance.def_id()) {
    println!("MIR_UNAVAILABLE_OR_DEPTH_LIMIT\nEND_INSTANCE");continue;
   }
   let env = ty::TypingEnv::fully_monomorphized();
   let body = instance.instantiate_mir_and_normalize_erasing_regions(tcx,env,
     ty::EarlyBinder::bind(tcx,tcx.instance_mir(instance.def).clone()));
   use rustc_data_structures::stable_hash::{StableHash, StableHasher};
   let mut hasher = StableHasher::new();
   tcx.with_stable_hashing_context(|mut hcx| {
    hcx.while_hashing_spans(false, |hcx| body.stable_hash(hcx, &mut hasher));
   });
   let fingerprint: rustc_data_structures::fingerprint::Fingerprint = hasher.finish();
   println!("BODY_FINGERPRINT {fingerprint:?}");
   println!("WHOLE_MIR {body:#?}");
   for scope in body.source_scopes.iter() {
    if let Some((callee,_)) = scope.inlined {
     println!("INLINED {:?}",callee);
     queue.push((callee,depth+1));
    }
   }
   for (bb, data) in body.basic_blocks.iter_enumerated() {
    println!("BLOCK {:?} {:?}",bb,data);
    if let mir::TerminatorKind::Call {func,..} = &data.terminator().kind {
     if let ty::FnDef(def,args) = *func.ty(&body.local_decls,tcx).kind() {
      match ty::Instance::try_resolve(tcx,env,def,args.skip_binder()) {
       Ok(Some(callee)) => {
        println!("CALL {:?} => {:?}",bb,callee);
        queue.push((callee,depth+1));
       }
       other => println!("UNRESOLVED_CALL {:?} {:?}",bb,other),
      }
     } else {println!("NON_FNDEF_CALL {:?}",bb);}
    }
   }
   println!("END_INSTANCE");
  }
  Compilation::Stop
 }
}
fn main() {
 let args: Vec<_> = std::env::args().collect();
 rustc_driver::run_compiler(&args,&mut Inspect);
}
'''

def main():
    upstream = authenticate()
    baseline = Path(sys.argv[1])
    # Only a freshly completed owned baseline is accepted as this diagnostic input.
    rec = json.loads((baseline/'receipt.json').read_text())
    need(rec['tool_receipt_sha256'] == BUNDLE_SHA, 'baseline bundle mismatch')
    need(all(x['input_guards_passed'] for x in rec['results'].values()), 'baseline input guards')
    out = Path(tempfile.mkdtemp(prefix='nonzero-ordering-mir-')); out.chmod(0o700)
    print(out,flush=True)
    src = out/'inspect.rs';src.write_text(INSPECTOR)
    native = out/'native.rs';native.write_bytes((baseline/'native.rs').read_bytes())
    env = environment(out)
    inputs = [Path(__file__), HERE/'run.py', TC/'bin/rustc', *sorted((TC/'lib/rustlib/x86_64-unknown-linux-gnu/lib').glob('libcore-*.rlib')), baseline/'receipt.json',
              baseline/'native.rs', src, native, upstream, *sorted((TC/'lib').glob('librustc_driver*'))]
    results = {}
    results['build'] = execute(out,'build',[TC/'bin/rustc',src,'--edition=2021','-o',out/'inspect-bin',
         '-L',str(TC/'lib'),'-C', 'rpath=yes'],inputs,env)
    need(results['build']['exit']==0, 'diagnostic inspector build failed')
    results['inspect'] = execute(out,'inspect',[out/'inspect-bin',native,'--crate-type=lib','--edition=2021',
        '--sysroot', TC],inputs+[out/'inspect-bin'],env)
    need(results['inspect']['exit']==0, 'diagnostic inspection failed')
    text = (out/'inspect/stdout').read_text()
    sections = text.split('BEGIN_INSTANCE ')[1:]
    inventory = []
    for section in sections:
        lines = section.splitlines()
        path = next(x[5:] for x in lines if x.startswith('PATH '))
        inventory.append(dict(instance=lines[0],path=path,body_fingerprint=next(x[17:] for x in lines if x.startswith('BODY_FINGERPRINT ')),whole_mir_sha256=hashlib.sha256(section.encode()).hexdigest(),
           whole_mir_bytes=len(section.encode()), calls=[x[5:] for x in lines if x.startswith('CALL ')],
           all_blocks=[x for x in lines if x.startswith('BLOCK ')],
           unavailable='MIR_UNAVAILABLE_OR_DEPTH_LIMIT' in section))
    # Whole debug MIR and per-block text are private. Compact inventory contains only names, counts and hashes.
    write(out/'inventory.json',inventory)
    write(out/'receipt.json',dict(results=results,inventory_sha256=sha(out/'inventory.json'),
        baseline=str(baseline),baseline_sha256=sha(baseline/'receipt.json'), publication=False,goal_complete=False))
    print(json.dumps([dict(path=x['path'],instance=x['instance'],calls=x['calls'],blocks=len(x['all_blocks']),unavailable=x['unavailable']) for x in inventory],indent=2))

if __name__ == '__main__':main()
