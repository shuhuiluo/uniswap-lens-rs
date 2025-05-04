//! # uniswap-lens
//!
//! A library for querying Uniswap V3 using ephemeral lens contracts.

#![cfg_attr(not(any(feature = "std", test)), no_std)]
#![warn(
    missing_copy_implementations,
    missing_debug_implementations,
    unreachable_pub,
    clippy::missing_const_for_fn,
    clippy::missing_inline_in_public_items,
    clippy::redundant_clone,
    rustdoc::all
)]
#![cfg_attr(not(test), warn(unused_crate_dependencies))]
#![cfg_attr(docsrs, feature(doc_cfg, doc_auto_cfg))]

extern crate alloc;

pub mod bindings;
pub mod caller;
pub mod error;
pub mod pool_lens;
pub mod position_lens;
pub mod storage_lens;

#[cfg(test)]
mod tests;

pub mod prelude {
    pub use super::{error::Error, pool_lens::*, position_lens::*, storage_lens::*};
}
