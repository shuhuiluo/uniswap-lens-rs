use alloy::{contract::Error as ContractError, sol_types::Error as AbiError};

#[derive(Debug)]
#[cfg_attr(feature = "std", derive(thiserror::Error))]
pub enum Error {
    /// An error occurred retrieving the revert data.
    #[cfg_attr(feature = "std", error("Invalid revert data"))]
    InvalidRevertData,

    /// An error occurred ABI encoding or decoding.
    #[cfg_attr(feature = "std", error("{0}"))]
    AbiError(AbiError),

    /// An error occurred interacting with a contract over RPC.
    #[cfg_attr(feature = "std", error("{0}"))]
    ContractError(ContractError),
}

impl From<AbiError> for Error {
    #[inline]
    fn from(e: AbiError) -> Self {
        Self::AbiError(e)
    }
}

impl From<ContractError> for Error {
    #[inline]
    fn from(e: ContractError) -> Self {
        Self::ContractError(e)
    }
}
