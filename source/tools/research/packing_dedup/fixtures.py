from trials import *
import importlib.util
spec=importlib.util.spec_from_file_location('pure_source',ROOT/'source/tools/research/item_tag_constructor_composition/source.py');S=importlib.util.module_from_spec(spec);spec.loader.exec_module(S)
def fixture():
 # Rebind pure generator input to authenticated completed full-help body bytes.
 # Never follow its historical upstream path into another artifact/checkout.
 p=Path('/tmp/item-packing-review-vr4p8_bo/full-help.rs')

 text=p.read_text();helper=S.block(text,'proof fn packed_fields_raw(')
 clean=text.replace(helper+'\n','')
 while 'proof {' in clean:
  b=S.block(clean,'proof {');a=clean.index('proof {');clean=clean[:a-8]+clean[a+len(b)+1:]
 # Preserve exactly the generator's original body indentation/newlines.
 bor=re.search(r'#\[derive\(Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord\)\]\npub struct BorTag\(NonZero<u64>\);',text).group()+'\n'
 it=re.search(r'#\[derive\(Copy, Clone, Hash, PartialEq, Eq\)\]\npub struct Item\(u64\);',text).group()+'\n'
 it+='\n'.join(re.findall(r'^const \w+: u64 = .*;',text,re.M))+'\n'+S.block(text,'pub enum Permission')+'\n'
 anchors=[('b','pub fn new(i:'),('b','pub fn get('),('i','pub fn tag('),('i','pub fn new(tag:'),('i','pub fn perm('),('i','pub fn protected('),('i','fn to_bits('),('i','fn from_bits(')]
 for owner,a in anchors:
  body=S.block(clean,a)
  if owner=='b':bor+='    '+body+'\n'
  else:it+='    '+body+'\n'
 it+='\n'.join(re.findall(r'^    const \w+: u64 = .*;',text,re.M))+'\n'
 S.load=lambda:(bor,it)
 full=S.generate('full');helped=S.generate('full-help')
 need(helped==text,'pure full-help regeneration byte mismatch')
 need(not re.search(r'\bproof\b|\bassert\s*\(',full),'local proof glue')
 fidelity=S.fidelity(full)
 original=json.loads((ROOT/'evidence/local-checked-foreign-review/RESULTS.json').read_text())['fidelity']
 need(set(fidelity)==set(original),'fidelity inventory')
 for n,v in fidelity.items():need(v['adapted_body_sha256']==original[n]['original_body_sha256'],'original body '+n)
 return full,helped,fidelity

if __name__=='__main__':
 full,helped,fd=fixture();need(hashlib.sha256(full.encode()).hexdigest()=='1adb8b6c2ee64cc1c6930c9530920105ad26cee8abf3212caff53dda0c36518d','exact full');(R/'full.rs').write_text(full);write(R/'fidelity.json',fd)
