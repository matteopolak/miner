#![feature(never_type)]
#![warn(clippy::pedantic)]

pub mod block;
pub mod error;
pub mod gpu;
pub mod miner;
pub mod rpc;

pub use error::Error;
pub use miner::Miner;
pub use rpc::Client;
