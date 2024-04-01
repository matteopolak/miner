use base64::{engine::general_purpose::STANDARD, Engine as _};
use reqwest::header;
use serde::{de, Deserialize, Serialize};

#[derive(Debug)]
pub struct Client {
	pub http: reqwest::Client,
}

#[derive(Debug, Serialize)]
pub struct Param {
	pub rules: &'static [&'static str],
}

#[derive(Debug, Serialize)]
pub struct Request {
	pub jsonrpc: &'static str,
	pub id: &'static str,
	pub method: &'static str,
	pub params: &'static [Param],
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
	pub fn new(username: &str, password: &str) -> Self {
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

		Self { http }
	}

	async fn request<T>(&self, request: &Request) -> Result<T, Error>
	where
		T: de::DeserializeOwned,
	{
		let response = self
			.http
			.post("http://127.0.0.1:18332")
			.json(request)
			.send()
			.await?
			.json::<Response<T>>()
			.await?;

		match response {
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
		}
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
						id: "1",
						method: $method,
						params: &[],
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
	// get_block_template, "getblocktemplate" -> BlockTemplate,
	get_network_info, "getnetworkinfo" -> NetworkInfo
}
