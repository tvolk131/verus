#!/usr/bin/env python3
"""Additive query-only recovery; original build/kernel driver stays frozen.

SB checks installed libraries not imported by its roots. Require them before
Print Module. Keep rejected receipts/queries; use distinct receipt filenames
for queries so successful query receipts cannot collide with inventory data.
"""
import inspect
import review
from review import *
def query(key,extra_imports=False):
 w=ROOT/key
 text=inspect.getsource(review.inventory)
 name='inventory-v2' if extra_imports else 'inventory-query'
 text=text.replace("run(key,'inventory',",f"run(key,'{name}',")
 if extra_imports:
  text=text.replace("out=work/'queries'", "out=work/'queries-v2'").replace("'inventory.v'","'inventory-v2.v'").replace("'discovery.json'","'discovery-v2.json'")
  text=text.replace("'Require '+' '.join(modules(key))+'.'","'Require '+' '.join(libraries)+'.'")
 ns=dict(review.__dict__);exec(compile(text,str(__file__)+'::'+name,'exec'),ns)
 save(w/(name+'-pretrial.json'),{'controller_sha256':sha(__file__),'original_controller_sha256':sha(OWN/'review.py'),'adapted_query_controller_sha256':hashlib.sha256(text.encode()).hexdigest(),'kernel_log_sha256':sha(w/'kernel.log')})
 ns['inventory'](key);review.final_guard(key)
if __name__=='__main__':
 w=ROOT/'sb';old=load(w/'inventory.json')
 if 'declarations' not in old:
  assert old['state']=='finished' and old['exit_code']!=0
  checked(old['log'],old['log_sha256']);save(w/'inventory-rejected.json',old)
  query('sb',True)
 for action in [review.build,review.kernel]:action('tb')
 query('tb');review.final_guard('tb')
