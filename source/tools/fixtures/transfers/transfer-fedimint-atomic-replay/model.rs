use vstd::prelude::*;

verus! {

pub struct Payload {
    pub item: u64,
    pub peer: u16,
}

pub struct Durable {
    pub marker: Option<Payload>,
    pub effects: Seq<Payload>,
    pub frame: int,
}

pub struct State {
    pub key: u64,
    pub disk: Durable,
    pub request: Payload,
    pub phase: u8,
    pub reply: u8,
}

pub open spec fn decision(present: bool, existing: Payload, request: Payload) -> u8 {
    if !present { 0 } else if existing == request { 1 } else { 2 }
}

pub fn replay_decision(present: bool, old_item: u64, old_peer: u16, item: u64, peer: u16)
    -> (result: u8)
    ensures result == decision(present, Payload { item: old_item, peer: old_peer },
        Payload { item, peer }),
{
    if !present { 0 } else if old_item == item && old_peer == peer { 1 } else { 2 }
}

pub open spec fn paired(disk: Durable) -> bool {
    match disk.marker {
        None => disk.effects.len() == 0,
        Some(payload) => disk.effects == seq![payload],
    }
}

pub open spec fn valid_state(state: State) -> bool {
    paired(state.disk)
    && state.phase <= 3 && state.reply <= 2
    && (state.phase > 0 ==> state.disk.marker is None && state.reply == 0)
    && (state.reply > 0 ==> state.disk.marker is Some)
}

pub open spec fn initial(key: u64, frame: int) -> State {
    State { key, disk: Durable { marker: None, effects: Seq::empty(), frame },
        request: Payload { item: 0, peer: 0 }, phase: 0, reply: 0 }
}

pub open spec fn start(state: State, request: Payload) -> State {
    match state.disk.marker {
        Some(existing) => State { request, phase: 0,
            reply: decision(true, existing, request), ..state },
        None => State { request, phase: 1, reply: 0, ..state },
    }
}

pub open spec fn stage_effects(state: State) -> State {
    State { phase: 2, ..state }
}

pub open spec fn stage_marker(state: State) -> State {
    State { phase: 3, ..state }
}

pub open spec fn abort(state: State) -> State {
    State { phase: 0, reply: 0, ..state }
}

pub open spec fn commit(state: State, takes_effect: bool, observed: bool) -> State {
    State {
        disk: if takes_effect {
            Durable { marker: Some(state.request), effects: state.disk.effects.push(state.request),
                frame: state.disk.frame }
        } else { state.disk },
        phase: 0,
        reply: if takes_effect && observed { 1 } else { 0 },
        ..state
    }
}

pub open spec fn crash(state: State) -> State {
    State { phase: 0, reply: 0, request: Payload { item: 0, peer: 0 }, ..state }
}

pub open spec fn step(before: State, after: State) -> bool {
    (before.phase == 0 && exists|request: Payload| after == start(before, request))
    || (before.phase == 1 && after == stage_effects(before))
    || (before.phase == 2 && after == stage_marker(before))
    || after == abort(before)
    || (before.phase == 3 && exists|takes_effect: bool, observed: bool|
        after == commit(before, takes_effect, observed))
    || after == crash(before)
}

pub open spec fn reachable(state: State, steps: nat, key: u64, frame: int) -> bool
    decreases steps,
{
    if steps == 0 { state == initial(key, frame) }
    else { exists|previous: State| reachable(previous, (steps - 1) as nat, key, frame)
        && step(previous, state) }
}

pub proof fn initial_invariant(key: u64, frame: int)
    ensures valid_state(initial(key, frame)),
{}

pub proof fn start_preserves(state: State, request: Payload)
    requires valid_state(state), state.phase == 0,
    ensures valid_state(start(state, request)), start(state, request).disk == state.disk,
        start(state, request).key == state.key,
{}

pub proof fn staging_preserves(state: State)
    requires valid_state(state),
    ensures
        state.phase == 1 ==> valid_state(stage_effects(state)),
        state.phase == 2 ==> valid_state(stage_marker(state)),
        stage_effects(state).disk == state.disk, stage_marker(state).disk == state.disk,
        stage_effects(state).key == state.key, stage_marker(state).key == state.key,
{}

pub proof fn abort_preserves(state: State)
    requires valid_state(state),
    ensures valid_state(abort(state)), abort(state).disk == state.disk,
        abort(state).key == state.key,
{}

pub proof fn commit_preserves(state: State, takes_effect: bool, observed: bool)
    requires valid_state(state), state.phase == 3,
    ensures valid_state(commit(state, takes_effect, observed)),
        commit(state, takes_effect, observed).key == state.key,
        commit(state, takes_effect, observed).disk.frame == state.disk.frame,
{
    assert(state.disk.effects == Seq::<Payload>::empty());
}

pub proof fn crash_preserves(state: State)
    requires valid_state(state),
    ensures valid_state(crash(state)), crash(state).disk == state.disk,
        crash(state).key == state.key,
{}

pub proof fn step_preserves(before: State, after: State)
    requires valid_state(before), step(before, after),
    ensures valid_state(after), after.key == before.key, after.disk.frame == before.disk.frame,
{
    if before.phase == 0 && exists|request: Payload| after == start(before, request) {
        let request = choose|request: Payload| after == start(before, request);
        start_preserves(before, request);
    } else if before.phase == 1 && after == stage_effects(before) {
        staging_preserves(before);
    } else if before.phase == 2 && after == stage_marker(before) {
        staging_preserves(before);
    } else if after == abort(before) {
        abort_preserves(before);
    } else if before.phase == 3 && exists|takes_effect: bool, observed: bool|
        after == commit(before, takes_effect, observed) {
        let (takes_effect, observed) = choose|takes_effect: bool, observed: bool|
            after == commit(before, takes_effect, observed);
        commit_preserves(before, takes_effect, observed);
    } else {
        crash_preserves(before);
    }
}

pub proof fn reachable_invariant(state: State, steps: nat, key: u64, frame: int)
    requires reachable(state, steps, key, frame),
    ensures valid_state(state), state.key == key, state.disk.frame == frame,
    decreases steps,
{
    if steps == 0 {
        initial_invariant(key, frame);
    } else {
        let previous = choose|previous: State|
            reachable(previous, (steps - 1) as nat, key, frame) && step(previous, state);
        reachable_invariant(previous, (steps - 1) as nat, key, frame);
        step_preserves(previous, state);
    }
}

pub proof fn recovered_pairing(state: State, steps: nat, key: u64, frame: int)
    requires reachable(state, steps, key, frame),
    ensures paired(crash(state).disk), crash(state).disk.effects.len() <= 1,
        crash(state).disk == state.disk, crash(state).phase == 0,
{
    reachable_invariant(state, steps, key, frame);
}

pub proof fn precommit_crash(state: State)
    requires valid_state(state), state.phase > 0,
    ensures crash(state).disk.marker is None, crash(state).disk.effects.len() == 0,
        crash(state).disk.frame == state.disk.frame,
{}

pub proof fn all_or_none_commit(state: State, takes_effect: bool, observed: bool)
    requires valid_state(state), state.phase == 3,
    ensures paired(crash(commit(state, takes_effect, observed)).disk),
        !takes_effect ==> crash(commit(state, takes_effect, observed)).disk == state.disk,
        takes_effect ==> crash(commit(state, takes_effect, observed)).disk.marker == Some(state.request),
        takes_effect ==> crash(commit(state, takes_effect, observed)).disk.effects == seq![state.request],
{
    commit_preserves(state, takes_effect, observed);
}

pub proof fn matching_replay_no_effect(state: State, request: Payload)
    requires valid_state(state), state.phase == 0, state.disk.marker == Some(request),
    ensures start(state, request).disk == state.disk,
        start(state, request).disk.effects.len() == 1,
        start(state, request).reply == 1, start(state, request).phase == 0,
{}

pub proof fn conflicting_replay_rejected(state: State, existing: Payload, request: Payload)
    requires valid_state(state), state.phase == 0, state.disk.marker == Some(existing),
        existing.item != request.item || existing.peer != request.peer,
    ensures start(state, request).reply == 2, start(state, request).phase == 0,
        start(state, request).disk == state.disk,
{}

pub proof fn lost_ack_retry(state: State)
    requires valid_state(state), state.phase == 3,
    ensures commit(state, true, false).reply == 0,
        start(crash(commit(state, true, false)), state.request).reply == 1,
        start(crash(commit(state, true, false)), state.request).disk
            == commit(state, true, false).disk,
        start(crash(commit(state, true, false)), state.request).disk.effects.len() == 1,
{
    commit_preserves(state, true, false);
    crash_preserves(commit(state, true, false));
    matching_replay_no_effect(crash(commit(state, true, false)), state.request);
}

pub open spec fn non_atomic_marker(state: State) -> State {
    State { disk: Durable { marker: Some(state.request), ..state.disk }, ..state }
}

pub open spec fn non_atomic_effects(state: State) -> State {
    State { disk: Durable { effects: state.disk.effects.push(state.request), ..state.disk }, ..state }
}

pub open spec fn without_dedup(state: State, request: Payload) -> State {
    State { request, phase: 1, reply: 0, ..state }
}

pub proof fn non_atomic_counterexamples(key: u64, frame: int, request: Payload)
    ensures
        !paired(crash(non_atomic_marker(start(initial(key, frame), request))).disk),
        !paired(crash(non_atomic_effects(start(initial(key, frame), request))).disk),
{}

pub proof fn missing_dedup_counterexample(key: u64, frame: int, request: Payload)
    ensures ({
        let ready = stage_marker(stage_effects(start(initial(key, frame), request)));
        let first = crash(commit(ready, true, false));
        let replay = stage_marker(stage_effects(without_dedup(first, request)));
        let second = commit(replay, true, true);
        first.disk.effects.len() == 1 && second.disk.effects.len() == 2
            && second.disk.marker == Some(request) && !paired(second.disk)
    }),
{}

}
