use crate::rpc;

// 61a01000-89d4-4c64-974b-84826c01a3ea
#[derive(Debug)]
pub struct Miner {
	pub rpc: rpc::Client,
}

impl Miner {
	pub fn new(rpc: rpc::Client) -> Self {
		Self { rpc }
	}
}
