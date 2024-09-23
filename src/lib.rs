pub mod route;
pub mod auth;
pub mod responses;
pub mod types;
pub mod config;
pub mod metadata;
pub mod search;

// TODO Pushing adds new indexes to the search tree and does not remove the old ones.
//    This means that many pushes will boost search results for certain extensions.