#![feature(never_type)]

use clap::Parser;
use miner::{rpc, Error};

#[derive(Parser)]
#[command(version, about, author)]
struct Args {
	/// RPC username
	#[arg(short, long, env = "RPC_USERNAME")]
	pub username: String,
	/// RPC password
	#[arg(short, long, env = "RPC_PASSWORD")]
	pub password: String,
	/// RPC address url
	#[arg(short, long, env = "RPC_ADDRESS")]
	pub address: String,
	/// Use the GPU for mining
	#[arg(short, long)]
	pub gpu: bool,
}

fn main() -> Result<!, Error> {
	let args = Args::parse();

	tracing_subscriber::fmt().init();

	if !args.gpu {
		rayon::ThreadPoolBuilder::new()
			.num_threads(num_cpus::get())
			.build_global()
			.unwrap();
	}

	let rpc = rpc::Client::new(args.address, &args.username, &args.password);

	let wallet_address = rpc.get_new_address()?;
	let miner = miner::Miner::new(rpc, wallet_address, args.gpu);

	miner.mine()
}
