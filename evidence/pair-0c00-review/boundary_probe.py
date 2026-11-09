#!/usr/bin/env python3
"""Repeat db26 boundary attacks with policy67 and old-policy66 rejection.

The historical probe stays byte-for-byte unchanged. Its Python assertions are
test oracles; this wrapper explicitly refuses optimized execution.
"""
from pathlib import Path
import runpy
import sys
import tempfile

if not __debug__:
    raise SystemExit('run this review test without -O')
root = Path(__file__).resolve().parents[2]
original = root / 'evidence/combined-db26-review/probe.py'
text = original.read_text()
text = text.replace("meta['policy'] == 'candidate-bound-evidence-v66'",
                    "meta['policy'] == 'candidate-bound-evidence-v67'")
text = text.replace('for old in [63,64,65]:', 'for old in [63,64,65,66]:')
with tempfile.TemporaryDirectory(prefix='pair-boundary-run-') as folder:
    script = Path(folder) / 'probe.py'
    script.write_text(text)
    runpy.run_path(str(script), run_name='__main__')
