/// Call an ephemeral contract and return the decoded data
#[macro_export]
macro_rules! call_ephemeral_contract {
    ($deploy_builder:expr, $call_type:ty, $block_id:expr) => {{
        let deploy_builder = match $block_id {
            Some(block_id) => $deploy_builder.block(block_id),
            None => $deploy_builder,
        };
        match deploy_builder.call_raw().await {
            Err(ContractError::TransportError(TransportError::ErrorResp(payload))) => {
                match payload.as_revert_data() {
                    Some(data) => Ok(<$call_type as SolCall>::abi_decode_returns(
                        data.as_ref(),
                        true,
                    )?),
                    None => Err(Error::InvalidRevertData(payload)),
                }
            }
            Err(err) => Err(Error::ContractError(err)),
            Ok(_) => panic!("deployment should revert"),
        }
    }};
}
