#!/usr/bin/env python3
"""Keep every development snapshot; failed compilation is not semantic evidence."""
import subprocess,sys
from pathlib import Path
p=Path(__file__).resolve().parent
n=1
while (p/f'trial-{n:03}.json').exists() or Path(f'/tmp/model-input-context-trial-{n:03}').exists(): n+=1
freeze=p/f'trial-{n:03}.json'
base=[sys.executable,'-B','-O',str(p/'check.py'),'--freeze',str(freeze)]
subprocess.run(base+['--create-freeze'],check=True)
raise SystemExit(subprocess.run(base+['--output',f'/tmp/model-input-context-trial-{n:03}']).returncode)
