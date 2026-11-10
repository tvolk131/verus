#!/usr/bin/env python3
"""Independent exit inventory with exact inherited short-name normalization."""
import inspect
from pathlib import Path
import re
import sys
sys.dont_write_bytecode=True
import review as r

# Preserve the previously frozen independent procedure, changing only parser
# normalization and the fresh output name. The exact resulting source is saved,
# frozen before execution and normally compiles/replays the original Coq modules.
# This Python-level specialization creates no Coq proofs or assumptions.
text=inspect.getsource(r.supplement)
old="axioms=re.findall(r'^(Coq\\.\\S+)',text,re.M)"
new="""axioms=re.findall(r'^(\\S+)\\s*:',text.split('Axioms:\\n',1)[1],re.M)
            aliases={'functional_extensionality_dep':'Coq.Logic.FunctionalExtensionality.functional_extensionality_dep',
                     'Eq_rect_eq.eq_rect_eq':'Coq.Logic.Eqdep.Eq_rect_eq.eq_rect_eq'}
            axioms=[aliases.get(a,a) for a in axioms]"""
r.require(text.count(old)==1,'unexpected original individual parser')
text=text.replace(old,new)
r.require(text.count("work=RAW/(short+'-independent')")==1,'unexpected private directory expression')
text=text.replace("work=RAW/(short+'-independent')","work=RAW/(short+'-independent-002')")
r.require(text.count("kernel_axioms=re.findall(r'^  (Coq\\.\\S+)',summary,re.M)")==1,'unexpected kernel parser')
text=text.replace("kernel_axioms=re.findall(r'^  (Coq\\.\\S+)',summary,re.M)",
                  "kernel_axioms=re.findall(r'^    (Coq\\.\\S+)',summary,re.M)\n    require(set(kernel_axioms)==allowed,'missing exact inherited kernel assumption inventory')")
# The new output records supersede only the independent exit command entries;
# retain and authenticate all successful outputs from the parser-failed attempt.
freeze=r.read(r.HERE/'exit-audit-freeze.json')
r.require(freeze['script_sha256']==r.sha(Path(__file__)),'exit audit changed')
r.require(freeze['specialized_source_sha256']==r.hashlib.sha256(text.encode()).hexdigest(),'audit specialization changed')
for p,h in freeze['previous_attempt_files'].items():
    r.require(r.sha(p)==h,'failed attempt changed '+p)
r.write(r.RAW/'exit-inventory-parser-failure.json',{'success':False,'semantic_negative':False,
    'stage':'independent individual-assumptions output parsing',
    'failure':'Coq prints functional_extensionality_dep as a short name; qualified-only parser rejected it',
    'commands_failed':[], 'kernel_replay_in_failed_supplement':'not reached',
    'previous_attempt_files':freeze['previous_attempt_files'],
    'source_of_correction':r.pin(r.HERE/'exit-audit-freeze.json')})
(r.RAW/'exit-audit-specialized.py').write_text(text)
namespace=dict(vars(r))
exec(compile(text,str(r.RAW/'exit-audit-specialized.py'),'exec'),namespace)
namespace['supplement']('exit')
