from trials import *
need(sys.flags.dont_write_bytecode,'use -B')
# Two real, independently self-consistent producer artifacts with the same
# crate identity but distinct frozen certificate fingerprints must not merge.
s='use vstd::prelude::*; verus! {pub fn value()->u64 {0}}'
opts=['--import','second='+str(R/'stable-full/warm/component.vir')]
r=app('conflicting-completed-artifacts','conflict',s,[('component',R/'stable-full/cold')],options=opts)
require('conflicting-completed-artifacts',r,None,success=False)
need('conflicting inference artifacts' in (R/'conflicting-completed-artifacts/stderr').read_text(),'no last-wins artifact merge')
require('cpu-provider',app('cpu-provider','provider',BOOL,goals=True,options=['-C','target-cpu=x86-64']),2)
client=BOOL.replace('u64::from(b)','provider::all_bool(b)')
require('cpu-matching-client',app('cpu-matching-client','cpu_client',client,[('provider',R/'cpu-provider')],options=['-C','target-cpu=x86-64']),1)
for order in ['current-first','current-last']:
 first=R/('cpu-provider' if order=='current-first' else 'bool-provider')
 second=R/('bool-provider' if order=='current-first' else 'cpu-provider')
 label='foreign-conflict-'+order
 r=app(label,'foreign_conflict',s,[('provider',first)],options=['--import','second='+str(second/'provider.vir'),'-C','target-cpu=x86-64'])
 require(label,r,None,success=False)
 need('checked foreign identity mismatch' in (R/label/'stderr').read_text(),'foreign conflict must reject regardless of order')
