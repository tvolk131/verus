"""Pinned source/graph identity binding, not a semantic proof."""
import json
from pathlib import Path
HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[3]
GRAPH = HERE.parent / 'native_constructor_erasure/graph.json'


def sites_source(graph):
    rows = [r for r in graph['operations'] if r['ir']['op'] in ('storage_live', 'storage_dead')]
    def event(r):
        return f"MirOperation {r['instance']}%nat {r['bb']}%nat (Some {r['index']}%nat)"
    result = '''From Coq Require Import List.
From NativeConstructorErasure Require Import Interface Graph.
Import ListNotations.
(* Generated from the pinned six-instance/44-operation graph. Identity only. *)
Definition storage_sites : list (event * (bool * nat)) := [
'''
    result += ';\n'.join('  ('+event(r)+', ('+str(r['ir']['op']=='storage_live').lower()+', '+str(r['ir']['local'])+'%nat))' for r in rows)
    result += ''' ].
Definition storage_at (e : event) : option (bool * nat) :=
  match e with
'''
    result += ''.join('  | '+event(r).replace('%nat','')+' => Some ('+str(r['ir']['op']=='storage_live').lower()+', '+str(r['ir']['local'])+'%nat)\n' for r in rows)
    result += '  | _ => None\n  end.\n'
    result += f'Lemma storage_sites_count : length storage_sites = {len(rows)}%nat.\nProof. reflexivity. Qed.\n'
    result += r'''Lemma storage_sites_in_graph : forall e b l,
  In (e,(b,l)) storage_sites -> In e graph_events.
Proof. intros e b l E. simpl in E. repeat destruct E as [E|E]; try contradiction;
  inversion E; subst; simpl; intuition congruence. Qed.
Lemma root_live_sites :
  storage_at (MirOperation 0 0 (Some 0)) = Some (true,2%nat) /\
  storage_at (MirOperation 0 0 (Some 1)) = Some (true,3%nat) /\
  storage_at (MirOperation 0 0 (Some 2)) = Some (true,4%nat).
Proof. repeat split; reflexivity. Qed.
Lemma root_dead_sites :
  storage_at (MirOperation 0 0 (Some 5)) = Some (false,4%nat) /\
  storage_at (MirOperation 0 1 (Some 0)) = Some (false,3%nat) /\
  storage_at (MirOperation 0 1 (Some 2)) = Some (false,2%nat).
Proof. repeat split; reflexivity. Qed.
'''
    return result

if __name__ == '__main__':
    (HERE/'candidates/Sites.v').write_text(sites_source(json.loads(GRAPH.read_text())))
