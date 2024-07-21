//! # uniswap-lens
#![warn(
    missing_copy_implementations,
    missing_debug_implementations,
    unreachable_pub,
    clippy::missing_const_for_fn,
    rustdoc::all
)]
#![cfg_attr(not(test), warn(unused_crate_dependencies))]
#![cfg_attr(docsrs, feature(doc_cfg, doc_auto_cfg))]

#[allow(
    missing_copy_implementations,
    missing_debug_implementations,
    clippy::missing_const_for_fn
)]
pub mod bindings;
pub mod caller;
pub mod pool_lens;
// pub mod position_lens;
// pub mod storage_lens;

pub mod prelude {
    pub use super::{bindings::*, pool_lens::*};
}
