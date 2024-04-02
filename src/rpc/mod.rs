mod auth;
mod error;

pub use error::Error;

use bitcoin::consensus::Encodable as _;
use serde::{de, Deserialize, Serialize};
use tracing::instrument;

use crate::block;

#[derive(Debug, Clone)]
pub struct Client {
	pub http: ureq::Agent,
	pub url: String,
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum Param<'r> {
	Option {
		#[serde(skip_serializing_if = "Option::is_none")]
		rules: Option<&'r [&'r str]>,
		#[serde(skip_serializing_if = "Option::is_none")]
		capabilities: Option<&'r [&'r str]>,
	},
	String(&'r str),
	Longpoll {
		#[serde(rename = "longpollid")]
		id: &'r str,
	},
}

#[derive(Debug, Serialize)]
pub struct Request<'r> {
	pub jsonrpc: &'r str,
	pub id: &'r str,
	pub method: &'r str,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub params: Option<[Param<'r>; 1]>,
}

#[derive(Debug, Deserialize)]
pub struct Response<T> {
	pub result: Option<T>,
	pub error: Option<Error>,
	pub id: String,
}

impl Client {
	pub fn new(url: String, username: &str, password: &str) -> Self {
		let http = ureq::AgentBuilder::new()
			.middleware(auth::Basic::new(username, password))
			.build();

		Self { http, url }
	}

	pub fn submit_block(&self, block: &bitcoin::Block) -> Result<(), Error> {
		// TODO: pre-allocate buffer
		let mut data = vec![];
		block.consensus_encode(&mut data).unwrap();

		self.request(&Request {
			jsonrpc: "1.0",
			id: env!("CARGO_PKG_NAME"),
			method: "submitblock",
			params: Some([Param::String(&hex::encode(data))]),
		})
	}

	pub fn get_block_template(&self, poll_id: Option<&str>) -> Result<block::Template, Error> {
		self.request(&Request {
			jsonrpc: "1.0",
			id: env!("CARGO_PKG_NAME"),
			method: "getblocktemplate",
			params: Some([poll_id.map_or_else(
				|| Param::Option {
					rules: Some(&["segwit"]),
					capabilities: Some(&["coinbase/append", "longpoll"]),
				},
				|id| Param::Longpoll { id },
			)]),
		})
	}

	#[instrument(name = "rpc", skip(self))]
	fn request<T>(&self, request: &Request<'_>) -> Result<T, Error>
	where
		T: de::DeserializeOwned,
	{
		let response = self.http.post(&self.url).send_json(request)?;

		tracing::Span::current().record("status", &response.status().to_string());

		let body = response.into_json::<Response<T>>()?;

		let response = match body {
			Response {
				result: Some(result),
				..
			} => Ok(result),
			Response {
				error: Some(error), ..
			} => Err(error),
			_ => Err(Error {
				code: 0,
				message: "no response".to_string(),
			}),
		};

		tracing::info!("request complete");

		response
	}
}

/// Implements basic RPC methods that take in no parameters
macro_rules! impl_basic_rpc {
	($($name:ident, $method:literal -> $result:path),*) => {
		$(
			impl $crate::rpc::Client {
				pub fn $name(&self) -> Result<$result, $crate::rpc::Error> {
					self.request(&Request {
						jsonrpc: "1.0",
						id: env!("CARGO_PKG_NAME"),
						method: $method,
						params: None,
					})
				}
			}
		)*
	};
}

impl_basic_rpc! {
	get_new_address, "getnewaddress" -> String
}
