use std::{fmt, io};

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Error {
	pub code: i32,
	pub message: String,
}

impl fmt::Display for Error {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{} (code {})", self.message, self.code)
	}
}

impl std::error::Error for Error {}

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
