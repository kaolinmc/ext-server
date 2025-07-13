use crate::responses::HttpResult;
use crate::route::metadata::ExtensionMetadataServer;
use crate::search::ExtensionSearchHandler;
use crate::types::SearchResponse;
use rocket::serde::json::Json;
use rocket::{get, routes, Route};
use std::cmp::{max, min};

pub struct ExtensionSearchServer;

impl Into<Vec<Route>> for ExtensionSearchServer {
    fn into(self) -> Vec<Route> {
        routes![search,]
    }
}

// Page index starts at 0
#[get("/search?<query>&<page>&<pagination>")]
fn search(
    query: String,
    page: usize,
    pagination: usize,
    search_handler: &ExtensionSearchHandler,
) -> HttpResult<Json<SearchResponse>> {
    let handler = search_handler.lock().unwrap();

    let result = handler.search(query.as_str())?;

    let result = if page * pagination < result.len() {
        let range = min(result.len(), (page + 1) * pagination);

        result
            .get((page * pagination)..range)
            .map(|it| it.to_vec())
            .unwrap_or(Vec::new())
    } else {
        Vec::new()
    };

    Ok(Json(SearchResponse { result }))
}
