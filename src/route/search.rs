use std::cmp::{max, min};
use rocket::{get, Route, routes};
use rocket::serde::json::Json;
use crate::responses::HttpResult;
use crate::route::metadata::ExtensionMetadataServer;
use crate::search::ExtensionSearchHandler;
use crate::types::SearchResponse;

pub struct ExtensionSearchServer;

impl Into<Vec<Route>> for ExtensionSearchServer {
    fn into(self) -> Vec<Route> {
        routes![
            search,
        ]
    }
}

#[get("/search?<query>&<page>&<pagination>")]
fn search(
    query: String,
    page: usize,
    pagination: usize,
    search_handler: &ExtensionSearchHandler,
) -> HttpResult<Json<SearchResponse>> {
    let handler = search_handler.lock().unwrap();

    let result = handler.search(query.as_str())?;

    let result = if page < result.len() {
        let range = min(result.len(), page + pagination);
        result.get(page..range).map(|it| it.to_vec()).unwrap_or(Vec::new())
    } else { Vec::new() };

    Ok(
        Json(SearchResponse {
            result
        })
    )
}