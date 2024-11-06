#![allow(clippy::missing_inline_in_public_items)]

use alloy::{contract::Error as ContractError, sol_types::Error as AbiError};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// An error occurred retrieving the revert data.
    #[error("Invalid revert data")]
    InvalidRevertData,

    /// An error occurred ABI encoding or decoding.
    #[error("{0}")]
    AbiError(#[from] AbiError),

    /// An error occurred interacting with a contract over RPC.
    #[error("{0}")]
    ContractError(#[from] ContractError),
}
