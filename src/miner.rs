use std::str::FromStr as _;

use bitcoin::{consensus::Encodable, hashes::Hash as _, Block};

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

	pub async fn mine(&self) -> Result<(), rpc::Error> {
		let template = self.rpc.get_block_template().await?;
		let target = bitcoin::BlockHash::from_byte_array(template.target);
		let nonce_range = template.nonce_range.clone();

		let mut block = self.create_block(template);

		for nonce in nonce_range {
			block.header.nonce = nonce;

			let hash = block.block_hash();

			if hash < target {
				tracing::info!(hash = ?hash, "found block hash");

				self.rpc.submit_block(&block).await?;

				break;
			}
		}

		Ok(())
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
