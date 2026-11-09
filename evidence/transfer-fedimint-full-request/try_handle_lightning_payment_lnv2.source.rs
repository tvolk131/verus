async fn try_handle_lightning_payment_lnv2(
        &self,
        htlc_request: &InterceptPaymentRequest,
        lightning_context: &LightningContext,
    ) -> Result<()> {
        // If `payment_hash` has been registered as a LNv2 payment, we try to complete
        // the payment by getting the preimage from the federation
        // using the LNv2 protocol. If the `payment_hash` is not registered,
        // this payment is either a legacy Lightning payment or the end destination is
        // not a Fedimint.
        // Match and fund against the amount actually locked in the incoming
        // HTLC, not the sender-controlled onion forward amount, so a forged
        // `amt_to_forward` cannot satisfy the registered contract's amount
        // check while only a token amount is really locked.
        let (contract, client) = self
            .get_registered_incoming_contract_and_client_v2(
                PaymentImage::Hash(htlc_request.payment_hash),
                htlc_request.incoming_amount_msat,
            )
            .await?;

        // An intercepted forward, or an unknown current height, leaves no time
        // to fund a fresh contract; see `lnv2_blocks_to_claim_deadline`.
        let current_block_height = lightning_context
            .lnrpc
            .info()
            .await
            .ok()
            .map(|info| info.block_height);
        let blocks_to_claim_deadline =
            htlc_request.lnv2_blocks_to_claim_deadline(current_block_height);

        if let Err(err) = client
            .get_first_module::<GatewayClientModuleV2>()
            .expect("Must have client module")
            .relay_incoming_htlc(
                htlc_request.payment_hash,
                htlc_request.incoming_chan_id,
                htlc_request.htlc_id,
                contract,
                htlc_request.incoming_amount_msat,
                blocks_to_claim_deadline,
            )
            .await
        {
            warn!(
                target: LOG_GATEWAY,
                err = %err.fmt_compact(),
                "Error relaying incoming lightning payment"
            );

            let outcome = InterceptPaymentResponse {
                action: PaymentAction::Cancel,
                payment_hash: htlc_request.payment_hash,
                incoming_chan_id: htlc_request.incoming_chan_id,
                htlc_id: htlc_request.htlc_id,
            };

            if let Err(err) = lightning_context.lnrpc.complete_htlc(outcome).await {
                warn!(target: LOG_GATEWAY, err = %err.fmt_compact(), "Error sending HTLC response to lightning node");
            }
        }

        Ok(())
    }
