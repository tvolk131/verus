#!/usr/bin/env python3
"""Read-only reuse of committed source projection; no executable body edits."""
import sys
sys.dont_write_bytecode = True
import importlib.util
from pathlib import Path
HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parent/'source'))
spec = importlib.util.spec_from_file_location('committed_source_probes', HERE.parent/'source/probes.py')
p = importlib.util.module_from_spec(spec)
spec.loader.exec_module(p)
NOMINAL = '''
// Opaque nominal identities only. No method specifications, views or field assumptions.
#[verifier::external_type_specification]
#[verifier::external_body]
pub struct ExIoError(std::io::Error);
#[verifier::external_type_specification]
#[verifier::external_body]
pub struct ExPanicLocation<'a>(core::panic::Location<'a>);
'''
def baseline(nominal=True):
    text = p.selection()
    if nominal:
        text = text.replace('verus! {', 'verus! {\n'+NOMINAL, 1)
    return text
