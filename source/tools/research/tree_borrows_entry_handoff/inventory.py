"""Inventory the compiled module and kernel declarations, including generated ones."""
import hashlib, json, pathlib, re, sys

OWN = pathlib.Path(__file__).resolve().parent
OUT = pathlib.Path('/tmp/tree-borrows-entry-handoff-20261110')

def sha(p):
    return hashlib.sha256(p.read_bytes()).hexdigest()

def write(p, x):
    p.write_text(json.dumps(x, indent=2) + '\n')

if sys.argv[1] == 'prepare':
    log = (OUT / 'kernel-final-01/stdout.log').read_text()
    receipt = json.loads((OUT / 'kernel-final-01/receipt.json').read_text())
    if receipt['exit_code'] != 0 or sha(OUT / 'kernel-final-01/stdout.log') != receipt['log_sha256']:
        raise RuntimeError('unsuccessful kernel receipt')
    all_constants = sorted(set(re.findall(r'checking cst:([^\s]+)', log)))
    own_constants = [x for x in all_constants if x.startswith('EntryHandoff.Entry.')]
    # The inductive and constructor are separately queried in addition to every
    # generated constant found by the recursive kernel check.
    names = sorted(set(own_constants + ['EntryHandoff.Entry.entry_result', 'EntryHandoff.Entry.Build_entry_result']))
    if not own_constants:
        raise RuntimeError('no own kernel constants')
    (OUT / 'queries').mkdir(exist_ok=True)
    lines = ['From EntryHandoff Require Import Entry.',
             'Set Printing Width 1000000.', 'Set Printing Depth 1000000.', 'Set Printing All.',
             'Redirect "' + str(OUT / 'queries/module') + '" Print Module EntryHandoff.Entry.']
    queries = []
    for i, name in enumerate(names):
        stem = OUT / 'queries' / ('declaration-%03d' % i)
        lines += ['Redirect "' + str(stem) + '-type" Check @' + name + '.',
                  'Redirect "' + str(stem) + '-assumptions" Print Assumptions ' + name + '.']
        queries.append({'name': name, 'type_file': str(stem) + '-type.out',
                        'assumptions_file': str(stem) + '-assumptions.out'})
    (OUT / 'inventory.v').write_text('\n'.join(lines) + '\n')
    write(OUT / 'inventory-plan.json', {
        'kernel_constants': all_constants, 'own_kernel_constants': own_constants,
        'checked_libraries': sorted(set(re.findall(r'Checking library: ([^\s]+)', log))),
        'kernel_own_lines': [x for x in log.splitlines() if 'EntryHandoff.Entry' in x],
        'queries': queries})
    print(len(own_constants), 'own constants;', len(names), 'total declarations')
elif sys.argv[1] == 'collect':
    plan = json.loads((OUT / 'inventory-plan.json').read_text())
    receipt = json.loads((OUT / 'inventory-final-01/receipt.json').read_text())
    if receipt['exit_code'] != 0 or sha(OUT / 'inventory-final-01/stdout.log') != receipt['log_sha256']:
        raise RuntimeError('unsuccessful inventory query')
    module = (OUT / 'queries/module.out').read_text()
    visible = re.findall(r'(?:Definition|Parameter)\s+(entry_\w+)\s*:', module)
    expected = {x.rsplit('.', 1)[-1] for x in plan['own_kernel_constants']}
    if set(visible) - expected:
        raise RuntimeError('module/kernel inventory gap: ' + repr(set(visible) - expected))
    rows = []
    for q in plan['queries']:
        t = pathlib.Path(q['type_file']); a = pathlib.Path(q['assumptions_file'])
        assumptions = a.read_text().strip()
        if assumptions != 'Closed under the global context':
            raise RuntimeError('non-CLOSED own declaration: ' + q['name'])
        rows.append({'name': q['name'], 'type': t.read_text().strip(),
                     'assumptions': assumptions, 'type_sha256': sha(t), 'assumptions_sha256': sha(a)})
    write(OWN / 'INVENTORY.json', {
        'method': 'full recursive kernel declaration list + compiled Print Module + individual Check/Print Assumptions',
        'own_kernel_constants': plan['own_kernel_constants'],
        'inductives_and_constructors': ['EntryHandoff.Entry.entry_result', 'EntryHandoff.Entry.Build_entry_result'],
        'compiled_module': module, 'compiled_module_sha256': sha(OUT / 'queries/module.out'),
        'all_own_declarations': rows, 'all_individually_closed': True,
        'recursive_checked_libraries': plan['checked_libraries'],
        'recursive_kernel_constant_count': len(plan['kernel_constants']),
        'kernel_own_lines': plan['kernel_own_lines']})
    print(len(rows), 'declarations individually CLOSED')
