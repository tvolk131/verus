#!/usr/bin/env python3
"""Pure source identity audit; never an interpreter simulation."""
import argparse
import importlib.util
import json
from pathlib import Path
import sys
sys.dont_write_bytecode = True
HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[1]
MODEL = ROOT/'source/tools/research/native_source_storage'
sys.path.insert(0, str(ROOT/'source/tools'))
from infer_eval_support import ArtifactGuard, digest
from infer_experiment import require


def main():
    p = argparse.ArgumentParser()
    p.add_argument('--output', type=Path, required=True)
    a = p.parse_args()
    pins = json.loads((MODEL/'pins.json').read_text())
    graph_path = MODEL.parent/'native_constructor_erasure/graph.json'
    extraction = Path('/tmp/native-semantic-bridge-e4a8dcab/source-v1/drop-native.json')
    paths = [Path(__file__), MODEL/'pins.json', MODEL/'source.py', graph_path,
        MODEL/'candidates/Sites.v', *[Path(p) for p in pins['completed_extraction']]]
    guard = ArtifactGuard(paths)
    for name, h in pins['completed_extraction'].items():
        require(digest(name) == h, 'extraction identity')
    require(digest(graph_path) == pins['committed_inputs'][str(graph_path.relative_to(ROOT))], 'graph identity')
    native = json.loads(extraction.read_text())
    graph = json.loads(graph_path.read_text())
    require(native['pointer_bits'] == 64 and len(native['instances']) == 6 and
        not native['extraction_blockers'] and not native['pending_instances'], 'incomplete extraction')
    rows = []
    for i, inst in enumerate(native['instances']):
        for block in inst['blocks']:
            for index, ir in [*[(n,s['ir']) for n,s in enumerate(block['statements'])], (None,block['term_ir'])]:
                rows.append(dict(instance=i, instance_identity=inst['id'], bb=block['bb'],
                    index=index, cleanup=block['cleanup'], ir=ir))
    projected = [{k:r[k] for k in ['instance','instance_identity','bb','index','cleanup','ir']}
        for r in graph['operations']]
    require(len(rows) == 44 == native['operations'] and rows == projected, 'graph/extraction row mismatch')
    root = native['instances'][0]
    require(root['id'] == native['root'], 'root identity')
    locals_ = {r['local']:r for r in root['locals']}
    require(locals_[1]['ty'] == '&mut [u8; 128]', 'incoming original reference type')
    require([locals_[i]['layout']['size'] for i in [2,3,4]] == [16,16,8], 'root sized locals')
    require([r['ir'] for r in rows[:3]] == [dict(local=i,op='storage_live') for i in [2,3,4]], 'prefix sites')
    require(rows[3]['ir']['value']['op'] == 'ref' and
        rows[3]['ir']['value']['place']['local'] == 1 and rows[3]['ir']['destination']['local'] == 4, 'next reborrow anchor')
    sites = [r for r in rows if r['ir']['op'] in ['storage_live','storage_dead']]
    require(len(sites) == 20 and all(r['ir']['local'] not in [0,1] for r in sites), 'storage target inventory')
    spec = importlib.util.spec_from_file_location('review_storage_sites', MODEL/'source.py')
    source = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(source)
    require(source.sites_source(graph) == (MODEL/'candidates/Sites.v').read_text(), 'Sites regeneration')
    guard.check()
    result = dict(success=True, classification='identity_only_not_simulation', inputs=guard.initial,
        graph_rows_matched=44, storage_sites_matched=20, root_prefix_locals=[2,3,4],
        root_layout_sizes=[16,16,8], no_storage_site_targets_original_local1=True,
        root_storage_dead_sites=[[0,0,5,4],[0,1,0,3],[0,1,2,2]],
        native_correspondences=0, publication=False, goal_complete=False)
    # This auxiliary audit writes only a fresh private receipt.
    require(a.output.resolve().is_relative_to(Path('/tmp')), 'private output required')
    with a.output.open('x') as stream:
        stream.write(json.dumps(result, indent=2)+'\n')
    print(json.dumps(dict(success=True, receipt=str(a.output))))


if __name__ == '__main__':
    main()
