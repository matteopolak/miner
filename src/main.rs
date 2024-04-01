use clap::Parser;

mod block;
mod miner;
mod rpc;

#[derive(Parser)]
#[command(version, about, author)]
struct Args {
	/// Defaults to the RPC_USERNAME environment variable if not set
	#[arg(short, long)]
	pub username: Option<String>,
	/// Defaults to the RPC_PASSWORD environment variable if not set
	#[arg(short, long)]
	pub password: Option<String>,
	/// Defaults to the RPC_ADDRESS environment variable if not set
	#[arg(short, long)]
	pub address: Option<String>,
}

#[tokio::main]
async fn main() {
	let args = Args::parse();

	let username = args
		.username
		.or_else(|| std::env::var("RPC_USERNAME").ok())
		.expect("RPC_USERNAME environment variable is not set");

	let password = args
		.password
		.or_else(|| std::env::var("RPC_PASSWORD").ok())
		.expect("RPC_PASSWORD environment variable is not set");

	let address = args
		.address
		.or_else(|| std::env::var("RPC_ADDRESS").ok())
		.expect("RPC_ADDRESS environment variable is not set");

	let rpc = rpc::Client::new(&address, &username, &password).unwrap();

	let address = rpc.get_new_address().await.unwrap();
	let miner = miner::Miner::new(rpc, address);

	//println!("{:?}", miner.rpc.get_block_template().await);
	miner.mine().await.unwrap();
}
