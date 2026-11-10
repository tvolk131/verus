import hashlib,json,pathlib,re
r=pathlib.Path.cwd(); o=r/'NEW/evidence/published-entry-source-review'
f=json.loads((r/'source/tools/research/stbor_entry_handoff/candidate003-freeze.json').read_text())
s=(r/'source/tools/research/stbor_entry_handoff/candidate.v').read_text()
names=re.findall(r'^(?:Definition|Lemma)\s+(\w+)',s,re.M)
if names!=f['expected_declarations']:raise ValueError('inventory')
text=re.sub(r'\(\*.*?\*\)','',s,flags=re.S)
for word in ['Admitted','admit','Axiom','Axioms','Parameter','Parameters']:
 if re.search(r'\b'+word+r'\b',text):raise ValueError(word)
for v in json.loads((o/'downloaded-primary.json').read_text()):
 if hashlib.sha256(pathlib.Path(v['path']).read_bytes()).hexdigest()!=v['sha256']:raise ValueError(v['path'])
receipt=json.loads((o/'policy-inspection/receipt.json').read_text())
if not receipt['success'] or not receipt['source_map_matches_pins'] or receipt['inspected_files']!=10:raise ValueError('identity inspector')
result=dict(classification='lexical_inventory_and_identity_only',declarations=names,forbidden_declaration_tokens_found=[],downloaded_primary_hashes_match=True,policy_identity_inspector_success=True,compilation='not_run',kernel_verification='not_run',proofs_accepted=0)
(o/'static-results.json').write_text(json.dumps(result,indent=2)+'\n')
print('12 declaration names match; lexical assumption/admission scan clear; identity-only checks pass')
