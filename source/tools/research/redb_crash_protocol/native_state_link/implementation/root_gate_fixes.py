#!/usr/bin/env python3
"""Bounded root corrections; reuse existing guarded offline execution helpers."""
import argparse
import datetime
import json
import os
import shutil
import subprocess
import sys
from pathlib import Path
sys.dont_write_bytecode = True
import check
from infer_eval_support import ArtifactGuard, digest
HERE = Path(__file__).resolve().parent
SNAPSHOT = Path('/tmp/owned-cell-F1-skip-fix-9cc0f65c-parent-snapshot')
RECEIPT_SHA = '9b414f0493385c649d856fd181625ec74778173e10623ee6ba3c3fda959b945f'
OVERRIDES = {'CARGO_INCREMENTAL': '0', 'CARGO_PROFILE_DEV_DEBUG': '0', 'CARGO_PROFILE_TEST_DEBUG': '0', 'PYTHONDONTWRITEBYTECODE': '1'}

def main():
    p = argparse.ArgumentParser()
    p.add_argument('--output', type=Path, required=True)
    a = p.parse_args()
    out = a.output.resolve()
    started = datetime.datetime.now(datetime.timezone.utc).isoformat()
    check.require(shutil.disk_usage('/tmp').free > 25 * 1024**3, 'insufficient disk for a fresh build')
    receipt = SNAPSHOT / 'receipt.json'
    check.require(digest(receipt) == RECEIPT_SHA, 'snapshot receipt identity')
    old = json.loads(receipt.read_text())['tools_sha256']
    historical = ArtifactGuard([receipt, *[SNAPSHOT / n for n in old]])
    for n, h in old.items():
        check.require(digest(SNAPSHOT / n) == h, 'frozen tool ' + n)
    check.setup(out)
    for n in [*old, 'verus-root']:
        dst = out / 'bundle' / n
        if dst.exists():
            check.require(digest(dst) == digest(SNAPSHOT / n), 'existing setup artifact ' + n)
        else:
            shutil.copy2(SNAPSHOT / n, dst)
    source = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=check.ROOT, text=True).strip()
    extras = [Path(__file__), HERE / 'ROOT_GATE_FIXES.md', check.ROOT / 'source/rust_verify_test/tests/local_checked_owned_cell_admission.rs', *sorted((check.ROOT / 'source/vir/src').rglob('*.rs'))]
    source_guard = ArtifactGuard(extras)
    phase = {'start': started, 'source_commit': source, 'production_fixes': ['ef662ee3f7d6708be8d212aa9b7b5861869d1d5f', 'c4df45e843647d83702be311ed5f484ed4a690c3'], 'build_environment_overrides': OVERRIDES, 'cache_copies': False, 'source_sha256': {str(q.relative_to(check.ROOT)): digest(q) for q in extras}, 'stage_accepted': False}
    (out / 'phase.json').write_text(json.dumps(phase, indent=2) + '\n')
    ordinary_env = check.env
    def low_disk_env(directory):
        env = ordinary_env(directory)
        env.update(OVERRIDES)
        return env
    check.env = low_disk_env
    cargo = [check.TC / 'bin/cargo', '--config', 'profile.dev.debug=0', '--config', 'profile.test.debug=0', '--config', 'build.incremental=false']
    def run(label, args):
        check.require(shutil.disk_usage('/tmp').free > 20 * 1024**3, 'low disk before ' + label)
        code = check.run(out, label, [*cargo, *args], extra_inputs=extras)
        historical.check()
        source_guard.check()
        check.require(code == 0, label + ' failed; raw outcome retained')
    run('root-compiler-build', ['build', '--offline', '--locked', '-p', 'rust_verify'])
    (out / 'bundle/rust_verify').unlink()
    shutil.copy2(out / 'target/debug/rust_verify', out / 'bundle/rust_verify')
    for profile in [out / 'build/debug', out / 'target/debug']:
        profile.mkdir(parents=True, exist_ok=True)
        for n in [*old, 'verus-root']:
            src, dst = out / 'bundle' / n, profile / n
            if dst.exists() and digest(dst) == digest(src):
                continue
            if dst.exists():
                dst.unlink()
            shutil.copy2(src, dst)
            check.require(digest(src) == digest(dst), 'private profile tool ' + n)
    tools = ArtifactGuard([out / 'bundle' / n for n in old])
    run('root-vir-tests', ['test', '--offline', '--locked', '-p', 'vir', '--lib', '--', '--test-threads=1'])
    run('root-native-controls', ['test', '--offline', '--locked', '-p', 'rust_verify', '--lib', 'checked_owned_cell::tests::loaded_native_methods_and_complete_mir_corruptions', '--', '--nocapture', '--test-threads=1'])
    run('root-focused-tests', ['test', '--offline', '--locked', '-p', 'rust_verify_test', '--test', 'local_checked_owned_cell', '--test', 'local_checked_owned_cell_admission', '--', '--test-threads=1'])
    tools.check()
    phase.update(end=datetime.datetime.now(datetime.timezone.utc).isoformat(), success=True, guard_passed=True, tools_sha256={n: digest(out / 'bundle' / n) for n in old}, expected_focused_invocations=226, expected_focused_tests=12, expected_vir_tests=172, native_body_proofs=0, state_proofs=0, source_coupling_proved=False, publication=False)
    (out / 'receipt.json').write_text(json.dumps(phase, indent=2) + '\n')
    print(json.dumps({'output': str(out), 'success': True, 'stage_accepted': False}), flush=True)
if __name__ == '__main__':
    main()
