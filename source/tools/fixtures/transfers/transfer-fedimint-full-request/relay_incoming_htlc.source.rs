pub async fn relay_incoming_htlc(
        &self,
        payment_hash: sha256::Hash,
        incoming_chan_id: u64,
        htlc_id: u64,
        contract: IncomingContract,
        amount_msat: u64,
        blocks_to_claim_deadline: u32,
    ) -> Result<(), RelayIncomingHtlcError> {
        let operation_start = now();
        let receive_operation_id = OperationId::from_encodable(&contract);
        let circuit = IncomingCircuitKey {
            incoming_chan_id,
            htlc_id,
        };
        let completion_operation_id = incoming_circuit_operation_id(receive_operation_id, circuit);
        let receive_exists = self.client_ctx.operation_exists(receive_operation_id).await;
        let completion_exists = self
            .client_ctx
            .operation_exists(completion_operation_id)
            .await;
        let legacy_completion_exists = self
            .legacy_completion_exists(receive_operation_id, circuit)
            .await;
        let plan = incoming_relay_plan(receive_exists, completion_exists, legacy_completion_exists);
        if plan == IncomingRelayPlan::Replay {
            return Ok(());
        }

        let commitment = contract.commitment.clone();
        if plan == IncomingRelayPlan::CreateReceiveAndCompletion {
            // Only gate fresh funding: the other plans resume an already
            // funded contract and must not be cancelled.
            //
            // Funding is irreversible, and the gateway is only reimbursed by
            // settling the HTLC before its claim deadline, so do not fund an
            // HTLC that is about to expire. The margin is small because LDK's
            // default invoices leave only 3 blocks before the claim deadline.
            if blocks_to_claim_deadline < LNV2_CLAIM_DEADLINE_MARGIN {
                return Err(RelayIncomingHtlcError::ClaimDeadlineTooClose {
                    blocks_to_claim_deadline,
                });
            }
            self.ensure_federation_responsive(payment_hash).await?;

            let refund_keypair = self.keypair;
            let client_output = ClientOutput::<LightningOutput> {
                output: LightningOutput::V0(LightningOutputV0::Incoming(contract.clone())),
                amounts: Amounts::new_bitcoin(contract.commitment.amount),
            };
            let client_output_sm = ClientOutputSM::<GatewayClientStateMachinesV2> {
                state_machines: Arc::new(move |range: OutPointRange| {
                    assert_eq!(range.count(), 1);

                    vec![GatewayClientStateMachinesV2::Receive(ReceiveStateMachine {
                        common: ReceiveSMCommon {
                            operation_id: receive_operation_id,
                            contract: contract.clone(),
                            outpoint: range.into_iter().next().unwrap(),
                            refund_keypair,
                        },
                        state: ReceiveSMState::Funding,
                    })]
                }),
            };

            let client_output = self.client_ctx.make_client_outputs(ClientOutputBundle::new(
                vec![client_output],
                vec![client_output_sm],
            ));
            let transaction = TransactionBuilder::new().with_outputs(client_output);

            let creation_result = self
                .client_ctx
                .finalize_and_submit_transaction(
                    receive_operation_id,
                    LightningCommonInit::KIND.as_str(),
                    |_| GatewayOperationMetaV2::role(GatewayOperationRoleV2::Receive),
                    transaction,
                )
                .await;
            if let Err(error) = creation_result {
                let operation_exists = self.client_ctx.operation_exists(receive_operation_id).await;
                if operation_creation_failed_permanently(true, operation_exists) {
                    return Err(error.into());
                }
            } else {
                let mut dbtx = self.client_ctx.module_db().begin_transaction().await;
                self.client_ctx
                    .log_event(
                        &mut dbtx,
                        IncomingPaymentStarted {
                            operation_start,
                            incoming_contract_commitment: commitment,
                            invoice_amount: Amount::from_msats(amount_msat),
                        },
                    )
                    .await;
                dbtx.commit_tx().await;
            }
        }

        let completion =
            GatewayClientStateMachinesV2::CircuitComplete(CircuitCompleteStateMachine {
                common: CircuitCompleteSMCommon {
                    operation_id: completion_operation_id,
                    receive_operation_id,
                    payment_hash,
                    circuit,
                },
                state: CompleteSMState::Pending,
            });
        let creation_result = self
            .client_ctx
            .manual_operation_start(
                completion_operation_id,
                LightningCommonInit::KIND.as_str(),
                GatewayOperationMetaV2::role(GatewayOperationRoleV2::CircuitCompletion),
                vec![self.client_ctx.make_dyn_state(completion)],
            )
            .await;
        if let Err(error) = creation_result {
            let operation_exists = self
                .client_ctx
                .operation_exists(completion_operation_id)
                .await;
            if operation_creation_failed_permanently(true, operation_exists) {
                return Err(error.into());
            }
        }

        Ok(())
    }
