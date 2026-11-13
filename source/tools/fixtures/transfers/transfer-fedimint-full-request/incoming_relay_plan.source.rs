fn incoming_relay_plan(
    receive_exists: bool,
    completion_exists: bool,
    legacy_completion_exists: bool,
) -> IncomingRelayPlan {
    if completion_exists || legacy_completion_exists {
        IncomingRelayPlan::Replay
    } else if receive_exists {
        IncomingRelayPlan::AddCompletion
    } else {
        IncomingRelayPlan::CreateReceiveAndCompletion
    }
}
