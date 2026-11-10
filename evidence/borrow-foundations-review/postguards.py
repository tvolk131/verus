#!/usr/bin/env python3
"""Posttrial read-only guards include file-set equality, not just listed hashes."""
from review import *
result={}
for key in SPEC:
 w=ROOT/key;roots={}
 for base,files in load(w/'installed-before.json').items():
  b=P(base)
  actual={str(p.relative_to(b)) for p in b.rglob('*') if (p.is_file() or (key=='tb' and p.is_symlink())) and (key=='sb' or '.git' not in p.relative_to(b).parts)}
  assert actual==set(files),(key,base,sorted(actual-set(files)),sorted(set(files)-actual))
  roots[base]=guard(b,files)
 expected={p for p in load(w/'source.json') if p.endswith('.v')};actual={str(p.relative_to(w/'src')) for p in (w/'src').rglob('*.v')}
 assert actual==expected,(key,actual^expected)
 result[key]={'installed_file_sets_and_content':roots,'upstream_proof_sources_exact_set':len(expected),'proof_source_guard':guard(w/'src',load(w/'source.json')),'objects':guard(w/'src',load(w/'objects.json'))}
save(ROOT/'postguards.json',{'controller_sha256':sha(__file__),'guards':result});print('all installed file sets/content and upstream source sets unchanged')
