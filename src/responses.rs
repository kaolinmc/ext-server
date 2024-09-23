use rocket::futures::future::err;
use rocket::http::Status;
use rocket::Responder;
use rocket::response::status;
use rocket::serde::json::Json;
use serde::Serialize;

pub type HttpResult<T> = Result<T, HandlerError>;

#[derive(Responder)]
pub struct HandlerError {
    inner: (Status, Json<ErrorContent>),
}

#[derive(Serialize)]
struct ErrorContent {
    error_message: String,
    details: Option<String>,
}

impl HandlerError {
    pub fn new(
        error_message: String,
        details: Option<String>,
        status: Status,
    ) -> HandlerError {
        HandlerError {
            inner: (status, Json(ErrorContent {
                error_message,
                details,
            }))
        }
    }

    pub fn server_error(
        error_message: String,
        details: Option<String>,
    ) -> HandlerError {
         Self::new(error_message, details, Status::InternalServerError)
    }
}