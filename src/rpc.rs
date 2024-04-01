use base64::{engine::general_purpose::STANDARD, Engine as _};
use bitcoin::consensus::Encodable as _;
use reqwest::{header, IntoUrl};
use serde::{de, Deserialize, Serialize};
use tracing::instrument;

use crate::block;

#[derive(Debug, Clone)]
pub struct Client {
	pub http: reqwest::Client,
	pub url: reqwest::Url,
}

#[derive(Debug, Serialize)]
pub enum Param<'r> {
	Option {
		#[serde(skip_serializing_if = "Option::is_none")]
		rules: Option<&'r [&'r str]>,
		#[serde(skip_serializing_if = "Option::is_none")]
		capabilities: Option<&'r [&'r str]>,
	},
	String(&'r str),
}

#[derive(Debug, Serialize)]
pub struct Request<'r> {
	pub jsonrpc: &'r str,
	pub id: &'r str,
	pub method: &'r str,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub params: Option<&'r [Param<'r>]>,
}

#[derive(Debug, Deserialize)]
pub struct Response<T> {
	pub result: Option<T>,
	pub error: Option<Error>,
	pub id: String,
}

#[derive(Debug, Deserialize)]
pub struct Error {
	pub code: i32,
	pub message: String,
}

impl From<reqwest::Error> for Error {
	fn from(value: reqwest::Error) -> Self {
		Self {
			code: 0,
			message: value.to_string(),
		}
	}
}

impl Client {
	pub fn new<U: IntoUrl>(url: U, username: &str, password: &str) -> Result<Self, reqwest::Error> {
		let url = url.into_url()?;
		let mut headers = header::HeaderMap::new();

		headers.insert(
			header::AUTHORIZATION,
			header::HeaderValue::try_from(format!(
				"Basic {}",
				STANDARD.encode(format!("{}:{}", username, password))
			))
			.expect("authorization header contains invalid characters"),
		);

		let http = reqwest::Client::builder()
			.default_headers(headers)
			.build()
			.expect("failed to build http client");

		Ok(Self { http, url })
	}

	pub async fn submit_block(&self, block: &bitcoin::Block) -> Result<(), Error> {
		let mut data = vec![];
		block.consensus_encode(&mut data).unwrap();

		self.request(&Request {
			jsonrpc: "1.0",
			id: env!("CARGO_PKG_NAME"),
			method: "submitblock",
			params: Some(&[Param::String(&hex::encode(data))]),
		})
		.await
	}

	pub async fn get_block_template(&self) -> Result<block::Template, Error> {
		self.request(&Request {
			jsonrpc: "1.0",
			id: env!("CARGO_PKG_NAME"),
			method: "getblocktemplate",
			params: Some(&[Param::Option {
				rules: Some(&["segwit"]),
				capabilities: Some(&["coinbasetxn", "workid", "coinbase/append"]),
			}]),
		})
		.await
	}

	#[instrument(name = "rpc", skip(self))]
	async fn request<T>(&self, request: &Request<'_>) -> Result<T, Error>
	where
		T: de::DeserializeOwned,
	{
		let response = self
			.http
			.post(self.url.clone())
			.json(request)
			.send()
			.await?;

		tracing::Span::current().record("status", &response.status().to_string());

		let body = response.json::<Response<T>>().await?;

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
				pub async fn $name(&self) -> Result<$result, $crate::rpc::Error> {
					self.request(&Request {
						jsonrpc: "1.0",
						id: env!("CARGO_PKG_NAME"),
						method: $method,
						params: None,
					})
					.await
				}
			}
		)*
	};
}

#[derive(Debug, Deserialize)]
pub struct NetworkInfo {
	pub version: i32,
}

impl_basic_rpc! {
	get_network_info, "getnetworkinfo" -> NetworkInfo,
	get_new_address, "getnewaddress" -> String
}
