"""Bind final compact artifacts; this does not certify any new foreign law."""
import json
from pathlib import Path
import subprocess
import sys
import controller as c


def main():
    c.need(sys.flags.dont_write_bytecode, 'run with Python -B')
    bundle = c.authenticate()
    results = json.loads((c.HERE / 'RESULTS.json').read_text())
    c.need(results['decision'] == 'INCOMPLETE_no_registered_checked_foreign_contract'
           and results['production_changes'] is False and results['publication'] is False
           and results['goal_complete'] is False, 'honest incomplete status')
    c.need(results['all_accepted_evidence_guards_passed'] is True
           and len(results['excluded_guard_failures']) == 1, 'guard rejection retained')
    changed = subprocess.check_output(['git', 'diff', '--name-only', '098a8be6'], cwd=c.ROOT).decode().splitlines()
    changed += subprocess.check_output(['git', 'ls-files', '--others', '--exclude-standard'], cwd=c.ROOT).decode().splitlines()
    c.need(all(p.startswith('source/tools/research/checked_bool_from_mir/') for p in changed),
           'research-only ownership')
    files = {p.name: c.sha(p) for p in c.HERE.iterdir()
             if p.is_file() and p.name != 'FINAL_GUARDS.json'}
    manifest = dict(base='098a8be66e00734a010bf525ff832a81157621ea',
        files=files, tools=bundle['tools'], early_interface_commit='ae3cd734',
        production_changes=False, foreign_contract_available=False,
        selected_evidence_guards_passed=True, rejected_guard_epochs=results['excluded_guard_failures'],
        unreceipted_setup_failures=2, unreceipted_locator_failure=1,
        publication=False, native_correspondences=0, goal_complete=False)
    c.write(c.HERE / 'FINAL_GUARDS.json', manifest)
    print(json.dumps(dict(files=len(files), research_only=True, contract_available=False,
                          selected_evidence_guards=True, rejected_guards=1)))


if __name__ == '__main__':
    main()
