from trials import *
need(sys.flags.dont_write_bytecode,'use -B')
for cargo in [False,True]:
 label=('cargo' if cargo else 'explicit')+'-failed-bv-auxiliary';d=R/label;d.mkdir(exist_ok=True);selected=d/('libprovider.vir' if cargo else 'provider.vir')
 if not (d/'receipt.json').exists():shutil.copy2(R/'bool-provider/provider.vir',selected)
 source=BOOL.replace('{u64::from(b)}','{let x=u64::from(b); assert((0u64 | (1u64 << 1u64)) == 0u64) by(bit_vector); x}')
 rec=json.loads((d/'receipt.json').read_text()) if (d/'receipt.json').exists() else app(label,'provider',source,goals=True,cargo=cargo);require(label,rec,None,success=False)
 need(not selected.exists(),'failed ordinary arithmetic auxiliary published')
 text=(d/'stderr').read_text();need('bit_vector' in text or 'bitvector' in text,'expected isolated BV failure')
 j=json.loads((d/'stdout').read_text());v=j['verification-results'];need(v['errors']>0 and v['verified']>=1,'healthy helper and failed auxiliary')
 air=(d/'vir/root-final.air').read_text();a=air.index(';; Function-Def provider::checked_foreign_bool_to_u64%\n');b=air.find('\n;; Function-',a+1);helper=air[a:b if b>=0 else len(air)]
 need('(check-valid' in helper,'actual helper query missing')

 need(j['verus-infer'].get('failed_functions')==[],'auxiliary-only failed_functions must be empty')
 need(v['verified']==10 and v['errors']==1,'exact isolated auxiliary failure10/1')
