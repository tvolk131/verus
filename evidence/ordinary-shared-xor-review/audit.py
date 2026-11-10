#!/usr/bin/env python3
"""Read-only source/statement checks supplementing the normal Coq replay."""
import hashlib
import json
import re
from pathlib import Path
ROOT = Path(__file__).resolve().parents[2]
SUBJECT = ROOT/'source/tools/research/model_ordinary_shared_xor'

def sha(s):
    return hashlib.sha256(s.encode()).hexdigest()

def statement(source, name):
    s = re.search(r'^\s*(?:Lemma|Theorem) '+name+r'\b', source, re.M)
    assert s, name
    return re.split(r'\bProof\b',source[s.start():],maxsplit=1)[0].strip()

def main():
    reader = (SUBJECT/'candidates/reader_na.v').read_text()
    loop = (SUBJECT/'candidates/xor_na.v').read_text()
    adapter = (SUBJECT/'candidates/adapter.v').read_text()
    sc_loop = Path('/tmp/verus-infer-buffer-cycle-20261109/buffer-transform-trial007/proofs/transform.v').read_text()
    old = sc_loop.split('Definition zip_transform : val :=',1)[1].split('Lemma transform_prefix_length',1)[0]
    new = loop.split('Definition zip_transform_na : val :=',1)[1].split('Definition shared_xor_na_cost',1)[0]
    norm = lambda s: ''.join(s.split())
    assert old.count('!ˢᶜ') == 2 and old.count('<-ˢᶜ') == 1
    assert norm(old.replace('!ˢᶜ','!').replace('<-ˢᶜ','<-')) == norm(new)
    assert reader.count('iApply (wp_read_na_guarded_cells') == 2
    assert not re.search(r'\b(?:ScOrd|wp_read_sc\w*|shared_key_read|key_reader_read)\b',reader)
    prepared = statement(reader,'shared_key_reader_na_session_prepared')
    raw = statement(reader,'shared_key_reader_na_session')
    zero = loop[loop.index('  Lemma wp_shared_xor_na_zero'):]
    assert '£' not in prepared and '£(g+1)' in raw
    assert not any(x in zero for x in ['key_reader_na ', 'wp_read', 'wp_write', 'heap_mapsto_vec_cons', 'lookup'])
    assert 'shared_key_cap' in zero
    assert 'wp_write_na ⊤ p _ _ _ with "PadHead"' in loop and 'WriteCredit' not in loop
    old_setup = Path('/tmp/model-pair-reader-trial-012/proofs/setup.v').read_text()
    equalities = {}
    for name in ['raw_pair_reader_context_wp','raw_pair_reader_observed']:
        na_name = name.replace('raw_pair_reader','raw_pair_reader_na')
        old_st = statement(old_setup,name)
        new_st = statement(adapter,na_name)
        converted = new_st.replace(na_name,name).replace('key_reader_na ','key_reader ')
        assert norm(old_st) == norm(converted), name
        equalities[name] = dict(old_statement_sha256=sha(old_st),new_statement_sha256=sha(new_st),same_after_reader_rename=True)
    print(json.dumps(dict(algorithm=dict(only_three_orders=True,reads=2,writes=1,
        frozen_sc_body_sha256=sha(old),na_body_sha256=sha(new)),
        adapter_statements=equalities,prepared_session_no_credits=True,raw_prepare='g+1',
        zero_branch_reader_and_access_free=True,na_reader_guarded_cell_applications=2,
        new_store_input_credit=False,
        notes='Textual checks supplement manual proof review and successful recursive kernel checking; no semantic negative controls.'),indent=2))
if __name__ == '__main__': main()
