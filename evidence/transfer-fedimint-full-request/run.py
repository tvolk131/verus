"""Source-connected full seven-field Fedimint replay; outputs never enter the app."""
import argparse
import json
import os
from pathlib import Path
import shutil
import sys
import tempfile
import tomllib

ROOT = Path(__file__).resolve().parents[2]
HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(ROOT / 'source/tools'))
from infer_eval_support import ArtifactGuard, digest, tool_artifacts
from infer_experiment import (LoggedCommands, require, require_exact,
                              require_publication, validate_run, replay_receipt)
import check_fedimint_native as native

OPAQUE = '''
// Supplied external type declaration: identity only, no digest semantics.
#[verifier::external_type_specification]
#[verifier::external_body]
pub struct ExternalSha256(bitcoin::hashes::sha256::Hash);
'''


def source(implementation, *, opaque=True, infer=True, goals=True):
    original = (HERE / 'original.rs').read_text()
    struct = native.item(original, 'pub struct InterceptPaymentRequest {')
    if infer:
        implementation = implementation.replace('    pub fn ', '    #[verifier::infer]\n    pub fn ')
    return ('use vstd::prelude::*;\nuse bitcoin::hashes::sha256;\nverus! {\n'
            + (OPAQUE if opaque else '') + native.CONSTANT + '\n' + struct + '\n'
            + implementation + ((HERE / 'goals.rs').read_text() if goals else '') + '\n}\n')


def check_source(application):
    frozen = json.loads((HERE / 'baseline.json').read_text())
    require({p: digest(application / p) for p in native.SOURCE_HASHES} == native.SOURCE_HASHES)
    text = (application / native.LIGHTNING).read_text()
    constant, implementation = native.extract(text)
    struct = native.item(text, 'pub struct InterceptPaymentRequest {')
    require(native.text_digest(struct) == frozen['struct_sha256'])
    require((HERE / 'original.rs').read_text() == 'use bitcoin::hashes::sha256;\n'
            + constant + '\n' + struct + '\n' + implementation + '\n')
    require(digest(HERE / 'goals.rs') == frozen['goals_sha256'])
    require('const LNV2_CLAIM_DEADLINE_MARGIN: u32 = 2;' in (application / native.CLIENT).read_text())
    return implementation


def check_dependencies(application, lock):
    packages = tomllib.loads(lock.read_text())['package']
    app = tomllib.loads((application / 'Cargo.lock').read_text())['package']
    pins = {(p['name'], p['version'], p.get('checksum')) for p in app}
    selected = [p for p in packages if 'source' in p]
    require(all((p['name'], p['version'], p.get('checksum')) in pins for p in selected),
            'selected registry dependency differs from application lock')
    require(any(p['name'] == 'bitcoin' and p['version'] == '0.32.8' for p in selected))
    require(any(p['name'] == 'bitcoin_hashes' and p['version'] == '0.14.0' for p in selected))
    return selected


def replay(args, out, receipt):
    application, verus, solver = args.application.resolve(), args.verus.resolve(), args.solver.resolve()
    env = dict(os.environ, PATH='/home/dev/.cargo/bin:' + os.environ['PATH'],
               CARGO_BUILD_JOBS='2', RUSTUP_TOOLCHAIN='1.99.1',
               VERUS_Z3_PATH=str(solver), RUST_MIN_STACK='20971520',
               CARGO_TARGET_DIR=str(out / 'native-target'))
    command = LoggedCommands(out, out, receipt['commands'], timeout=args.timeout)

    def invoke(label, argv, expected=0):
        rc = command(label, list(map(str, argv)), env)
        require(rc == expected, 'unexpected command outcome: ' + label)
        return (out / (label + '.stdout')).read_text().strip()

    require(invoke('app_head', ['git', '-C', application, 'rev-parse', 'HEAD']) == native.APPLICATION)
    require(invoke('app_clean', ['git', '-C', application, 'status', '--porcelain']) == '')
    implementation = check_source(application)
    receipt['engine_head'] = invoke('engine_head', ['git', '-C', ROOT, 'rev-parse', 'HEAD'])
    files = invoke('engine_files', ['git', '-C', ROOT, 'ls-files', 'source', 'rust-toolchain.toml']).splitlines()
    receipt['engine_source_sha256'] = {f: digest(ROOT / f) for f in files if (ROOT / f).is_file()}
    invoke('engine_patch', ['git', '-C', ROOT, 'diff', 'HEAD', '--', 'source'])
    receipt['engine_patch_sha256'] = digest(out / 'engine_patch.stdout')
    receipt['inventory_sha256'] = digest(HERE / 'baseline.json')
    receipt['research_source_sha256'] = {str(p.relative_to(HERE)): digest(p) for p in HERE.rglob('*')
        if p.is_file() and p.suffix in ('.py', '.rs', '.toml', '.lock')}
    receipt['source_sha256'] = native.SOURCE_HASHES
    receipt['dependencies'] = check_dependencies(application, HERE / 'native/Cargo.lock')
    project = out / 'native'
    shutil.copytree(HERE / 'native', project)
    original = (HERE / 'original.rs').read_text()
    original_derived = original.replace('pub struct InterceptPaymentRequest {',
        '#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]\npub struct InterceptPaymentRequest {')
    (project / 'src').mkdir(exist_ok=True)
    (project / 'src/lib.rs').write_text(original_derived)
    (out / 'original.rs').write_text(original_derived)
    (out / 'native_regression.rs').write_text(original_derived.replace(native.GUARD, '', 1))
    invoke('native_build', ['cargo', 'build', '--offline', '--locked', '--manifest-path', project / 'Cargo.toml'])
    deps = out / 'native-target/debug/deps'
    bitcoin, = list(deps.glob('libbitcoin-*.rlib'))
    extern = ['--extern', 'bitcoin=' + str(bitcoin), '-L', 'dependency=' + str(deps)]
    receipt['native_compiler'] = invoke('rustc_version', ['rustc', '-vV'])
    receipt['native_compilation'] = 'full struct with Debug/Serialize/Deserialize/Clone and unchanged helper bodies'
    (out / 'oracle.rs').write_bytes((HERE / 'oracle.rs').read_bytes())
    sources = {
        'bare': source(implementation, opaque=False),
        'translated': source(implementation, infer=False, goals=False),
        'full': source(implementation),
        'regression': source(native.regress(implementation)),
        'wrong_type': source(implementation).replace('ExternalSha256(bitcoin::hashes::sha256::Hash)',
            'ExternalSha256(bitcoin::hashes::sha256d::Hash)'),
        'false_goal': source(implementation).replace('blocks == 0,', 'blocks == 1,', 1),
    }
    for name, text in sources.items():
        (out / (name + '.rs')).write_text(text)
    receipt['generated_sha256'] = {p.name: digest(p) for p in out.glob('*.rs')}
    compiler = Path(invoke('rustc_path', ['rustup', 'which', '--toolchain', '1.99.1', 'rustc']))
    guard = ArtifactGuard(tool_artifacts(verus, solver, extra=[compiler, *compiler.parent.parent.glob('lib/librustc_driver*.so'), bitcoin, *deps.glob('*.rlib'),
        *deps.glob('*.so'), *out.glob('*.rs'), HERE / 'run.py', HERE / 'baseline.json',
        HERE / 'goals.rs', *(ROOT / f for f in receipt['engine_source_sha256']),
        *(application / f for f in native.SOURCE_HASHES)]))
    command.guard = guard
    receipt['artifacts_before'] = guard.initial
    invoke('verus_version', [verus, '--version'])
    invoke('solver_version', [solver, '-version'])

    def verify(name, phase, expected, *, frontend=False, imported=False):
        label = name + '_' + phase
        artifact = out / (name + '.vir')
        previous = None
        if not expected and (out / 'full.vir').exists():
            artifact.write_bytes((out / 'full.vir').read_bytes())
        if artifact.exists():
            previous = dict(sha256=digest(artifact), mtime_ns=artifact.stat().st_mtime_ns)
        env['VERUS_INFER_CACHE'] = str(out / (name + '-cache'))
        flags = [verus, out / (name + '.rs'), '--crate-type=lib', '--edition=2024',
                 '--crate-name', 'deadline_full' if name == 'full' else name,
                 '--no-cheating', '--output-json', '--num-threads', '1', '--export', artifact, *extern]
        if expected:
            flags += ['--compile', '--out-dir', out]
        if imported:
            flags += ['--extern', 'deadline_full=' + str(out / 'libdeadline_full.rlib'),
                      '--import', 'deadline_full=' + str(out / 'full.vir'), '-L', str(verus.parent)]
        rc = command(label, list(map(str, flags)), env)
        run = dict(returncode=rc, stdout=(out / (label + '.stdout')).read_text(),
                   stderr=(out / (label + '.stderr')).read_text())
        data = validate_run(run, expected, allow_frontend=frontend)
        if not frontend:
            require(data['verification-results'].get('is-verifying-entire-crate') is True)
            if name == 'translated':
                require(data['verus-infer']['status'] == 'no_inference' and artifact.exists())
                require(data['verus-infer']['verification_scope']['complete_selected_scope_verified'] is True)
            else:
                require_publication(data, expected, artifact)
            if expected and previous is not None:
                require(artifact.stat().st_mtime_ns != previous['mtime_ns'], 'interface was not freshly published')
        receipt['cases'].append(dict(name=name, phase=phase, previous_interface=previous,
            interface_exists=artifact.exists(), inference=data.get('verus-infer'),
            verification=data['verification-results'], diagnostic=run['stderr']))
        return data, run

    data, run = verify('bare', 'frontend', False, frontend=True)
    require('bitcoin_hashes::sha256::Hash' in run['stderr'] and 'external_type_specification' in run['stderr'])
    verify('translated', 'frontend', True)
    if args.probe:
        data, run = verify('full', 'eligibility', False, frontend=True)
        require('record state requires' in run['stderr'])
        guard.check()
        require(invoke('app_clean_after', ['git', '-C', application, 'status', '--porcelain']) == '')
        receipt['stage'] = 'unchanged-engine bare-type blocker plus opaque translation and inference eligibility probe'
        return
    contracts = None
    for phase in ['cold', 'warm']:
        data, run = verify('full', phase, True)
        fs = data['verus-infer']['functions']
        require(len(fs) == 2 and {f['function'].split('::')[-1] for f in fs} ==
                {'incoming_circuit', 'lnv2_blocks_to_claim_deadline'})
        require_exact(fs)
        require(all(f['contract']['requires'] == 'true' and f['exported'] for f in fs))
        require(all(f['metrics']['cache'] == 'miss' and f['metrics'].get('cache_policy') == 'regenerated_state' for f in fs))
        observed = {f['function']: f['contract'] for f in fs}
        if contracts is not None:
            require(observed == contracts)
        contracts = observed
        scope = data['verus-infer']['verification_scope']
        require(scope['complete_selected_scope_verified'] is True and scope['checked_exec_bodies'] == 5)
    data, wrong = verify('wrong_type', 'identity', False, frontend=True)
    require('bitcoin_hashes::sha256::Hash' in wrong['stderr'] and 'not supported' in wrong['stderr'])
    require(not (out / 'wrong_type.vir').exists())
    for name in ['regression', 'false_goal']:
        data, run = verify(name, 'semantic', False)
        fs = data['verus-infer']['functions']
        require(len(fs) == 2 and {f['function'].split('::')[-1] for f in fs} ==
                {'incoming_circuit', 'lnv2_blocks_to_claim_deadline'})
        require_exact(fs)
        require(all(not f['exported'] and f['contract']['requires'] == 'true' for f in fs))
        require(data['verification-results']['errors'] == 1)
        require('postcondition not satisfied' in run['stderr'] and 'precondition not satisfied' not in run['stderr'])
        require(any('deadline_policy' in f for f in data['verus-infer']['failed_functions']))
    (out / 'consumer.rs').write_text('use vstd::prelude::*;\nuse deadline_full::InterceptPaymentRequest;\nverus! {\n'
        + (HERE / 'goals.rs').read_text() + '\n}\n')
    for phase in ['cold', 'warm']:
        data, run = verify('consumer', phase, True, imported=True)
        scope = data['verus-infer']['verification_scope']
        require(scope['complete_selected_scope_verified'] is True and scope['checked_exec_bodies'] == 3)
    # Execute the Verus-erased full request against independently compiled original source.
    invoke('oracle_build', ['rustc', out / 'oracle.rs', '--edition=2024', '-C', 'overflow-checks=yes',
        '--extern', 'deadline_full=' + str(out / 'libdeadline_full.rlib'),
        '--extern', 'serde=' + str(next(deps.glob('libserde-*.rlib'))),
        *extern, '-L', str(verus.parent), '-o', out / 'oracle'])
    receipt['native_erasure_oracle'] = json.loads(invoke('oracle', [out / 'oracle']))
    guard.check()
    require(invoke('app_clean_after', ['git', '-C', application, 'status', '--porcelain']) == '')
    receipt['generated_sha256'] = {p.name: digest(p) for p in out.glob('*.rs')}
    receipt['stage'] = 'full seven-field selected-scope proof and controls'


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--application', type=Path, required=True)
    parser.add_argument('--verus', type=Path, required=True)
    parser.add_argument('--solver', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--timeout', type=float, default=180)
    parser.add_argument('--probe', action='store_true')
    args = parser.parse_args()
    args.output.mkdir(parents=True, exist_ok=True)
    out = Path(tempfile.mkdtemp(prefix='full-request-', dir=args.output)).resolve()
    receipt = dict(commands=[], cases=[])
    try:
        with replay_receipt(out, receipt):
            replay(args, out, receipt)
    finally:
        print(out)


if __name__ == '__main__':
    main()
