"""Rustc-only macro-erasure observation, not a source/native proof."""
from build import *
f=json.loads((root()/'bundle-exact-cold/freeze.json').read_text());p=next(iter(f['fixtures']))
j=run('review-erased-expansion',[TC/'bin/rustc',p,'--edition=2021','--crate-type=lib','--extern','vstd='+str(bundle()/'libvstd.rlib'),'-L','dependency='+str(bundle()),'-Zunpretty=expanded'])
need(j['exit']==0,'erasure diagnostic setup')
s=(root()/'review-erased-expansion/stdout').read_text()
need('a.cmp(b)' in s and 'checked_nonzero' not in s,'original erased std call observation')
write(root()/'erased-observation.json',dict(erased_source_sha256=sha(root()/'review-erased-expansion/stdout'),std_cmp_kept=True,synthetic_helper_present=False,classification='ordinary rustc macro-erased source diagnostic; not native correspondence'))
