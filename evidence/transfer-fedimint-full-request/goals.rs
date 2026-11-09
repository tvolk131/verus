
pub fn deadline_policy(request: &InterceptPaymentRequest, height: Option<u32>) -> (blocks: u32)
    ensures
        (request.incoming_chan_id != 0 || request.htlc_id != 0) ==> blocks == 0,
        match height {
            None => blocks == 0,
            Some(current) => blocks >= 2 ==> request.expiry as int >= current as int + 2,
        },
        (request.incoming_chan_id == 0 && request.htlc_id == 0) ==> match height {
            None => true,
            Some(current) => request.expiry as int >= current as int + 2 ==> blocks >= 2,
        },
{
    request.lnv2_blocks_to_claim_deadline(height)
}

pub fn deadline_boundary(request: &InterceptPaymentRequest) -> (blocks: u32)
    ensures blocks == 0,
{ request.lnv2_blocks_to_claim_deadline(Some(request.expiry)) }

pub fn deadline_monotonicity(request: &InterceptPaymentRequest, earlier: u32, later: u32)
    -> (budgets: (u32, u32))
    ensures earlier <= later ==> budgets.1 <= budgets.0,
{
    (request.lnv2_blocks_to_claim_deadline(Some(earlier)),
     request.lnv2_blocks_to_claim_deadline(Some(later)))
}
