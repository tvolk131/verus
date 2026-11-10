"""Exact typed-source selection. This supplies identity, never Rust semantics."""
import json
from pathlib import Path

HERE = Path(__file__).resolve().parent
GRAPH = HERE.parent / 'native_constructor_erasure/graph.json'
EXPECTED = {
    'destination': {'local': 3, 'projection': [], 'ty': '&mut [u8]'},
    'op': 'assign',
    'value': {
        'kind': 'Unsize', 'op': 'cast',
        'operand': {'op': 'move', 'place': {
            'local': 4, 'projection': [], 'ty': '&mut [u8; 128]'}},
        'to': '&mut [u8]',
    },
}


def source_text(graph):
    rows = [r for r in graph['operations']
            if (r['instance'], r['bb'], r['index']) == (0, 0, 4)]
    if len(rows) != 1 or rows[0]['ir'] != EXPECTED:
        raise ValueError('frozen typed unsizing operation changed')
    return '''From Coq Require Import List.
From NativeConstructorErasure Require Import Interface Graph.
From NativeSourceStorage Require Import Storage.
From NativeSourceUnsize Require Import Carrier.
Import ListNotations.
(* Generated source identity only; the complete Rust statement is unproved. *)
Definition root_unsize_site := MirOperation 0 0 (Some 4).
Lemma selected_site_in_graph : In root_unsize_site graph_events.
Proof. simpl. intuition congruence. Qed.
Lemma root_carrier_progress H (s : state H) r :
  locals s 4 = Some (Immediate (Init (RefValue r))) ->
  locals s 3 = Some (Immediate Uninit) ->
  carrier_store s 4 3 128 =
    Some (set_local s 3 (Immediate (Init (SliceValue r 128)))).
Proof. intros. apply byte128_carrier; assumption || discriminate. Qed.
Lemma root_still_pending H (s t : state H) :
  current_event s = root_unsize_site ->
  carrier_store s 4 3 128 = Some t -> current_event t = root_unsize_site.
Proof.
  intros S E. destruct (carrier_pending_control H s t 4 3 128 E) as [P _].
  unfold current_event. rewrite P. exact S.
Qed.
Lemma source_dead_does_not_discard_payload H (s t u : state H) :
  carrier_store s 4 3 128 = Some t -> storage_dead t 4 = Some u ->
  exists r, locals u 3 = Some (Immediate (Init (SliceValue r 128))) /\\
    locals u 4 = Some Dead /\\ anchors u = anchors s.
Proof.
  intros E D. destruct (carrier_destination H s t 4 3 128 E) as [r [S T]].
  exists r. split.
  - rewrite (dead_other H t u 4 3 D); [exact T|discriminate].
  - split; [exact (dead_local H t u 4 D)|].
    rewrite (dead_anchors H t u 4 D). exact (carrier_anchors H s t 4 3 128 E).
Qed.
'''


if __name__ == '__main__':
    (HERE/'candidates/Root.v').write_text(source_text(json.loads(GRAPH.read_text())))
