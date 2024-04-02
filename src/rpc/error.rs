use std::io;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Error {
	pub code: i32,
	pub message: String,
}

impl From<io::Error> for Error {
	fn from(value: io::Error) -> Self {
		Self {
			code: 0,
			message: value.to_string(),
		}
	}
}

impl From<ureq::Error> for Error {
	fn from(value: ureq::Error) -> Self {
		Self {
			code: 0,
			message: value.to_string(),
		}
	}
}
