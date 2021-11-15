use actix_web::{HttpResponse, ResponseError, http::StatusCode};
use serde::{Serialize, Deserialize};

#[allow(dead_code)]
pub type Res = Result<HttpResponse, ResErr>;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ResBody<T> {
    message: String,
    data: T,
}

#[derive(Debug, Clone)]
pub struct ResErr {
    stt: StatusCode,
    body: ResBody<&'static str>,
}

impl ResErr {
    #[allow(dead_code)]
    pub fn new(stt: StatusCode, msg: String) -> Self {
        Self {
            stt,
            body: ResBody {
                message: msg,
                data: "",
            }
        }
    }
}

impl std::fmt::Display for ResErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let dbg: &dyn std::fmt::Debug = self;
        dbg.fmt(f)
    }
}

impl ResponseError for ResErr {
    fn status_code(&self) -> StatusCode {
        self.stt
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).json(self.body.clone())
    }
}
