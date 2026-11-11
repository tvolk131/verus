#!/usr/bin/env python3
"""Focused S2 R2 replay using existing execution/fidelity guards; no acceptance claim."""
import importlib.util
import json
import os
from pathlib import Path
import shutil
import sys

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parents[2]))
from infer_experiment import (LoggedCommands, replay_receipt, require, require_exact,
                              require_publication, require_failure_kind)
from infer_eval_support import ArtifactGuard, digest, support_artifacts
spec = importlib.util.spec_from_file_location('r2_replay', HERE.parent / 'redb_tree/replay.py')
r2 = importlib.util.module_from_spec(spec)
spec.loader.exec_module(r2)


def main():
    runtime, out = map(lambda p: Path(p).resolve(), sys.argv[1:])
    require(str(out).startswith('/tmp/practical-s2-'), 'private unique output required')
    out.mkdir(mode=0o700, exist_ok=False)
    receipt = dict(layer='S2', accepted=False, application_property_proved=False, commands=[], outcomes=[])
    with replay_receipt(out, receipt):
        meta = json.loads((HERE.parent / 'redb_tree/private.json').read_text())
        tools = [runtime / n for n in meta['runtime']['tools']]
        require(all(p.is_file() for p in tools), 'matching complete runtime required')
        receipt['runtime_sha256'] = {p.name: digest(p) for p in tools}
        fixture = HERE / 's2-reduced.rs'
        require(digest(fixture) == 'f6650e723717ff03e41b3e1533ff998430ea190e91cac54023e9320da5488a20', 'S2 freeze changed')
        receipt['fidelity'] = r2.fidelity(meta, fixture)
        inputs = [fixture, HERE / 's2-replay.py', *[r2.HERE / n for n in ('controls.rs','interface.rs','callers.rs','private.json','manual.rs','corrected.rs','replay.py')]]
        guard = ArtifactGuard([*tools, *inputs, *support_artifacts()])
        receipt['source_sha256'] = {str(p):digest(p) for p in inputs}
        env = dict(os.environ, RUSTUP_HOME='/home/dev/.rustup', RUSTUP_TOOLCHAIN='1.99.1',
                   PATH='/home/dev/.cargo/bin:/usr/bin:/bin', VERUS_Z3_PATH=str(runtime/'z3'),
                   VERUS_INFER_CACHE=str(out/'cache'))
        run = LoggedCommands(out, out, receipt['commands'], timeout=180, guard=guard)
        proofs = [p for p in r2.PROOF if p not in ('word_bits','word_frame')]

        def check(label, source, success, *, flags=(), execs=r2.EXEC, proof=proofs, failures=(), artifact=None, generated=None):
            r2.fidelity(meta, source) if 'U64GroupedBitmap' in source.read_text() else None
            sg = ArtifactGuard([source])
            code = run(label, [str(runtime/'verus'), str(source), '--edition=2024', '--no-cheating',
                              '--output-json', '--multiple-errors', '20', *flags], env)
            sg.check()
            data = json.loads((out/(label+'.stdout')).read_text())
            err = (out/(label+'.stderr')).read_text(); inf = data['verus-infer']; v = data['verification-results']; scope = inf['verification_scope']
            require((code == 0 and v.get('success') is True) is success, 'unexpected outcome: '+label)
            require(not v.get('encountered-vir-error') and not any(t in err.lower() for t in ('resource limit','panicked at','timed out','error[e')), 'frontend/resource failure: '+label)
            require_exact(inf.get('functions', [])); require(not inf.get('functions', []), 'nonempty inferred helper set')
            require(scope['native_collection_completed'], 'incomplete native collection')
            if success:
                require(scope['complete_selected_scope_verified'], 'incomplete selected scope')
                for kind, names in [('exec',execs),('proof',proof)]:
                    actual = [b['rust_function'] for b in scope['native_functions'] if b['body_status']=='checked_'+kind+'_body']
                    require(sorted(actual)==sorted(names), 'body scope mismatch: '+label+':'+str(actual))
            else:
                require_failure_kind(err, 'postcondition not satisfied')
                actual = [n.split('::', 1)[-1] for n in inf['failed_functions']]
                expected = list(failures)
                require(sorted(actual)==sorted(expected), 'failure identities: '+label+':'+str(actual))
                require(not any(t in err for t in ('possible arithmetic underflow/overflow','invariant not satisfied','recommendation not met')), 'mixed failure')
            frames = inf.get('word_sequence_assistance', [])
            if generated is not None:
                require(len(frames)==generated, 'generated frame count: '+label)
                require(sorted(x['function'].split('::',1)[-1] for x in frames)==([] if generated==0 else ['U64GroupedBitmap::clear','U64GroupedBitmap::set']), 'generated identities: '+label)
                require(all(x['snapshots']==x['quantified_blocks']==x['library_calls']==1 for x in frames), 'generated inventory')
            if artifact: require_publication(data, success, artifact)
            receipt['outcomes'].append(dict(label=label, success=success, verification=v, scope=scope,
                failed_functions=inf['failed_functions'], word_sequence_assistance=frames, inferred_helpers=[],
                stdout_sha256=digest(out/(label+'.stdout')), stderr_sha256=digest(out/(label+'.stderr'))))
            return data

        component = out/'component.rs'; shutil.copyfile(fixture, component)
        check('off', component, False, failures=['U64GroupedBitmap::set','U64GroupedBitmap::clear'], generated=0)
        check('cold', component, True, flags=['--infer-goals','--compile','-o',str(out/'component')], generated=2)
        check('repeat', component, True, flags=['--infer-goals'], generated=2)
        controls=out/'controls.rs'; controls.write_text(component.read_text()+(r2.HERE/'controls.rs').read_text())
        for kind,name in [('leaf','wrong_set_membership'),('count','wrong_repeat_count'),('frame','wrong_clear_frame')]:
            vir=out/('negative_'+kind+'.vir')
            check('negative_'+kind,controls,False,flags=['--infer-goals','--cfg','negative_'+kind,'--export',str(vir)],
                  execs=[*r2.EXEC,name],failures=[name],artifact=vir,generated=2)
        api=out/'api.rs';api.write_text(component.read_text()+(r2.HERE/'interface.rs').read_text())
        vir=out/'api.vir';lib=out/'libapi.rlib'
        check('provider',api,True,flags=['--infer-goals','--compile','--crate-type=lib','--crate-name=r2_api','--export',str(vir),'-o',str(lib)],
              execs=[*r2.EXEC,'CheckedBitmap::mark_allocated','CheckedBitmap::mark_free'],artifact=vir,generated=2)
        pg=ArtifactGuard([vir,lib]); consumer=out/'callers.rs';shutil.copyfile(r2.HERE/'callers.rs',consumer)
        for label,extra in [('ordinary',[]),('inferred',['--infer-goals'])]:
            check('import_'+label,consumer,True,flags=['--crate-type=lib','--extern','r2_api='+str(lib),'--import','r2_api='+str(vir),*extra],
                  execs=['record_alloc_membership','free_membership','repeat_alloc_membership'],proof=[],generated=0)
            pg.check()
        # A semantic failure must remove an existing successful export.
        old = out/'stale.vir'; shutil.copyfile(vir, old)
        check('stale_export',controls,False,flags=['--infer-goals','--cfg','negative_leaf','--export',str(old)],
              execs=[*r2.EXEC,'wrong_set_membership'],failures=['wrong_set_membership'],artifact=old,generated=2)
        # Mutate only the ordinary inference trailer, retaining the authenticated
        # healthy provider and rlib. Import policy/precision failures are frontend
        # controls, never semantic counterexamples.
        original = vir.read_bytes(); start = original.rfind(b'{"schema":')
        require(start >= 0 and original.endswith(b'VRINFER1'), 'inference trailer missing')
        metadata = json.loads(original[start:-8]); require(metadata['proofs']==[], 'expected empty inferred helper evidence')
        for label,key,value in [('policy','policy','candidate-bound-evidence-v75'),('precision','exactness',False),('schema','schema',38)]:
            changed = dict(metadata); changed[key]=value
            stale=out/(label+'.vir'); stale.write_bytes(original[:start]+json.dumps(changed,separators=(',',':')).encode()+b'VRINFER1')
            artifact=out/(label+'_consumer.vir')
            code=run('reject_'+label,[str(runtime/'verus'),str(consumer),'--edition=2024','--no-cheating','--crate-type=lib',
                '--extern','r2_api='+str(lib),'--import','r2_api='+str(stale),'--export',str(artifact)],env)
            err=(out/('reject_'+label+'.stderr')).read_text()
            require(code != 0 and not artifact.exists() and 'Verus Infer' in err, 'metadata rejection missing: '+label)
            require(not any(t in err.lower() for t in ('panicked at','resource limit','timed out')), 'metadata control internal/resource failure')
            receipt['outcomes'].append(dict(label='reject_'+label,kind='frontend_metadata_control',success=False,stderr_sha256=digest(out/('reject_'+label+'.stderr'))))
            pg.check()
        receipt['application_property_proved']=True
        guard.check()
    print(out)

if __name__ == '__main__':
    main()
