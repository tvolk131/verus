"""Prepare normal Rocq queries of checked modules and their exact assumptions.

The raw Print Module results inventory constructors, eliminators, projections,
nested module signatures and generated helpers as well as ordinary constants.
No source regex is used as a substitute for the compiled declaration inventory.
"""
import argparse,json,pathlib,re
p=argparse.ArgumentParser();p.add_argument('root',type=pathlib.Path);a=p.parse_args();r=a.root
closure=json.loads((r/'closure.json').read_text());log=(r/'jobs/kernel.log').read_text()
constants=sorted(set(re.findall(r'checking cst:([^\s]+)',log)))
modules=sorted(set(re.findall(r'checking module: ([^\s]+)',log)))
public=[c for c in constants if c.startswith('simuliris.')]
# Kernel includes instantiated functor constants as well. Public qualified names
# are independently inventoried by Print Module; no missing query is ignored.
(r/'query-results').mkdir(exist_ok=True)
lines=['From simuliris.tree_borrows Require Import bor_semantics defs steps_wf steps_progress adequacy.',
       'From simuliris.tree_borrows.step_laws Require Import steps_retag.',
       'From simuliris.simulation Require Import slsls lifting.',
       'Set Printing Width 1000000.','Set Printing Depth 1000000.','Set Printing All.']
queries=[]
for i,m in enumerate(closure['modules']):
    f=r/'query-results'/('module-%05d'%i)
    lines.append('Redirect "'+str(f)+'" Print Module '+m+'.')
for i,c in enumerate(public):
    f=r/'query-results'/('constant-%05d'%i)
    lines.extend(['Redirect "'+str(f)+'-type" Check @'+c+'.','Redirect "'+str(f)+'-assumptions" Print Assumptions '+c+'.'])
    queries.append({'name':c,'type':str(f)+'-type.out','assumptions':str(f)+'-assumptions.out'})
(r/'inventory.v').write_text('\n'.join(lines)+'\n')
(r/'inventory-queries.json').write_text(json.dumps({'all_kernel_constants':constants,'all_kernel_modules':modules,'internal_constants':queries,'compiled_modules':closure['modules']},indent=2)+'\n')
print(len(constants),'total constants;',len(public),'internal constant queries;',len(closure['modules']),'module queries')
