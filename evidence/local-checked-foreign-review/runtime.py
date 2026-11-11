from pathlib import Path
import review as t
from proofs import source
R=t.R;B=R/'bundle';d=R/'runtime';d.mkdir(exist_ok=True)
src=d/'component.rs';src.write_text(source('full-help')+'''
fn main() {
 let tag=BorTag::new(1).unwrap();
 let yes=Item::new(tag,Permission::Unique,true);
 let no=Item::new(tag,Permission::Unique,false);
 assert!(yes.protected());assert!(!no.protected());
 assert_eq!(yes.tag().get(),1);assert_eq!(no.tag().get(),1);
}
''')
r=t.run('runtime-build',[B/'verus',src,'--crate-name=component','--crate-type=bin','--infer-goals','--no-cheating','--output-json','--compile','--emit=link,mir','-o',d/'component'])
t.need(r['exit']==0,'runtime compilation')
t.need('<u64 as From<bool>>::from' in (d/'component.mir').read_text() or '<u64 as std::convert::From<bool>>::from' in (d/'component.mir').read_text(),'original runtime call')
t.need('checked_foreign_bool_to_u64' not in (d/'component.mir').read_text(),'helper escaped into runtime')
r=t.run('runtime-execution',[d/'component']);t.need(r['exit']==0,'runtime execution')
