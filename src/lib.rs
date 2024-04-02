#![feature(never_type)]

pub mod block;
pub mod gpu;
pub mod miner;
pub mod rpc;

pub use miner::Miner;
pub use rpc::Client;
pub use rpc::Error;
