From ModelStateObservation Require Import observation instances controls.
Goal True. idtac "BEGIN observation.heap_values". exact I. Qed.
Print Assumptions ModelStateObservation.observation.heap_values.
Goal True. idtac "END observation.heap_values". exact I. Qed.
Goal True. idtac "BEGIN observation.interval_observed". exact I. Qed.
Print Assumptions ModelStateObservation.observation.interval_observed.
Goal True. idtac "END observation.interval_observed". exact I. Qed.
Goal True. idtac "BEGIN observation.frame_observed". exact I. Qed.
Print Assumptions ModelStateObservation.observation.frame_observed.
Goal True. idtac "END observation.frame_observed". exact I. Qed.
Goal True. idtac "BEGIN observation.fragments_observe". exact I. Qed.
Print Assumptions ModelStateObservation.observation.fragments_observe.
Goal True. idtac "END observation.fragments_observe". exact I. Qed.
Goal True. idtac "BEGIN observation.interval_observe". exact I. Qed.
Print Assumptions ModelStateObservation.observation.interval_observe.
Goal True. idtac "END observation.interval_observe". exact I. Qed.
Goal True. idtac "BEGIN observation.initialized_observation_adequacy". exact I. Qed.
Print Assumptions ModelStateObservation.observation.initialized_observation_adequacy.
Goal True. idtac "END observation.initialized_observation_adequacy". exact I. Qed.
Goal True. idtac "BEGIN observation.initialized_interval_adequacy". exact I. Qed.
Print Assumptions ModelStateObservation.observation.initialized_interval_adequacy.
Goal True. idtac "END observation.initialized_interval_adequacy". exact I. Qed.
Goal True. idtac "BEGIN observation.initialized_interval_frame_adequacy". exact I. Qed.
Print Assumptions ModelStateObservation.observation.initialized_interval_frame_adequacy.
Goal True. idtac "END observation.initialized_interval_frame_adequacy". exact I. Qed.
Goal True. idtac "BEGIN observation.initialized_interior_observation". exact I. Qed.
Print Assumptions ModelStateObservation.observation.initialized_interior_observation.
Goal True. idtac "END observation.initialized_interior_observation". exact I. Qed.
Goal True. idtac "BEGIN instances.read_interval_final". exact I. Qed.
Print Assumptions ModelStateObservation.instances.read_interval_final.
Goal True. idtac "END instances.read_interval_final". exact I. Qed.
Goal True. idtac "BEGIN instances.write_interval_final". exact I. Qed.
Print Assumptions ModelStateObservation.instances.write_interval_final.
Goal True. idtac "END instances.write_interval_final". exact I. Qed.
Goal True. idtac "BEGIN instances.write_read_interval_final". exact I. Qed.
Print Assumptions ModelStateObservation.instances.write_read_interval_final.
Goal True. idtac "END instances.write_read_interval_final". exact I. Qed.
Goal True. idtac "BEGIN instances.head_return_execution". exact I. Qed.
Print Assumptions ModelStateObservation.instances.head_return_execution.
Goal True. idtac "END instances.head_return_execution". exact I. Qed.
Goal True. idtac "BEGIN instances.stored_state". exact I. Qed.
Print Assumptions ModelStateObservation.instances.stored_state.
Goal True. idtac "END instances.stored_state". exact I. Qed.
Goal True. idtac "BEGIN instances.store_execution". exact I. Qed.
Print Assumptions ModelStateObservation.instances.store_execution.
Goal True. idtac "END instances.store_execution". exact I. Qed.
Goal True. idtac "BEGIN instances.stored_state_lookup". exact I. Qed.
Print Assumptions ModelStateObservation.instances.stored_state_lookup.
Goal True. idtac "END instances.stored_state_lookup". exact I. Qed.
Goal True. idtac "BEGIN controls.returned_value_cannot_disagree". exact I. Qed.
Print Assumptions ModelStateObservation.controls.returned_value_cannot_disagree.
Goal True. idtac "END controls.returned_value_cannot_disagree". exact I. Qed.
Goal True. idtac "BEGIN controls.explicit_length_refuses_empty". exact I. Qed.
Print Assumptions ModelStateObservation.controls.explicit_length_refuses_empty.
Goal True. idtac "END controls.explicit_length_refuses_empty". exact I. Qed.
Goal True. idtac "BEGIN controls.same_return_different_memory". exact I. Qed.
Print Assumptions ModelStateObservation.controls.same_return_different_memory.
Goal True. idtac "END controls.same_return_different_memory". exact I. Qed.
Goal True. idtac "BEGIN controls.value_only_does_not_imply_unchanged". exact I. Qed.
Print Assumptions ModelStateObservation.controls.value_only_does_not_imply_unchanged.
Goal True. idtac "END controls.value_only_does_not_imply_unchanged". exact I. Qed.
