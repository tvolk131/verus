from build import *
need(sys.flags.dont_write_bytecode,'use -B')
r=root()
if sys.argv[1]=='focused':
 j=run('focused-test-build',[TC/'bin/cargo','test','--offline','--locked','-j','2','-p','rust_verify_test','--no-run','--message-format=json',*sum((['--test',t] for t in ['certified_foreign_bool','local_checked_foreign','local_checked_packing','infer_goals','infer_nonzero_derive','constructor_function_values']),[])])
else:
 j=run('unit-build-03',[TC/'bin/cargo','test','--offline','--locked','-j','2','-p','vir','-p','rust_verify','--lib','--no-run','--message-format=json'])
need(j['exit']==0,'test build failed')
