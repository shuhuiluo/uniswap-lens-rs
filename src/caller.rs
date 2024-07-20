/// Call an ephemeral contract and return the decoded data
#[macro_export]
macro_rules! call_ephemeral_contract {
    ($deploy_builder:expr, $call_type:ty, $block_id:expr) => {{
        let deploy_builder = match $block_id {
            Some(block_id) => $deploy_builder.block(block_id),
            None => $deploy_builder,
        };
        match deploy_builder.call_raw().await {
            Err(Error::TransportError(err)) => match err {
                TransportError::ErrorResp(payload) => {
                    let data: Bytes = payload.try_data_as().unwrap().unwrap();
                    Ok(<$call_type>::abi_decode_returns(data.as_ref(), true).unwrap())
                }
                _ => panic!("should be an error response: {:?}", err),
            },
            Err(err) => Err(err),
            Ok(_) => panic!("deployment should revert"),
        }
    }};
}
