#!/usr/bin/env python3
"""Additional read-only provenance checks, never executes delivered controllers."""
from review import *
import tarfile
out={}
# Verify every retained TB package source and metadata file against bound maps.
deps=load(DEL/'tree_borrows_checked_foundation/dependencies.json')
for pkg,e in deps.items():
 d={'metadata':guard(safe_artifact(e['metadata_root']),e['metadata_files'])}
 if 'source_root' in e:
  manifest=TB/'source-manifests'/(pkg+'.json');checked(manifest,e['source_manifest_sha256']);d['sources']=guard(safe_artifact(e['source_root']),load(manifest))
 out[pkg]=d
repositories={}
for name,e in load(DEL/'tree_borrows_checked_foundation/repositories.json').items():
 m=TB/(name+'-tracked.json');checked(m,e['manifest_sha256']);repositories[name]=guard(TB/'repositories'/name,load(m))
# Release archive payloads must match all delivered authenticated Git source hashes.
releases={}
for name,e in load(SB/'manifests/release-authentication.json').items():
 archive=SB/'downloads'/({'coq':'coq-V8.9.1.tar.gz','equations':'equations-v1.2.1-8.9.tar.gz'}[name]);checked(archive,e['archive_sha256']);files=e['tracked_source_sha256'];seen=set()
 with tarfile.open(archive) as tar:
  for member in tar:
   if member.isfile():
    p='/'.join(member.name.split('/')[1:]);b=tar.extractfile(member).read();assert p in files and hashlib.sha256(b).hexdigest()==files[p],p;seen.add(p)
 assert sorted(set(files)-seen)==e['export_omitted']
 assert not any(p.endswith(('.v','.ml','.mli','.c','.h')) for p in e['export_omitted'])
 releases[name]={'commit':e['commit'],'tree':e['tree'],'matched_archive_files':len(seen),'export_omitted':e['export_omitted']}
# Source/objects as recorded by every delivered kernel intern, with hash checks.
loaded=load(TB/'loaded-objects.json');guard('/',loaded)
# Both complete recorded maps are authenticated by their readiness transitive hashes.
sbmap=load(SB/'manifests/object-dependency-map.json')
save(ROOT/'provenance.json',{'tb_packages':out,'tb_repositories':repositories,'sb_release_payloads':releases,'tb_loaded_objects':len(loaded),'sb_object_map_keys':list(sbmap),'source_identity_trust':'Upstream commit/tree labels are authenticated delivered metadata, not a newly fetched independent repository. Full byte maps and release archive payloads were rehashed. Installed tools/dependencies are reused, not rebuilt.'})
print('additional provenance matched',len(deps),'TB packages',len(loaded),'TB loaded objects')
