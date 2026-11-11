"""Compare raw typed body bytes with committed shipping hashes, rebinding only exact filenames."""
import audit as a
import re,json,hashlib
for target,key,oldname in [('full','full','stable-full'),('packe','reuse','stable-reuse')]:
 d=a.R/target/'cold';fixture=next(iter(json.loads((d/'freeze.json').read_text())['fixtures']));old=next(iter(json.loads((a.t.OLD/oldname/'cold/freeze.json').read_text())['fixtures']));expected=json.loads((a.t.ROOT/'source/tools/research/item_packing_goal/INVENTORY.json').read_text())[key]['ordinary_original_functions'];txt=(d/'vir/crate.vir').read_text();got={}
 for m in re.finditer(r'\(Function\s',txt):
  s=txt[m.start():a.end(txt,m.start())];n=a.field(s,'name');path=n[len('(Fun :path '):-1]
  if path in expected:got[path]=hashlib.sha256(a.field(s,'body').replace(fixture,old).encode()).hexdigest()
 a.t.need(got.keys()==expected.keys() and all(h==expected[n]['body_sha256'] for n,h in got.items()),'ALL raw typed body hashes '+target)
 a.t.write(a.t.HERE/('COMMITTED_BODY_'+('FULL' if target=='full' else 'PACKET')+'.json'),dict(actual_fixture=fixture,committed_fixture=old,hashes=got,normalization='Only exact fixture filename replacement; no whitespace/operator/type/coordinate normalization.'))
print('all full38 and packet3 raw typed body hashes match committed source inventory')
