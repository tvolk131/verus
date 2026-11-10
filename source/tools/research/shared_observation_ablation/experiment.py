#!/usr/bin/env python3
"""Private, source-frozen observer ablations. Never changes production persistently."""
import argparse
import difflib
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import sys
import tempfile

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[3]
sys.path.insert(0, str(ROOT / 'source/tools'))
from infer_eval_support import ArtifactGuard, digest, tool_artifacts
from infer_experiment import (LoggedCommands, replay_receipt, require, validate_run,
    validate_goal_run, require_function_coverage, require_exact, require_publication)

BASE = 'd35c31fae9d05d7c0e797a60612a8be4716c2dc9'
OBSERVER = 'source/vir/src/infer/buffer_observation_goals.rs'
BUNDLE = Path('/tmp/verus-infer-buffer-cycle-20261109/parent-origin-fix-bundle-d35c31fa')
NAMES = ('verus', 'rust_verify', 'vstd.vir', 'libvstd.rlib', 'libverus_builtin.rlib',
         'libverus_builtin_macros.so', 'libverus_state_machines_macros.so', 'z3')
FIXTURES = {
    'canonical': 'source/tools/research/varint_canonical_goal/candidate-008.rs',
    'prefix': 'source/tools/research/buffer_observation_goal/prefix-zip.rs',
    'full': 'source/tools/research/buffer_observation_goal/full-zip.rs',
    'plain': 'source/tools/research/buffer_observation_goal/plain.rs',
}

def sha(b):
    return hashlib.sha256(b).hexdigest()

def git(*args):
    return subprocess.check_output(['git', *args], cwd=ROOT)

def environment():
    return dict(os.environ, PATH='/home/dev/.cargo/bin:' + os.environ['PATH'],
        RUSTUP_TOOLCHAIN='1.99.1', CARGO_BUILD_JOBS='2', RUST_MIN_STACK='33554432',
        VERUS_Z3_PATH=str(BUNDLE / 'z3'))

def altered(original, variant):
    if variant == 'observer-off':
        needle = '    let all = super::pipeline::function_map(fs, imported);'
        require(original.count(needle) == 1, 'ambiguous observer preparation hook')
        return original.replace(needle,
            '    // PRIVATE CAUSAL DIAGNOSTIC: suppress all observer proposals.\n'
            '    return Ok(fs.to_vec());\n' + needle)
    require(variant == 'capacity-off', 'unknown variant')
    needle = '        eq(s, c.len(s, seq.clone()), length),\n'
    require(original.count(needle) == 1, 'ambiguous capacity proposal')
    text = original.replace(needle,
        '        // PRIVATE CAUSAL DIAGNOSTIC: omit only iterator sequence capacity.\n')
    needle = 'if d.prefix.len() != 4 || d.invariants.len() != 4 {'
    require(text.count(needle) == 1, 'ambiguous exact projection inventory')
    # Adjust the independent exact insertion count, retaining its whole-function
    # projection/ID/purity checks and all normal full-body checks.
    return text.replace(needle, 'if d.prefix.len() != 4 || d.invariants.len() != 3 {')

def source_texts():
    texts = {k: (ROOT / v).read_text() for k, v in FIXTURES.items()}
    # Exact existing observation replay false-property and safety controls.
    texts['wrong_value'] = texts['prefix'].replace('old(ipad)@[j] ^ key@[j]',
                                                   'old(ipad)@[j] | key@[j]')
    texts['safety'] = '''use vstd::prelude::*; verus! {
                fn overflow(buf: &mut [u8;10]) { for b in buf.iter_mut() { *b += 1; } }
            } fn main() {}'''
    return texts

def validate_baseline(freeze):
    require(BUNDLE.is_dir() and not BUNDLE.is_symlink(), 'baseline bundle absent/link')
    manifest = json.loads((BUNDLE / 'copy-manifest.json').read_text())
    require(digest(BUNDLE / 'copy-manifest.json') == freeze['baseline_manifest_sha256'],
            'baseline copy manifest drift')
    require(set(manifest['files']) == set(NAMES), 'incomplete baseline bundle inventory')
    for name in NAMES:
        f = BUNDLE / name
        require(f.is_file() and not f.is_symlink(), 'missing/link bundle artifact: ' + name)
        require(digest(f) == manifest['files'][name]['sha256'] ==
                freeze['baseline_artifacts'][name], 'baseline artifact mismatch: ' + name)
    for p, h in manifest['engine_sources'].items():
        require(digest(ROOT / p) == h, 'baseline engine source mismatch: ' + p)

def check_freeze():
    f = json.loads((HERE / 'freeze.json').read_text())
    for p, h in f['controller_and_contracts'].items():
        require(digest(ROOT / p) == h, 'controller/contract drift: ' + p)
    for k, h in f['trial_source_sha256'].items():
        require(sha(source_texts()[k].encode()) == h, 'frozen application drift: ' + k)
    require(git('rev-parse', f'{BASE}^{{tree}}').decode().strip() == f['base_tree'],
            'baseline source tree mismatch')
    validate_baseline(f)
    return f

def freeze():
    require(git('rev-parse', 'HEAD').decode().strip() == BASE, 'freeze must start on base')
    require(not git('diff', '--name-only').strip(), 'tracked modifications before freeze')
    original = git('show', BASE + ':' + OBSERVER).decode()
    f = dict(schema=1, base=BASE, base_tree=git('rev-parse', 'HEAD^{tree}').decode().strip(),
        production_change_committed=False, goal_complete=False, publication=False,
        observer_source=OBSERVER, observer_base_sha256=sha(original.encode()),
        trial_source_sha256={k:sha(v.encode()) for k,v in source_texts().items()},
        controller_and_contracts={p:digest(ROOT / p) for p in (
            str(Path(__file__).relative_to(ROOT)), 'source/tools/infer_experiment.py',
            'source/tools/infer_eval_support.py',
            'source/tools/research/varint_canonical_goal/replay.py',
            'source/tools/research/buffer_observation_goal/replay.py',
            'source/tools/research/mutable_buffer_goal/TARGET.md')},
        baseline_manifest_sha256=digest(BUNDLE / 'copy-manifest.json'),
        baseline_artifacts={n:digest(BUNDLE / n) for n in NAMES}, variants={})
    for variant in ('observer-off', 'capacity-off'):
        modified = altered(original, variant)
        patch = ''.join(difflib.unified_diff(original.splitlines(True), modified.splitlines(True),
            fromfile='a/' + OBSERVER, tofile='b/' + OBSERVER))
        path = HERE / (variant + '.patch')
        path.write_text(patch)
        f['variants'][variant] = dict(altered_source_sha256=sha(modified.encode()),
            patch_sha256=digest(path), modified_files=[OBSERVER],
            hypothesis={'canonical':'ordinary proof failure',
                'prefix':'ordinary proof failure', 'full':'ordinary proof failure'},
            evidence_policy='Measure contradictions honestly; no frontend/resource/mode failures accepted')
    f['expected_baseline_enabled'] = dict(canonical=[6,0], prefix=[2,0], full=[1,1])
    f['expected_disabled'] = dict(canonical='ordinary proof failure, multiple kinds allowed',
                                  prefix=[1,1], full=[1,1])
    f['controls'] = dict(plain='success', wrong_value='postcondition not satisfied only',
                         safety='arithmetic underflow/overflow only')
    f['inventories'] = {k:dict(exec_bodies=[('u64' if k=='canonical' else 'plain'
         if k=='plain' else 'overflow' if k=='safety' else 'xor_key')],
         proof_bodies=[], inferred_helpers=[]) for k in source_texts()}
    f['trial_design'] = 'Each runtime x canonical/prefix/full x goals on/off x cold/warm; controls goals on. Same source path for cold/warm; private cache per runtime/source/goals; ordinary --export gate, no --compile; --no-cheating all applications.'
    f['limits'] = ['Full HMAC suffix remains required and unproved',
        'native correspondences 0; all 101 native effect obligations open',
        'fresh parent and independent acceptance required; publication false',
        'capacity-off production report hardcodes 4 invariants; actual insertion count is 3, patch-bound; selection report is never proof evidence']
    (HERE / 'freeze.json').write_text(json.dumps(f, indent=2) + '\n')
    validate_baseline(f)
    print(HERE / 'freeze.json')

def setup(out):
    out.mkdir(parents=True, exist_ok=False)
    receipt = dict(commands=[], kind='setup')
    with replay_receipt(out, receipt):
        f = check_freeze()
        guard = ArtifactGuard([BUNDLE / n for n in (*NAMES, 'copy-manifest.json')])
        receipt['baseline_artifacts'] = guard.initial
        call = LoggedCommands(out, ROOT, receipt['commands'], timeout=60,
                              guard=guard, status_key='returncode')
        for label, command, match in (
            ('verus-version', [str(BUNDLE/'verus'), '--version'], '0.2026.11.10.d35c31f'),
            ('z3-version', [str(BUNDLE/'z3'), '--version'], '4.16.0'),
            ('rust-version', ['rustc', '--version'], '1.99.1')):
            require(call(label, command, environment()) == 0, 'version command failed')
            require(match in (out/(label+'.stdout')).read_text(), 'unexpected '+label)
        receipt['freeze_sha256'] = digest(HERE / 'freeze.json')
        guard.check()

def build(out, variant):
    out.mkdir(parents=True, exist_ok=False)
    receipt = dict(commands=[], kind='build', variant=variant)
    with replay_receipt(out, receipt):
        f = check_freeze()
        original = git('show', BASE + ':' + OBSERVER)
        path = ROOT / OBSERVER
        require(path.read_bytes() == original, 'observer is not base')
        modified = altered(original.decode(), variant).encode()
        require(sha(modified) == f['variants'][variant]['altered_source_sha256'], 'patch drift')
        require(digest(HERE/(variant+'.patch')) == f['variants'][variant]['patch_sha256'], 'patch drift')
        guard = ArtifactGuard([BUNDLE / n for n in (*NAMES, 'copy-manifest.json')])
        # Complete build-source inventory: every tracked input under source/,
        # dependencies/, tools/, plus Rust toolchain. One known modified file.
        files = [p for p in git('ls-tree', '-r', '--name-only', BASE).decode().splitlines() if
            p.startswith(('source/', 'dependencies/', 'tools/')) or p=='rust-toolchain.toml']
        receipt['source_before_sha256'] = {p:digest(ROOT/p) for p in files}
        require(all(sha(git('show', BASE+':'+p)) == h
            for p,h in receipt['source_before_sha256'].items()), 'nonbase build source')
        try:
            path.write_bytes(modified)
            receipt['source_built_sha256'] = {p:digest(ROOT/p) for p in files}
            source_guard = ArtifactGuard([ROOT/p for p in files])
            call = LoggedCommands(out, ROOT/'source', receipt['commands'], timeout=1800,
                                  guard=source_guard, status_key='returncode')
            env = environment()
            # Standard release package build and standard vstd_build verifier.
            cmd = ['cargo', 'build', '--release', '-p', 'rust_verify', '-p', 'verus',
                '-p', 'verus_builtin', '-p', 'verus_builtin_macros',
                '-p', 'verus_state_machines_macros', '-p', 'vstd_build']
            require(call('release-build', cmd, env)==0, 'release build failed')
            target = ROOT / 'source/target-verus/release'
            sysroot = subprocess.check_output(['rustc', '--print', 'sysroot'], env=env, text=True).strip()
            env['LD_LIBRARY_PATH'] = sysroot + '/lib:' + env.get('LD_LIBRARY_PATH', '')
            env['VERUS_IN_VARGO'] = '1'
            require(call('matching-vstd', [str(target/'vstd_build'), str(target), '--release'], env)==0,
                    'matching vstd failed')
            require('verification results:: 2226 verified, 0 errors' in (out/'matching-vstd.stdout').read_text(),
                    'matching vstd count changed')
            source_guard.check()
            bundle = out / 'bundle'
            bundle.mkdir(mode=0o700)
            receipt['bundle'] = dict(files={}, engine_sources={p:digest(ROOT/p)
                for p in json.loads((BUNDLE/'copy-manifest.json').read_text())['engine_sources']})
            for n in NAMES:
                src = BUNDLE/'z3' if n=='z3' else target/n
                dest = bundle/n
                require(not src.is_symlink(), 'build artifact symlink')
                shutil.copy2(src, dest)
                dest.chmod(0o555 if n in ('verus','rust_verify','z3') else 0o444)
                require(digest(src)==digest(dest) and src.stat().st_ino!=dest.stat().st_ino,
                        'nonindependent artifact copy')
                receipt['bundle']['files'][n] = dict(source=str(src), sha256=digest(dest))
            (bundle/'copy-manifest.json').write_text(json.dumps(receipt['bundle'], indent=2)+'\n')
            bguard = ArtifactGuard([bundle/n for n in (*NAMES,'copy-manifest.json')])
            receipt['bundle_guard'] = bguard.initial
            version = LoggedCommands(out, ROOT, receipt['commands'], timeout=60,
                                     guard=bguard, status_key='returncode')
            require(version('built-version', [str(bundle/'verus'),'--version'], env)==0,
                    'private bundle version failed')
            require('Profile: release' in (out/'built-version.stdout').read_text(), 'not release')
        finally:
            # Only restore the single file whose bytes this invocation owns.
            path.write_bytes(original)
            receipt['source_after_sha256'] = {p:digest(ROOT/p) for p in files}
            receipt['production_restored'] = receipt['source_before_sha256']==receipt['source_after_sha256']
            require(receipt['production_restored'], 'production restoration mismatch')
            guard.check()
        for label in ('release-build','matching-vstd','built-version'):
            for suffix in ('stdout','stderr'):
                receipt.setdefault('log_sha256',{})[label+'.'+suffix]=digest(out/(label+'.'+suffix))

def diagnostic(run, expected, execs):
    """Multi-obligation causal diagnostic: intentionally NOT strict semantic control."""
    if expected:
        return validate_goal_run(run, True, expected_functions=set(),
                                 exec_bodies=set(execs), proof_bodies=set())
    data = validate_run(run, False)
    v = data['verification-results']
    require(v.get('is-verifying-entire-crate') is True and v.get('errors',0)>0,
            'causal diagnostic partial/empty')
    require(not v.get('encountered-vir-error'), 'frontend failure is invalid evidence')
    stderr = run['stderr']
    require(not re.search(r'error\[E\d+\]', stderr), 'Rust frontend failure')
    require(any(x in stderr for x in ('postcondition not satisfied','precondition not satisfied',
        'assertion failed','invariant not satisfied','possible arithmetic underflow/overflow')),
        'no ordinary proof diagnostic')
    # encountered-error is also true for ordinary SMT failures. Accept only
    # explicit ordinary-proof diagnostic headers (plus rustc's abort summary),
    # not that ambiguous aggregate flag, to exclude mixed frontend failures.
    allowed = ('postcondition not satisfied','precondition not satisfied',
        'assertion failed','invariant not satisfied','loop invariant not satisfied',
        'possible arithmetic underflow/overflow','aborting due to ')
    require(all(any(line.startswith('error: '+s) for s in allowed)
        for line in stderr.splitlines() if line.startswith('error: ')),
        'mixed frontend or unknown diagnostic header')
    inf = data['verus-infer']
    require(inf.get('crate_success') is False, 'crate status mismatch')
    require_function_coverage(inf.get('functions',[]),set())
    require_exact(inf.get('functions',[]))
    scope = inf['verification_scope']
    require(scope.get('native_collection_completed') is True and
            scope.get('complete_selected_scope_verified') is False, 'bad scope status')
    selected = [b for b in scope['native_functions'] if b.get('selected') is True]
    proxies = [b for b in selected if b.get('body_status')=='selected_item_without_translated_body']
    # The frozen canonical const creates this exact compiler proxy. It is not
    # an authored executable/proof function; do not admit arbitrary unknown items.
    require_function_coverage(proxies, {'VERUS_UNERASED_PROXY__U64_LEN'} if set(execs)=={'u64'} else set())
    bodies = [b for b in selected if b.get('body_status') not in
        ('specification_definition_or_declaration','selected_item_without_translated_body')]
    require(all(b.get('body_status')=='selected_body_not_certified_by_successful_full_run'
        for b in bodies), 'selected body bypass')
    require_function_coverage(bodies,set(execs))
    return data

def trials(out, variant, bundle):
    out.mkdir(parents=True, exist_ok=False)
    receipt = dict(commands=[], runs=[], kind='trials', variant=variant,
        goal_complete=False, publication=False, native_correspondences=0,
        native_effect_obligations_open=101)
    with replay_receipt(out, receipt):
        f = check_freeze()
        if variant=='baseline':
            require(bundle==BUNDLE,'baseline bundle must be delivered immutable copy')
        else:
            b = json.loads((bundle.parent/'receipt.json').read_text())
            require(b['success'] and b['production_restored'] and b['variant']==variant,
                    'unvalidated build')
            require(b['source_built_sha256'][OBSERVER]==f['variants'][variant]['altered_source_sha256'],
                    'build-source identity mismatch')
            for n,h in b['bundle']['files'].items():
                require(digest(bundle/n)==h['sha256'],'built artifact drift')
            receipt['build_receipt_sha256']=digest(bundle.parent/'receipt.json')
        guard = ArtifactGuard(tool_artifacts(bundle/'verus',bundle/'z3',extra=[
            HERE/'freeze.json', Path(__file__), BUNDLE/'copy-manifest.json',
            *[ROOT/p for p in FIXTURES.values()], *[BUNDLE/n for n in NAMES]]))
        receipt['artifacts']=guard.initial
        receipt['freeze_sha256']=digest(HERE/'freeze.json')
        call = LoggedCommands(out, out, receipt['commands'], timeout=240,
                              guard=guard,status_key='returncode')
        texts = source_texts()
        env = environment()
        env['VERUS_Z3_PATH']=str(bundle/'z3')
        for case in texts:
            (out/(case+'.rs')).write_text(texts[case])
        def run(case, goals, phase):
            label=f'{case}_{"on" if goals else "off"}_{phase}'
            source=out/(case+'.rs')
            require(digest(source)==f['trial_source_sha256'][case], 'application drift')
            cache=out/f'cache-{case}-{goals}'
            if phase=='cold':
                require(not cache.exists(),'cold cache already present')
            env['VERUS_INFER_CACHE']=str(cache)
            execs=f['inventories'][case]['exec_bodies']
            trial=dict(label=label,case=case,goals=goals,phase=phase,
                source_sha256=digest(source), **f['inventories'][case],
                cache_files_before={p.name:digest(p) for p in cache.glob('**/*') if p.is_file()})
            receipt['runs'].append(trial)
            (out/(label+'.freeze.json')).write_text(json.dumps(trial,indent=2)+'\n')
            artifact=out/(label+'.vir')
            # --export exercises publication gating; successes may publish only
            # checked ordinary interfaces, and failed diagnostics must remove it.
            cmd=[str(bundle/'verus'),str(source),'--crate-type=lib','--no-cheating',
                '--output-json','--multiple-errors','20','--export',str(artifact)]
            if goals: cmd.append('--infer-goals')
            sg=ArtifactGuard([source])
            rc=call(label,cmd,env)
            sg.check()
            result=dict(returncode=rc,stdout=(out/(label+'.stdout')).read_text(),
                        stderr=(out/(label+'.stderr')).read_text())
            observed=json.loads(result['stdout'])['verification-results'].get('success') is True
            if case in ('plain','wrong_value'):
                data=validate_goal_run(result,case=='plain',expected_functions=set(),
                    exec_bodies=set(execs),proof_bodies=set(),
                    failure_kind=None if case=='plain' else 'postcondition not satisfied',artifact=artifact)
            else:
                data=diagnostic(result,observed,execs)
                require_publication(data,observed,artifact)
            v=data['verification-results']
            if variant=='baseline' and goals and case in f['expected_baseline_enabled']:
                require([v['verified'],v['errors']]==f['expected_baseline_enabled'][case],
                        'baseline expectation failed: '+label)
            if not goals and case in ('prefix','full'):
                require([v['verified'],v['errors']]==[1,1],'disabled expectation failed')
            if not goals and case=='canonical':
                require(not observed,'canonical disabled unexpectedly proved')
            if case=='safety':
                require(not observed and 'possible arithmetic underflow/overflow' in result['stderr'],
                        'safety check lost')
                require(not any(x in result['stderr'] for x in ('postcondition not satisfied',
                    'invariant not satisfied','assertion failed','precondition not satisfied')),
                    'mixed safety control')
            inf=data['verus-infer']
            require(inf.get('generation_failure') is None and
                inf.get('requested_functions')==0 and inf.get('generated_functions')==0 and
                inf.get('status') in ('no_inference','caller_proof_failed','verification_failed'),
                'unhealthy inference mandatory/generation statuses')
            prep=inf['assistance_explanations']
            require(prep.get('preparation_started') is goals and
                prep.get('preparation_completed') is goals, 'assistance preparation incomplete')
            decisions=inf.get('assistance_explanations',{}).get('decisions',[])
            numeric=any(d.get('profile')=='mutable_buffer_u64_radix128' and
                d.get('status')=='applied' for d in decisions)
            require(numeric==(case=='canonical' and goals),'numeric assistance not retained')
            observers=inf.get('buffer_observation_assistance',[])
            require({e['function'].rsplit('::',1)[-1] for e in observers} ==
                (set(execs) if goals and variant!='observer-off' else set()),
                'observer selection mismatch')
            trial.update(verification=v,returncode=rc,numeric_applied=numeric,
                observation_report=observers,verification_scope=inf['verification_scope'],
                functions=inf.get('functions',[]),
                diagnostic_counts={s:len(re.findall(r'(?m)^error: '+re.escape(s),result['stderr']))
                    for s in ('postcondition not satisfied','precondition not satisfied','assertion failed',
                        'invariant not satisfied','loop invariant not satisfied',
                        'possible arithmetic underflow/overflow')},
                stderr=result['stderr'], stdout_sha256=digest(out/(label+'.stdout')),
                stderr_sha256=digest(out/(label+'.stderr')),
                interface_exists=artifact.exists(),cache_files_after={p.name:digest(p)
                    for p in cache.glob('**/*') if p.is_file()})
            print(variant,label,v['verified'],v['errors'],flush=True)
        for case in ('canonical','prefix','full'):
            for goals in (True,False):
                for phase in ('cold','warm'): run(case,goals,phase)
        for case in ('plain','wrong_value','safety'): run(case,True,'cold')
        guard.check()


def main():
    p=argparse.ArgumentParser()
    p.add_argument('action',choices=['freeze','setup','build','trials'])
    p.add_argument('--output',type=Path)
    p.add_argument('--variant',choices=['baseline','observer-off','capacity-off'],default='baseline')
    p.add_argument('--bundle',type=Path,default=BUNDLE)
    a=p.parse_args()
    if a.action=='freeze': freeze()
    elif a.action=='setup': setup(a.output.resolve())
    elif a.action=='build': build(a.output.resolve(),a.variant)
    else: trials(a.output.resolve(),a.variant,a.bundle.resolve())

if __name__=='__main__': main()
