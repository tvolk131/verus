#!/usr/bin/env python3
"""Authenticate additive controller epochs by rehoming labels to this checkout."""
from review import *
out={}
for filename in ['supplemental-controller-freeze.json','object-controller-freeze.json']:
 d=load(SB/'manifests'/filename);matched={}
 for label,h in d['files'].items():
  marker='/source/tools/research/stbor_checked_foundation/'
  p=DEL/'stbor_checked_foundation'/label.split(marker,1)[1] if marker in label else safe_artifact(label)
  checked(p,h);matched[label]={'checked_at':str(p),'sha256':h}
 out[filename]=matched
support=load(DEL/'tree_borrows_checked_foundation/SUPPORT.json')
for name,e in support.items():checked(DEL/'tree_borrows_checked_foundation'/name,e['sha256'])
graph=load(SB/'manifests/object-dependency-map.json')
checked(safe_artifact(graph['reader']),graph['reader_sha256']);checked(SB/'object-reader-v3/vo_summary.ml',graph['reader_source_sha256'])
assert len(graph['modules'])==518
for name,e in graph['modules'].items():assert set(e['recorded_compiled_dependencies'])<=set(graph['modules'])
checked(SB/'opamroot/config',load(SB/'manifests/upstream-tool-freeze.json')['opam_config_sha256'])
# Authenticate all delivered individual TB assumption outputs against bound classes.
q=load(TB/'inventory-queries.json');a=load(DEL/'tree_borrows_checked_foundation/ASSUMPTIONS.json')
for e in q['internal_constants']:checked(safe_artifact(e['assumptions']),a['individual_constant_classes'][e['name']])
save(ROOT/'controller-authentication.json',{'controller_sha256':sha(__file__),'sb_rehomed_epochs':out,'tb_support_sources':support,'sb_compiled_map_modules':len(graph['modules']),'tb_assumption_outputs':len(q['internal_constants']),'historical_checkout_access':False})
print('additive controller epochs and compiled dependency map authenticated')
