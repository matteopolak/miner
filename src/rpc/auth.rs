use base64::{engine::general_purpose::STANDARD, Engine as _};

pub struct Basic {
	content: String,
}

impl Basic {
	pub fn new(username: &str, password: &str) -> Self {
		let content = format!(
			"Basic {}",
			STANDARD.encode(format!("{username}:{password}"))
		);

		Self { content }
	}
}

impl ureq::Middleware for Basic {
	fn handle(
		&self,
		request: ureq::Request,
		next: ureq::MiddlewareNext,
	) -> Result<ureq::Response, ureq::Error> {
		next.handle(request.set("Authorization", &self.content))
	}
}
