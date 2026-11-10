"""Independent mathematical object comparison and pinned tool queries."""
import importlib.util, json, pathlib, sys, re
sys.dont_write_bytecode=True
P=pathlib.Path
spec=importlib.util.spec_from_file_location('review',P(__file__).with_name('replay.py'))
r=importlib.util.module_from_spec(spec); spec.loader.exec_module(r)
d=r.OUT/'object-comparison'; d.mkdir()
r.save(d/'pretrial.json',{'controller_sha256':r.sha(__file__),'replay_support_sha256':r.sha(r.E/'replay.py'),'before_queries':True,'delivered_entry_object_sha256':r.sha(r.TB/'final-01-source/Entry.vo'),'fresh_entry_object_sha256':r.sha(r.OUT/'tree/source/Entry.vo')})
env=r.clean(d,r.BT/'opamroot/tb/bin'); env['LD_LIBRARY_PATH']=str(r.BT/'deps/lib')
inv=r.load(r.E/'tree-INVENTORY.json'); names=list(inv['declarations'])
for label,src in [('delivered',r.TB/'final-01-source'),('fresh',r.OUT/'tree/source')]:
 q=d/label; q.mkdir()
 lines=['From EntryHandoff Require Import Entry.','Set Printing All.','Set Printing Universes.','Set Printing Depth 1000000.','Set Printing Width 1000000.']
 for i,n in enumerate(names): lines.append(f'Redirect "{q}/{i}" Print {n}.')
 query=q/'bodies.v'; query.write_text('\n'.join(lines)+'\n')
 r.run(d,label,[str(r.BT/'opamroot/tb/bin/rocq'),'repl','-q','-batch','-Q',str(r.BT/'simuliris/theories'),'simuliris','-Q',str(src),'EntryHandoff','-l',str(query)],env)
rows={}
for i,n in enumerate(names):
 a=d/f'delivered/{i}.out'; b=d/f'fresh/{i}.out'
 r.need(a.read_bytes()==b.read_bytes(),'mathematical object body differs '+n)
 rows[n]={'identical_printed_body':True,'sha256':r.sha(a)}
objects={}
for n,fresh,old in [('candidate',r.OUT/'stbor/fresh/candidate.vo',r.ST/'candidate.vo'),('composition',r.OUT/'stbor/fresh/composition.vo',r.ST/'composition.vo'),('Entry',r.OUT/'tree/source/Entry.vo',r.TB/'final-01-source/Entry.vo')]:
 objects[n]={'fresh_sha256':r.sha(fresh),'delivered_sha256':r.sha(old),'byte_identical':r.sha(fresh)==r.sha(old),'fresh_size':fresh.stat().st_size,'delivered_size':old.stat().st_size}
 if n=='Entry':
  a=fresh.read_bytes(); b=old.read_bytes()
  oldpath=str(old.with_suffix('.v')).encode(); newpath=str(fresh.with_suffix('.v')).encode()
  r.need(a.count(newpath)==1 and b.count(oldpath)==1,'source metadata location')
  objects[n].update(fresh_embedded_source=str(fresh.with_suffix('.v')),delivered_embedded_source=str(old.with_suffix('.v')),source_path_size_difference=len(newpath)-len(oldpath),explanation='Rocq serialization embeds physical source path; complete printed bodies of all 17 declarations including opaque proofs are identical, same exact source, namespace, tool and upstream object set. No byte identity claimed for Entry.vo.')
r.save(r.E/'PRODUCT_COMPARISON.json',{'objects':objects,'tree_all_17_printed_bodies':rows,'pretrial_sha256':r.sha(d/'pretrial.json')})
# Version/loadpath evidence, distinct toolchains. No compiler/library mutation.
for name,argv in [('coq-version',[str(r.SB/'opamroot/legacy/bin/coqc'),'-v']),('stbor-ocaml-version',[str(r.SB/'opamroot/legacy/bin/ocamlc'),'-version']),('rocq-version',[str(r.BT/'opamroot/tb/bin/rocq'),'--version']),('tree-ocaml-version',[str(r.BT/'opamroot/tb/bin/ocamlc'),'-version'])]:
 r.run(d,name,argv,env)
print('All 17 Tree declaration bodies identical; pinned version queries passed')
