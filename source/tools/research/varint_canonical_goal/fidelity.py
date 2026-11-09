"""Exact executable/predicate comparison for the frozen macro syntax adapter."""
from pathlib import Path
import re
import sys
HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[3]
sys.path.insert(0, str(ROOT / 'source/tools'))
from infer_experiment import require

BASE = ROOT / 'evidence/verus-infer-varint-3b05c191/diagnostics/application-no-layout-ordinary.rs'

def without_proof_blocks(s):
    while match := re.search(r'\bproof\s*\{', s):
        end = match.end()
        depth = 1
        while depth:
            require(end < len(s), 'unbalanced proof block')
            depth += (s[end] == '{') - (s[end] == '}')
            end += 1
        s = s[:match.start()] + s[end:]
    return s

def executable(s):
    s = s[s.index('macro_rules! encode {'):s.index('#[verus_spec(encoded')]
    s = re.sub(r'for b in it: \$buf.iter_mut\(\)\n.*?\n        \{',
               'for b in $buf.iter_mut() {', s, flags=re.S)
    s = without_proof_blocks(s)
    s = re.sub(r'//[^\n]*', '', s)
    return re.findall(r'[A-Za-z_$][A-Za-z0-9_$]*|[0-9]+|[^\s]', s)

def fidelity(path):
    original = BASE.read_text()
    candidate = Path(path).read_text()
    require(executable(original) == executable(candidate), 'executable macro changed')
    # The semantic predicate/recursive radix are byte-for-byte unchanged.
    require(original[:original.index('macro_rules! encode')].strip() ==
            candidate[:candidate.index('\n\nverus! {\n// Diagnostic') if '// Diagnostic' in candidate
                      else candidate.index('macro_rules! encode')].strip(), 'semantic definitions changed')
    require(original[original.index('#[verus_spec(encoded'):] ==
            candidate[candidate.index('#[verus_spec(encoded'):], 'signature, contract or domain changed')
    require(not any(t in candidate for t in ('let ghost', 'assume(', 'admit(', 'external_body',
            'layout_', 'no_lifetime', 'exec_allows_no_decreases_clause')), 'snapshot or trust bypass')

if __name__ == '__main__':
    for arg in sys.argv[1:]:
        fidelity(Path(arg))
        print(arg, 'fidelity OK')
