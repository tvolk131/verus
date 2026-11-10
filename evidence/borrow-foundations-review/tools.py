#!/usr/bin/env python3
"""Actual pinned version/loadpath queries in private cwd/environment."""
from review import *
identity={}
for k,(base,_,_,prefix,logical) in SPEC.items():
 w=ROOT/k;b=base/prefix/'bin';save(w/'tool-query-pretrial.json',{'controller_sha256':sha(__file__),'toolchain_manifest_sha256':sha(w/'installed-before.json')})
 commands=([['coqc','--version'],['coqchk','-v'],['ocamlc','-version'],['coqc','-config']] if k=='sb' else [['rocq','--version'],['rocq','compile','-config'],['ocamlc','-version']])
 q=[]
 for i,cmd in enumerate(commands):
  log=run(k,f'tool-version-{i}',[b/cmd[0]]+cmd[1:]);q.append({'argv':cmd,'output':log.read_text(),'binary_sha256':sha(b/cmd[0])})
 query=w/'loadpath.v';query.write_text('Print LoadPath.\n')
 argv=[b/'coqtop','-quiet','-batch','-coqlib',base/prefix/'lib/coq'] if k=='sb' else [b/'rocq','repl','-q','-batch']
 log=run(k,'loadpath',argv+['-Q',str(w/'src/theories'),logical,'-l',str(query)])
 identity[k]={'versions':q,'loadpath':log.read_text(),'controller_sha256':sha(__file__)}
save(OWN/'TOOLS.json',identity)
