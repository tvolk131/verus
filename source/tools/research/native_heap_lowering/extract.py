#!/usr/bin/env python3
"""Actual structured native and local borrowck extraction, guarded and receipted."""
import argparse,json,os,sys
from pathlib import Path
sys.dont_write_bytecode=True
HERE=Path(__file__).resolve().parent; ROOT=HERE.parents[3]
sys.path.insert(0,str(ROOT/'source/tools'))
from infer_eval_support import ArtifactGuard,digest,support_artifacts
from infer_experiment import LoggedCommands,replay_receipt,require
p=argparse.ArgumentParser();p.add_argument('--output',type=Path,required=True);p.add_argument('--freeze',type=Path,required=True)
a=p.parse_args();a.output=a.output.resolve();a.output.mkdir(parents=True,exist_ok=False)
r=dict(schema=1,commands=[],native_bridge_proved=False,publication=False)
with replay_receipt(a.output,r):
 f=json.loads(a.freeze.read_text());require(f['before_first_trial'] is True,'pretrial freeze missing')
 paths=[ROOT/n for n in f['files']]
 for n,s in f['files'].items(): require(digest(ROOT/n)==s,'source freeze mismatch '+n)
 sysroot=Path('/home/dev/.rustup/toolchains/1.99.1-x86_64-unknown-linux-gnu')
 tools=[sysroot/'bin/rustc',sysroot/'bin/cargo',*sysroot.glob('lib/librustc_driver-*.so'),*sysroot.glob('lib/rustlib/x86_64-unknown-linux-gnu/lib/libcore-*.rlib'),*sysroot.glob('lib/rustlib/x86_64-unknown-linux-gnu/lib/librustc_*.rmeta')]
 sm=HERE.parent/'native_iterator_refinement/source-map.json'; native=sysroot/'lib/rustlib/src/rust'
 for n,s in json.loads(sm.read_text())['native_files'].items(): require(digest(native/n)==s,'native source changed');paths.append(native/n)
 g=ArtifactGuard([*paths,*tools,*support_artifacts(),sm,a.freeze]);r['artifacts']=g.initial
 env=dict(os.environ,PATH='/home/dev/.cargo/bin:'+os.environ['PATH'],RUSTUP_TOOLCHAIN='1.99.1',RUSTC_BOOTSTRAP='1',CARGO_BUILD_JOBS='2',CARGO_TARGET_DIR=str(a.output/'target'),LD_LIBRARY_PATH=str(sysroot/'lib'))
 run=LoggedCommands(a.output,ROOT/'source',r['commands'],timeout=240,guard=g)
 require(run('build',[str(sysroot/'bin/cargo'),'build','--offline','--locked','--manifest-path',str(HERE/'Cargo.toml'),'--config',f'build.build-dir="{a.output/"build"}"'],env)==0,'extractor build failure (not semantic)')
 bins=[a.output/'target/debug'/n for n in ['native-drop-extract','native-drop-borrowck']];bg=ArtifactGuard(bins);r['extractors']=bg.initial
 for src in sorted((HERE/'fixtures').glob('*.rs')):
  for binary,kind in zip(bins,['native','borrowck']):
   label=src.stem+'-'+kind;out=a.output/(label+'.json');bg.check()
   args=[str(out)]+(['256','100000'] if kind=='native' else [])+[str(src),'--crate-name','native_writer','--edition','2024','--sysroot',str(sysroot),'-Zmir-opt-level=0','-Coverflow-checks=on']
   code=run(label,[str(binary),*args],env)
   if src.stem=='invalid-alias':
    require(code!=0 and not out.exists() and 'E0499' in (a.output/(label+'.stderr')).read_text(),'invalid alias must fail lifetime')
   else:
    require(code==0,'extraction failed '+label);d=json.loads(out.read_text());require(d.get('input_sha256',d.get('source_sha256'))==digest(src),'source rehash mismatch')
   bg.check()
 g.check()
 print(json.dumps(dict(output=str(a.output),publication=False)))
