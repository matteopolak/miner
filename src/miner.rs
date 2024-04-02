use std::{ops::Range, str::FromStr as _, sync::mpsc};

use bitcoin::hashes::Hash as _;

use crate::{block, rpc};

// 61a01000-89d4-4c64-974b-84826c01a3ea
#[derive(Debug, Clone)]
pub struct Miner {
	pub rpc: rpc::Client,
	pub address: bitcoin::Address,
}

impl Miner {
	pub fn new(rpc: rpc::Client, address: String) -> Self {
		let address = bitcoin::Address::from_str(&address)
			.expect("invalid address")
			.require_network(bitcoin::Network::Bitcoin)
			.expect("invalid network");

		Self { rpc, address }
	}

	pub fn mine(&self) -> Result<!, rpc::Error> {
		let (tx, rx) = mpsc::channel::<block::Template>();
		let mut template = self.rpc.get_block_template(None)?;
		let poll_id = std::mem::take(&mut template.longpoll_id);

		std::thread::scope(|s| {
			s.spawn(|| self.poll_new_block(tx, poll_id));
		});

		loop {
			let block = self.mine_block(template, &rx);

			self.rpc.submit_block(&block)?;

			template = self.rpc.get_block_template(None)?;
		}
	}

	pub fn mine_block(
		&self,
		template: block::Template,
		new: &mpsc::Receiver<block::Template>,
	) -> bitcoin::Block {
		let (mut target, mut nonce_range, mut block) = self.process_template(template);

		loop {
			for nonce in nonce_range.clone() {
				block.header.nonce = nonce;

				let hash = block.block_hash();

				if hash < target {
					tracing::info!(hash = ?hash, "found block hash");

					return block;
				}
			}

			// if we didn't find a valid nonce, increase the timestamp
			block.header.time += 1;

			let message = new.try_recv();

			if let Ok(template) = message {
				(target, nonce_range, block) = self.process_template(template);
			}
		}
	}

	fn poll_new_block(&self, template_tx: mpsc::Sender<block::Template>, mut poll_id: String) {
		loop {
			let template = self.rpc.get_block_template(Some(&poll_id));

			if let Ok(mut template) = template {
				poll_id = std::mem::take(&mut template.longpoll_id);

				let _ = template_tx.send(template);
			}
		}
	}

	fn process_template(
		&self,
		template: block::Template,
	) -> (bitcoin::BlockHash, Range<u32>, bitcoin::Block) {
		let target = bitcoin::BlockHash::from_byte_array(template.target);
		let nonce_range = template.nonce_range.clone();
		let block = self.create_block(template);

		(target, nonce_range, block)
	}

	fn create_block(&self, template: block::Template) -> bitcoin::Block {
		let script_pubkey = self.address.script_pubkey();

		// Creates the coinbase transaction
		let transaction = bitcoin::Transaction {
			version: bitcoin::transaction::Version::ONE,
			lock_time: bitcoin::locktime::absolute::LockTime::ZERO,
			input: vec![bitcoin::TxIn {
				previous_output: bitcoin::OutPoint::null(),
				script_sig: bitcoin::script::Builder::new().push_int(0).into_script(),
				sequence: bitcoin::Sequence::MAX,
				witness: bitcoin::Witness::new(),
			}],
			output: vec![bitcoin::TxOut {
				value: bitcoin::Amount::from_sat(template.coinbase_value),
				script_pubkey,
			}],
		};

		bitcoin::Block {
			header: bitcoin::block::Header {
				version: bitcoin::block::Version::from_consensus(template.version),
				prev_blockhash: bitcoin::BlockHash::from_byte_array(template.previous_block),
				merkle_root: bitcoin::TxMerkleNode::from_byte_array(template.transactions[0].hash),
				time: template.current_time,
				bits: bitcoin::CompactTarget::from_consensus(u32::from_le_bytes(template.bits)),
				nonce: template.nonce_range.start,
			},
			txdata: vec![transaction],
		}
	}
}
