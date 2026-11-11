#!/usr/bin/env python3
"""Redirect ONLY unit-test fixture-path env!; preserve all source and rustc args."""
import os,sys,json
from pathlib import Path
a=sys.argv[1:]
if '--test' in a and '--crate-name' in a and a[a.index('--crate-name')+1]=='rust_verify':
 p=Path('/tmp/practical-word-combined-review-5ecea5fe/library-fixtures/rust_verify');p.mkdir(parents=True,exist_ok=True)
 os.environ['CARGO_MANIFEST_DIR']=str(p)
 Path('/tmp/practical-word-combined-review-5ecea5fe/library-test-compile.json').write_text(json.dumps({'argv':a,'CARGO_MANIFEST_DIR':str(p)},indent=2)+'\n')
os.execvpe(a[0],a,os.environ)
