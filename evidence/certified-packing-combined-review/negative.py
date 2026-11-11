"""Semantic and stale-output controls, using only freshly produced artifacts."""
from trials import *
need(sys.flags.dont_write_bytecode,'use -B')
# A normal import cannot expose an unused private foreign helper or closed executable body.
# Malformed/incompatible metadata controls are supplied by the unmodified compiler test target.
for cargo in [False,True]:
 for flag in ['--verify-function=all_bool','--verify-only-module=other']:
  label=('cargo' if cargo else 'explicit')+'-incomplete-valid-'+flag.split('=')[0][2:];d=R/label;d.mkdir();selected=d/('libprovider.vir' if cargo else 'provider.vir');shutil.copy2(R/'bool-provider/provider.vir',selected)
  require(label,app(label,'provider',BOOL,goals=True,cargo=cargo,options=(['--verify-root',flag] if flag.startswith('--verify-function') else [flag])),None,success=False);need(not selected.exists(),'incomplete publication')
