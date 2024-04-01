mod miner;
mod rpc;

#[tokio::main]
async fn main() {
	println!("{}", hex::encode([0x45u8, 0xfe, 0x89]));

	let rpc = rpc::Client::new("m", "61a01000-89d4-4c64-974b-84826c01a3ea");
	let miner = miner::Miner::new(rpc);

	println!("{:?}", miner.rpc.get_network_info().await);
}
