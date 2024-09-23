use std::sync::{Arc, Mutex};
use rocket::{async_trait, Request};
use rocket::http::Status;
use rocket::outcome::Outcome::Forward;
use rocket::request::{FromRequest, Outcome};

pub struct Authorization;

pub trait Authorizer: Send + Sync {
    fn is_authorized(&self, request: &Request, token: &str) -> bool;
}

#[async_trait]
impl<'r> FromRequest<'r> for Authorization {
    type Error = ();

    async fn from_request(
        request: &'r Request<'_>
    ) -> Outcome<Self, Self::Error> {
        let auth_header = if let Some(x) = request.headers().get("Authorization")
            .next() {
            x
        } else {
            return Forward(Status::Unauthorized);
        };

        let auth_header = if let Some(result) = auth_header.strip_prefix("Bearer ") {
            result
        } else {
            return Forward(Status::Unauthorized)
        };

        let authorizer = request.rocket().state::<Arc<Mutex<Box<dyn Authorizer>>>>().expect("No authorizer provided!");

        let authorized = authorizer.lock().unwrap().is_authorized(request, auth_header);

        if authorized {
            Outcome::Success(Authorization)
        } else {
            Forward(Status::Unauthorized)
        }
    }
}