"""Source-only observation adapters; pure reuse of the existing pinned extractor."""
import hashlib, importlib.util, json, re
from pathlib import Path
HERE = Path(__file__).resolve().parent
PREVIOUS = HERE.parent / 'item_tag_constructor_composition'
spec = importlib.util.spec_from_file_location('original_tag_source', PREVIOUS / 'source.py')
original = importlib.util.module_from_spec(spec)
spec.loader.exec_module(original)
need, sha = original.need, original.sha

def provider(mode):
    text = original.generate('tag')
    old = 'pub closed spec fn value(&self)->u64 { self.0@ }'
    need(text.count(old) == 1, 'exact original observation adapter')
    if mode == 'alias':
        text = text.replace(old, 'pub open spec fn value(&self)->u64 { self.__verus_infer_nonzero_u64_view() }')
    elif mode == 'zero':
        text = text.replace(old, 'pub open spec fn value(&self)->u64 { 0 }')
    else:
        need(mode in ['closed', 'compat'], 'known provider adapter')
    if mode == 'compat':
        text = text[:-2] + '''pub broadcast proof fn eq_value_compatibility(a:&BorTag,b:&BorTag)
ensures #[trigger] vstd::std_specs::cmp::PartialEqSpec::eq_spec(a,b) == (a.value()==b.value()),
{}
}\n'''
    original.fidelity(text)
    need(not re.search(r'\b(assume|admit|axiom|external_body|snapshot)\b', text), 'no mechanical assistance')
    return text

def client(mode):
    head = 'use vstd::prelude::*;\nextern crate component;\nverus! {\n'
    if mode.startswith('compat-'):
        head += 'broadcast use component::eq_value_compatibility;\n'
        mode = mode[len('compat-'):]
    read = '''pub fn read(i:component::Item)->(r:u64)
requires component::item_word_wf(i.word()), ensures r==(i.word() & (u64::MAX >> 3)),
{ i.tag().get() }
'''
    if mode == 'lookup': return head + read + '}\n'
    if mode == 'boundaries':
        return head + '''pub fn max()->(r:u64) ensures r==u64::MAX,
{ component::BorTag::new(u64::MAX).unwrap().get() }
pub fn highbit()->(r:u64) ensures r==0x8000000000000000u64,
{ component::BorTag::new(0x8000000000000000u64).unwrap().get() }
pub fn zero()->(r:bool) ensures r,
{ component::BorTag::new(0).is_none() }
}\n'''
    need(mode in ['equal', 'false'], 'known client adapter')
    relation = '!=' if mode == 'false' else '=='
    return head + read + '''pub fn equal(a:&component::BorTag,b:&component::BorTag)->(r:bool)
ensures r''' + relation + '''(a.value()==b.value()), { a==b }
}\n'''

def identity(text, is_provider):
    return dict(source_sha256=hashlib.sha256(text.encode()).hexdigest(),
                fidelity=original.fidelity(text) if is_provider else {},
                proof_blocks=text.count('proof {'), proof_functions=len(re.findall(r'proof fn ',text)), theorem_calls=0, snapshots=0, broadcast_imports=text.count('broadcast use '),
                private_carriers=is_provider,
                executable_body_tokens={n:hashlib.sha256(json.dumps(re.findall(r'\w+|[^\s]', original.block(text,a)[original.block(text,a).index('{'):]),separators=(',',':')).encode()).hexdigest() for n,a in ([('BorTag.new','pub fn new(i:'),('BorTag.get','pub fn get('),('Item.tag','pub fn tag('),('compare_equal','pub fn compare_equal(')] if is_provider else [])})
