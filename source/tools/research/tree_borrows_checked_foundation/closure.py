"""Compute transitive internal compilation targets from actual rocq dep output."""
import argparse, json, pathlib, shlex
ROOTS=['tree_borrows/bor_semantics','tree_borrows/defs','tree_borrows/steps_wf','tree_borrows/steps_progress','tree_borrows/step_laws/steps_retag','tree_borrows/adequacy','simulation/slsls','simulation/lifting']
p=argparse.ArgumentParser();p.add_argument('root',type=pathlib.Path);p.add_argument('dependencies',type=pathlib.Path);p.add_argument('output',type=pathlib.Path);a=p.parse_args()
graph={}
for line in a.dependencies.read_text().replace('\\\n',' ').splitlines():
    if ':' not in line: continue
    left,right=line.split(':',1)
    targets=[x for x in shlex.split(left) if x.endswith('.vo')]
    deps=[x for x in shlex.split(right) if x.endswith('.vo')]
    for target in targets: graph[target]=deps
seen=set();external=set();order=[]
def visit(node):
    if node in seen:return
    if node not in graph:external.add(node);return
    seen.add(node)
    for dep in graph[node]:visit(dep)
    order.append(node)
for root in ROOTS:visit('theories/'+root+'.vo')
for root in ROOTS:
    if 'theories/'+root+'.vo' not in seen:raise SystemExit('missing dependency root '+root)
result={'roots':['simuliris.'+x.replace('/','.') for x in ROOTS], 'generator':'rocq dep -f _RocqProject', 'targets':order,'modules':['simuliris.'+x[len('theories/'):-3].replace('/','.') for x in order], 'direct_dependencies':{x:graph[x] for x in order},'external_vo_paths':sorted(external)}
a.output.write_text(json.dumps(result,indent=2)+'\n');print(len(order),'internal modules;',len(external),'external direct paths')
