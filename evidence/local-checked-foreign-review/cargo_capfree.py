from controls import raw,wrap
import review as t
for goals in [False,True]:
 label='cargo-capfree-'+('infer-aware' if goals else 'ordinary')
 r=raw(label,wrap('pub fn seed()->u64 {0}'),goals=goals,cargo=True)
 t.need(r['exit']==0 and r['verification']['verified']==1 and r['verification']['errors']==0,'healthy cargo ordinary export')
 t.need((t.R/label/'libcomponent.vir').is_file(),'healthy cargo selected VIR')
