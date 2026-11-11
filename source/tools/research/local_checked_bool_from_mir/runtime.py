"""Concrete erasure/runtime smoke test, explicitly not a native theorem."""
from pathlib import Path
import sys
import trials as t
r=Path(sys.argv[1]);b=r/'bundle';d=r/'runtime';d.mkdir(exist_ok=True)
text=t.source('full-help')+'''
fn main() {
    let tag=bor_tag();
    let yes=Item::new(tag,Permission::Unique,true);
    let no=Item::new(tag,Permission::Unique,false);
    assert!(yes.protected()); assert!(!no.protected());
    assert_eq!(yes.tag().get(),1); assert_eq!(no.tag().get(),1);
}
fn bor_tag()->BorTag { BorTag::new(1).unwrap() }
'''
src=d/'component.rs';src.write_text(text)
args=[b/'verus',src,'--crate-name=component','--crate-type=bin','--infer-goals','--no-cheating','--output-json','--compile','--emit=link,mir','-o',d/'component']
out,record=t.run(r,'runtime-build',args,b);t.need(record['exit']==0,'runtime compilation/proof')
out,record=t.run(r,'runtime-execution',[d/'component'],b);t.need(record['exit']==0,'concrete runtime')
