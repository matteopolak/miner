use std::ops::Range;

use serde::{de, Deserialize};

type Hex<const L: usize> = [u8; L];
type Hash = Hex<32>;

#[derive(Debug, Deserialize)]
pub struct Template {
	pub version: i32,
	#[serde(with = "hex::serde", rename = "previousblockhash")]
	pub previous_block: Hash,
	pub transactions: Vec<Transaction>,
	#[serde(rename = "longpollid")]
	pub longpoll_id: String,
	#[serde(with = "hex::serde")]
	pub target: Hash,
	#[serde(with = "hex::serde")]
	pub bits: Hex<4>,
	#[serde(rename = "curtime")]
	pub current_time: u32,
	#[serde(rename = "coinbasevalue")]
	pub coinbase_value: u64,
	#[serde(deserialize_with = "hex_range", rename = "noncerange")]
	pub nonce_range: Range<u32>,
}

#[derive(Debug, Deserialize)]
pub struct Transaction {
	#[serde(with = "hex::serde", rename = "txid")]
	pub id: Hash,
	#[serde(with = "hex::serde")]
	pub data: Vec<u8>,
	#[serde(with = "hex::serde")]
	pub hash: Hash,
	pub fee: u64,
	pub weight: u32,
}

/// Parses "00000000ffffffff" into a range Range { start: 0, end: 4294967295 }
fn hex_range<'de, D>(deserializer: D) -> Result<Range<u32>, D::Error>
where
	D: de::Deserializer<'de>,
{
	let bytes: [u8; 8] = hex::serde::deserialize(deserializer)?;
	// transmute [u8; 8] into [[u8; 4]; 2]
	let [start, end]: [[u8; 4]; 2] = unsafe { std::mem::transmute(bytes) };

	let start = u32::from_le_bytes(start);
	let end = u32::from_le_bytes(end);

	Ok(start..end)
}
