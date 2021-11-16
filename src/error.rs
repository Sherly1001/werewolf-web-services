use actix_web::{
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    http::{header, HeaderValue},
    Error, HttpResponse,
};
use futures::{
    future::{ok, Ready},
    Future,
};
use serde::{Deserialize, Serialize};

use std::pin::Pin;
use std::task::{Context, Poll};

#[allow(dead_code)]
pub type Res = Result<HttpResponse, Error>;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ResBody<T> {
    message: String,
    data: T,
}

impl<T: Serialize> ResBody<T> {
    pub fn new(msg: String, data: T) -> Res {
        Ok(HttpResponse::Ok().json(Self::new_body(msg, data)))
    }

    pub fn new_body(msg: String, data: T) -> Self {
        Self { message: msg, data }
    }
}

impl<T: Serialize> std::fmt::Display for ResBody<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", serde_json::to_string(self).unwrap())
    }
}

pub struct ResErrWrap;

impl<S, B> Transform<S> for ResErrWrap
where
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = ResErrMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(ResErrMiddleware { service })
    }
}

pub struct ResErrMiddleware<S> {
    service: S,
}

impl<S, B> Service for ResErrMiddleware<S>
where
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&mut self, mut req: ServiceRequest) -> Self::Future {
        req.headers_mut().insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static("application/json"),
        );

        let fut = self.service.call(req);

        Box::pin(async move {
            let mut res = fut.await?;

            res.headers_mut().insert(
                header::CONTENT_TYPE,
                HeaderValue::from_static("application/json"),
            );

            if let Some(err) = res.response().error() {
                let stt = err.as_response_error().status_code();
                return Ok(ServiceResponse::new(
                    res.request().clone(),
                    HttpResponse::build(stt)
                        .json(ResBody::new_body(err.to_string(), ""))
                        .into_body(),
                ));
            }

            Ok(res)
        })
    }
}
