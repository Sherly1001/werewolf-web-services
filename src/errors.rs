use rocket::http::Status;
use rocket::response::{self, status, Responder};
use rocket::serde::json::{Value, serde_json::json};

pub struct Error(pub &'static str);

impl Error {
    pub fn new(err: &'static str) -> Self {
        Self(err)
    }
}

#[rocket::async_trait]
impl<'r> Responder<'r, 'static> for Error {
    fn respond_to(self, request: &'r rocket::Request<'_>) -> response::Result<'static> {
        status::Custom::<Value>(
            Status::UnprocessableEntity,
            json!({
                "status": "error",
                "message": self.0,
            })
        ).respond_to(request)
    }
}
