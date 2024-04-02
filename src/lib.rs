#![feature(never_type)]

pub mod block;
pub mod error;
pub mod gpu;
pub mod miner;
pub mod rpc;

pub use error::Error;
pub use miner::Miner;
pub use rpc::Client;
