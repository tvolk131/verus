#!/usr/bin/env python3
"""One bounded native expansion + authenticated private copies; zero Verus trials."""
import json, os, shutil, sys, tarfile, tomllib
from pathlib import Path
HERE=Path(__file__).resolve().parent
ROOT=HERE.parents[4]
sys.path.insert(0,str(ROOT/'source/tools'))
from infer_experiment import LoggedCommands, replay_receipt, require
from infer_eval_support import ArtifactGuard, digest
OUT=Path('/tmp/fedimint-bootstrap-worker-3ad8038e')
TC=Path('/home/dev/.rustup/toolchains/1.99.1-x86_64-unknown-linux-gnu')
ORIGIN=Path('/tmp/practical-bootstrap-types-feature2')
BUNDLE=Path('/tmp/practical-combined78-5ecea5fe-core/bundle')
VERSIONS={'derive_more':'1.0.0','derive_more-impl':'1.0.0','proc-macro2':'1.0.107','quote':'1.0.47','syn':'2.0.104','unicode-ident':'1.0.18'}

def main():
    OUT.mkdir(exist_ok=False)
    r=dict(kind='F3C_PRIVATE_CONTEXT_AND_NATIVE_EXPANSION_NOT_PROOF',commands=[],crypto_proved=False)
    with replay_receipt(OUT,r):
        freeze=json.loads((HERE/'PREPARATION.json').read_text())
        for p,h in freeze['inputs'].items():require(digest(Path(p))==h,'preparation input '+p)
        ctx=json.loads((ORIGIN/'receipt.json').read_text())
        require(ctx['success'] and ctx['native_context_only'] and not ctx['crypto_proved'],'native identity')
        # Every source/archive/tool named by the authenticated receipt is immutable input.
        for group in ['inputs','archives','artifacts']:
            for p,h in ctx[group].items():require(digest(Path(p))==h,'native input '+p)
        (OUT/'context/libs').mkdir(parents=True)
        shutil.copyfile(ORIGIN/'receipt.json',OUT/'context/original-receipt.json')
        copied={}
        for p,h in ctx['artifacts'].items():
            dst=OUT/'context/libs'/Path(p).name;shutil.copyfile(p,dst);require(digest(dst)==h,'private lib');copied[str(dst)]=h
        require(digest(BUNDLE/'receipt.json')==freeze['runtime_receipt_sha256'],'runtime receipt')
        runtime=json.loads((BUNDLE/'receipt.json').read_text())
        require(runtime['success'] and runtime['source_commit']=='5ecea5fe81c2bc2ef0c9050cdb29cea0c7ef21c4' and len(runtime['tools'])==8,'accepted runtime')
        for n,h in runtime['tools'].items():require(digest(BUNDLE/n)==h,'runtime '+n)
        shutil.copytree(BUNDLE,OUT/'bundle')
        require((OUT/'bundle/verus-root').is_file(),'runtime marker')
        for n,h in runtime['tools'].items():require(digest(OUT/'bundle'/n)==h,'copied runtime '+n)
        lock=tomllib.loads(Path(freeze['lock']).read_text());cache=Path('/home/dev/.cargo/registry/cache/index.crates.io-1949cf8c6b5b557f')
        packages={};archives={};original_manifests={};adapted={}
        active={'derive_more':['std','from','into'],'derive_more-impl':['from','into'],'proc-macro2':['proc-macro'],'quote':['proc-macro'],'syn':['derive','parsing','printing','clone-impls','proc-macro','extra-traits'],'unicode-ident':[]}
        edges={'derive_more':['derive_more-impl'],'derive_more-impl':['proc-macro2','quote','syn'],'proc-macro2':['unicode-ident'],'quote':['proc-macro2'],'syn':['proc-macro2','quote','unicode-ident'],'unicode-ident':[]}
        for name,version in VERSIONS.items():
            rec=next(p for p in lock['package'] if p['name']==name and p['version']==version)
            archive=cache/(name+'-'+version+'.crate');require(digest(archive)==rec['checksum'],'macro archive '+name);archives[str(archive)]=rec['checksum']
            with tarfile.open(archive) as t:
                for m in t.getmembers():
                    if not m.isfile():continue
                    rel=Path(m.name);require(not rel.is_absolute() and '..' not in rel.parts,'archive path')
                    dst=OUT/'expansion/sources'/rel;dst.parent.mkdir(parents=True,exist_ok=True);dst.write_bytes(t.extractfile(m).read())
            folder=OUT/'expansion/sources'/(name+'-'+version);packages[name]=folder
            meta=tomllib.loads((folder/'Cargo.toml').read_text());original_manifests[name]=digest(folder/'Cargo.toml')
            require(set(active[name])<=meta.get('features',{}).keys(),'actual selected features')
        # Cargo must not resolve unrelated optional/dev dependency graphs. Keep published Rust
        # source/build scripts/lib metadata; project each manifest onto inspected active features.
        for name,folder in packages.items():
            path=folder/'Cargo.toml';meta=tomllib.loads(path.read_text());shutil.copyfile(path,folder/'Cargo.original.toml')
            pkg=meta['package'];lines=['[package]',f'name = {json.dumps(name)}',f'version = {json.dumps(pkg["version"])}',f'edition = {json.dumps(pkg.get("edition","2015"))}']
            if 'rust-version' in pkg:lines+=['rust-version = '+json.dumps(pkg['rust-version'])]
            if (folder/'build.rs').exists():lines+=['build = "build.rs"']
            lines+=['[lib]']
            for k in ['name','path','proc-macro']:
                if k in meta.get('lib',{}):lines+=[k+' = '+json.dumps(meta['lib'][k])]
            lines+=['[features]','default = []']
            for feature in active[name]:lines+=[json.dumps(feature)+' = []']
            for dep in edges[name]:
                spec=meta['dependencies'][dep]
                lines+=['[dependencies.'+json.dumps(dep)+']','path = '+json.dumps(str(packages[dep])),'default-features = false','features = '+json.dumps(active[dep])]
                require(dep in meta['dependencies'],'genuine dependency')
                require(not spec.get('optional',False) or dep=='quote','unexpected active optional dep')
            path.write_text('\n'.join(lines)+'\n');adapted[name]=digest(path)
        project=OUT/'expansion/project';(project/'src').mkdir(parents=True)
        (project/'Cargo.toml').write_text('[package]\nname="nodecount-expansion"\nversion="0.0.0"\nedition="2024"\n[dependencies]\nderive_more = { path = '+json.dumps(str(packages['derive_more']))+', default-features=false, features=["std","from","into"] }\n')
        (project/'src/lib.rs').write_text('#[derive(derive_more::From, derive_more::Into)]\npub struct NodeCount(pub usize);\n')
        home=OUT/'expansion/cargo-home';(home/'.').mkdir(parents=True)
        (home/'config.toml').write_text('[net]\noffline=true\n[build]\nbuild-dir='+json.dumps(str(OUT/'expansion/build-dir'))+'\n')
        env=dict(os.environ,PATH=str(TC/'bin')+':/home/dev/.cargo/bin:/usr/bin:/bin',RUSTUP_HOME='/home/dev/.rustup',RUSTUP_TOOLCHAIN=str(TC),LD_LIBRARY_PATH=str(TC/'lib'),CARGO_HOME=str(home),CARGO_TARGET_DIR=str(OUT/'expansion/target'))
        guarded=[p for folder in packages.values() for p in folder.rglob('*') if p.is_file()]+[project/'src/lib.rs',project/'Cargo.toml',home/'config.toml',HERE/'PREPARATION.json',Path(__file__),TC/'bin/rustc']
        guard=ArtifactGuard(guarded);run=LoggedCommands(OUT,project,r['commands'],timeout=180,guard=guard)
        code=run('expand',[str(TC/'bin/cargo'),'rustc','--offline','--lib','--','-Zunpretty=expanded'],env)
        require(code==0,'bounded expansion preparation failed; do not expand dependencies')
        shutil.copyfile(OUT/'expand.stdout',OUT/'expansion/expanded.rs')
        r.update(native_context_only=True,original_context_receipt_sha256=digest(ORIGIN/'receipt.json'),private_artifacts=copied,runtime=runtime,archives=archives,original_manifests=original_manifests,adapted_manifests=adapted,expanded_sha256=digest(OUT/'expansion/expanded.rs'),source_input_sha256=digest(project/'src/lib.rs'),guarded_inputs={str(p):digest(p) for p in guarded})
    print(OUT/'receipt.json')
if __name__=='__main__':main()
