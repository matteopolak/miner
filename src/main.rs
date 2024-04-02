#![feature(never_type)]

use clap::Parser;

mod block;
mod gpu;
mod hash;
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

fn main() -> Result<!, rpc::Error> {
	hash::main();
	panic!();
	/*let args = Args::parse();

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

	tracing_subscriber::fmt().init();
	rayon::ThreadPoolBuilder::new()
		.num_threads(num_cpus::get())
		.build_global()
		.unwrap();

	let rpc = rpc::Client::new(address, &username, &password);

	let address = rpc.get_new_address()?;
	let miner = miner::Miner::new(rpc, address);

	miner.mine()*/
}
