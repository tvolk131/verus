"""Explicit, NON-ACCEPTED frontend diagnostics; never an original-source proof."""
from pathlib import Path
import json
import tempfile
from source_selection import selection, require

if __name__ == '__main__':
    meta,literal=selection()
    here=Path(__file__).resolve().parent
    require(meta==json.loads((here/'selection.json').read_bytes()),'source freeze changed')
    out=Path(tempfile.mkdtemp(prefix='native-miri-tag-diagnostic-',dir='/tmp'));out.chmod(0o700)
    closure=literal.replace('.map(BorTag)', '.map(|n| BorTag(n))').replace('.map(Self)', '.map(|n| Self(n))')
    require(closure!=literal,'diagnostic did not alter source')
    (out/'closure.rs').write_text('use vstd::prelude::*;\nverus! {\n'+closure+'\n}\n')
    # A diagnostic using the genuine carrier but only original get/inner bodies.
    carrier='use vstd::prelude::*;\nuse std::num::NonZero;\nverus! {\n'
    carrier+='#[derive(Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]\npub struct BorTag(NonZero<u64>);\n'
    carrier+='impl BorTag { pub fn get(&self) -> u64 { self.0.get() }\n'
    carrier+='pub fn inner(&self) -> NonZero<u64> { self.0 } }\n}\n'
    (out/'carrier.rs').write_text(carrier)
    print(out)
