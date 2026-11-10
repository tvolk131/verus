"""Explicit bounded imported-interface probes. No publication outside private /tmp."""
import json
from pathlib import Path
import epoch as e

def main():
    rows=[json.loads(x) for x in (e.HERE/'attempts.jsonl').read_text().splitlines()]
    p=Path(next(x['path'] for x in rows if x['label']=='tag-provider-final'))
    w=Path(json.loads((e.HERE/'work.json').read_text())['foreign_work']);src=w/'lookup_client.rs'
    src.write_text('''use vstd::prelude::*;
extern crate component;
verus! {
pub fn read(i:component::Item)->(r:u64)
requires component::item_word_wf(i.word()), ensures r==(i.word() & (u64::MAX >> 3)),
{ i.tag().get() }
}
''')
    e.run('lookup-import',[e.B/'verus',src,'--crate-name=client','--crate-type=lib','--no-cheating','--output-json','--log-all','--log-dir','__EPOCH__/vir','--extern','component='+str(p/'libcomponent.rlib'),'--import','component='+str(p/'component.vir'),'-L',p],extras=[src,p/'libcomponent.rlib',p/'component.vir',p/'receipt.json'],planextra={'expected_exec':['client::read'],'expected_local_details':['client::read'],'expected_helpers':[],'imported_closed_bodies_must_be_absent':['component::impl&%5::eq','component::impl&%9::__verus_infer_nonzero_u64_view','component::impl&%11::eq_spec','component::impl&%19::value','component::impl&%20::word']})
if __name__=='__main__':main()
