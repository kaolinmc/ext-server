use std::sync::{Arc, Mutex};
use rocket::http::Status;
use rocket::State;
use tokio::io;
use crate::responses::HandlerError;
use crate::search::search::SearchHandler;
use crate::types::ExtensionIdentifier;

mod index;
mod token;
pub mod search;

#[derive(Debug)]
pub enum SearchError {
    TokenizationError,
    IoError(io::Error)
}

impl From<SearchError> for HandlerError {
    fn from(value: SearchError) -> Self {
        match value {
            SearchError::TokenizationError =>  HandlerError::new(
                "Internal search error".into(),
                Some("Failed to tokenize input".to_string()),
                Status::InternalServerError
            ),
            SearchError::IoError(e) => e.into()
        }
    }
}

pub type ExtensionSearchHandler = State<Arc<Mutex<SearchHandler<ExtensionIdentifier>>>>;