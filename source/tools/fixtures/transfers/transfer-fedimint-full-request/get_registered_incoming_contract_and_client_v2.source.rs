pub async fn get_registered_incoming_contract_and_client_v2(
        &self,
        payment_image: PaymentImage,
        amount_msats: u64,
    ) -> Result<(IncomingContract, ClientHandleArc)> {
        let registered_incoming_contract = self
            .gateway_db
            .begin_transaction_nc()
            .await
            .load_registered_incoming_contract(payment_image)
            .await
            .ok_or(PublicGatewayError::LNv2(LNv2Error::IncomingPayment(
                "No corresponding decryption contract available".to_string(),
            )))?;

        if registered_incoming_contract.incoming_amount_msats != amount_msats {
            return Err(PublicGatewayError::LNv2(LNv2Error::IncomingPayment(
                "The available decryption contract's amount is not equal to the requested amount"
                    .to_string(),
            )));
        }

        // Turning receives off covers invoices issued before the switch was
        // flipped: every incoming contract of the federation is refused, whether
        // it would be funded from an HTLC or from a direct swap.
        if !self
            .receive_enabled(registered_incoming_contract.federation_id)
            .await
        {
            return Err(PublicGatewayError::ReceiveDisabled {
                federation_id_prefix: registered_incoming_contract.federation_id.to_prefix(),
            });
        }

        let client = self
            .select_client(registered_incoming_contract.federation_id)
            .await?
            .into_value();

        Ok((registered_incoming_contract.contract, client))
    }
