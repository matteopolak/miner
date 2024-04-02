use std::fmt;

use crate::{gpu, rpc};

#[derive(Debug)]
pub enum Error {
	Gpu(gpu::Error),
	Rpc(rpc::Error),
	Bitcoin(bitcoin::consensus::encode::Error),
}

impl fmt::Display for Error {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::Gpu(e) => write!(f, "gpu error: {}", e),
			Self::Rpc(e) => write!(f, "rpc error: {}", e),
			Self::Bitcoin(e) => write!(f, "bitcoin error: {}", e),
		}
	}
}

impl std::error::Error for Error {}

impl From<gpu::Error> for Error {
	fn from(value: gpu::Error) -> Self {
		Self::Gpu(value)
	}
}

impl From<rpc::Error> for Error {
	fn from(value: rpc::Error) -> Self {
		Self::Rpc(value)
	}
}

impl From<bitcoin::consensus::encode::Error> for Error {
	fn from(value: bitcoin::consensus::encode::Error) -> Self {
		Self::Bitcoin(value)
	}
}
