"""Read only exact completed allowed inputs; historical paths are labels."""
from build import *
def audit():
 old=ROOT/'evidence/local-nonzero-ord-parent-review';val=json.loads((old/'VALIDATION.json').read_text());res=json.loads((old/'RESULTS.json').read_text());private=Path('/tmp/local-nonzero-ord-09t1tshb')
 for n,key in [('REPORT.md','report_sha256'),('RESULTS.json','results_sha256'),('SOURCE_REVIEW.md','source_review_sha256')]:need(sha(old/n)==val[key],'committed prior '+n)
 freeze=json.loads((private/'bundle-final-tools.json').read_text());need(sha(private/'bundle-final-tools.json')==res['freeze_sha256'],'prior tools freeze')
 for n,h in freeze['tools'].items():need(sha(private/'bundle'/n)==h,'prior allowed tool '+n)
 for label in ['review-dual-valid-prior','review-dual-valid-refusal','review-type-dependency-actual-function','review-full-proofless-export','review-packet-export']:
  need(sha(private/label/'receipt.json')==res['runs'][label]['receipt_sha256'],'prior receipt '+label)
 need(sha(private/'dual-valid-retained-prior.vir')==res['dual_selected_output_finding']['before_sha256'],'prior genuine Bool')
 for n,key in [('type-dependency-control-result.json','structural_variant'),('dual-valid-result.json','dual_selected_output_finding')]:need(json.loads((private/n).read_text())==res[key],'prior finding '+n)
 # Rebind exact source using OWN Git; never read historical checkout labels.
 base='913044c43a476188537a087618a205e01aafc9eb';fix='297332c265e85bcc0fbcaac253277f307906c3f2'
 files=subprocess.check_output(['git','diff-tree','--no-commit-id','--name-only','-r',fix],cwd=ROOT).decode().splitlines()
 need(set(files)=={'source/rust_verify/src/checked_nonzero_ord.rs','source/rust_verify/src/checked_nonzero_ord_tests.rs','source/rust_verify/src/import_export.rs'},'fix scope')
 for p in manifest(ROOT):need((ROOT/p).read_bytes()==subprocess.check_output(['git','show',base+':'+p],cwd=ROOT),'prepared source identity '+p)
 need(subprocess.check_output(['git','diff',fix,base,'--','source','dependencies','tools','rust-toolchain.toml'],cwd=ROOT)==b'','no additional production changes')
 core=res['compiler_core']
 for p,h in core.items():need(sha(p)==h,'pinned compiler/core')
 s=(ROOT/'source/rust_verify/src/checked_nonzero_ord.rs').read_text();need('types_equal(' not in s,'no weak local type comparator')
 # TypX/ImplPath derive serde without skipped fields; nominal and witness identity both retained.
 return dict(prepared_head=base,fix=fix,fix_files={p:sha(ROOT/p) for p in files},compiler_core=core,prior_results_sha256=sha(old/'RESULTS.json'),prior_validation_sha256=sha(old/'VALIDATION.json'),prior_tools_authenticated=True,versions=dict(schema=39,policy=73,cache=76),item_fixture_sha256='1adb8b6c2ee64cc1c6930c9530920105ad26cee8abf3212caff53dda0c36518d')
if __name__=='__main__':
 j=audit()
 if '--write' in sys.argv:write(HERE/'SOURCE_AUTHENTICATION.json',j)
 else:need(j==json.loads((HERE/'SOURCE_AUTHENTICATION.json').read_text()),'authentication drift')
