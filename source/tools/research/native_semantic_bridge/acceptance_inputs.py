"""Complete model-source identity and final model-result checks; no native facts."""
from pathlib import Path
import re
import sys

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parents[3]/'source/tools'))
from infer_experiment import require
from proof_checks import NAMES


def recorded_model_sources(receipt):
    require(receipt['modules'] == list(NAMES) and set(receipt['proof_terms']) == set(NAMES),
            'incomplete model receipt module inventory')
    result = {}
    artifacts = receipt['proof_artifacts']
    for module in NAMES:
        matches = [info['sha256'] for name, info in artifacts.items()
                   if Path(name).name == module+'.v']
        require(len(matches) == 1 and isinstance(matches[0], str)
                and re.fullmatch(r'[0-9a-f]{64}', matches[0]) is not None,
                'missing or ambiguous recorded model source: '+module)
        result[module] = matches[0]
    return result


def accept_model_result(receipt, result, guards):
    """A failed final check must leave the initial false proof status intact."""
    require(bool(guards), 'missing final model guards')
    for guard in guards:
        guard.check()
    receipt.update(result)
