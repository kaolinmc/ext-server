#[macro_use]
extern crate rocket;

use std::env;
use std::fs::File;
use std::ops::Deref;
use std::str::FromStr;
use std::sync::{Arc, Mutex};

use rocket::{Request, Response, Rocket};
use rocket::fairing::{Fairing, Info, Kind};
use rocket::http::Header;

use ext_server::auth::Authorizer;
use ext_server::metadata::MetadataHandler;
use ext_server::route::metadata::ExtensionMetadataServer;
use ext_server::route::registry::ExtensionFileServer;
use ext_server::route::search::ExtensionSearchServer;
use ext_server::search::search::SearchHandler;
use ext_server::types::{ExtensionIdentifier, RepositoryMetadata};

struct BasicAuth;

impl Authorizer for BasicAuth {
    fn is_authorized(&self, request: &Request, token: &str) -> bool {
        let real_token = env::var("AUTH_TOKEN").expect("No auth token set in 'AUTH_TOKEN'.");

        real_token == token
    }
}

pub struct CORS;

#[rocket::async_trait]
impl Fairing for CORS {
    fn info(&self) -> Info {
        Info {
            name: "Add CORS headers",
            kind: Kind::Response,
        }
    }

    async fn on_response<'r>(&self, _req: &'r Request<'_>, res: &mut Response<'r>) {
        res.set_header(Header::new("Access-Control-Allow-Origin", "*"));
        res.set_header(Header::new("Access-Control-Allow-Methods", "GET, POST, PUT, DELETE, OPTIONS"));
        res.set_header(Header::new("Access-Control-Allow-Headers", "Content-Type, Authorization"));
    }
}

#[rocket::main]
async fn main() {
    let repository_metadata = File::open("data/config.json").expect("No config file setup for this repository! Please define it in data/config.json");
    let repository_metadata: RepositoryMetadata = serde_json::from_reader(repository_metadata).expect("Invalid config.json in data/config.json.");

    let rocket = Rocket::build()
        .attach(CORS)
        .configure(rocket::Config::figment().merge((
            "port", u16::from_str(&*env::var("PORT").unwrap_or("8080".into())).expect("Invalid $PORT env variable defined, not a u16.")
        )))
        .mount("/", ExtensionFileServer)
        .mount("/", ExtensionMetadataServer)
        .mount("/", ExtensionSearchServer)
        .mount("/", routes![home])
        .manage(Arc::new(Mutex::new(Box::new(BasicAuth) as Box<dyn Authorizer>)))
        .manage(MetadataHandler::hydrate_cache("data/metadata.json").unwrap())
        .manage(repository_metadata)
        .manage(Arc::new(Mutex::new(SearchHandler::<ExtensionIdentifier>::hydrate_cache("data/search_index.json").unwrap())))
        .launch().await.unwrap();

    let handler: &MetadataHandler = rocket.state().unwrap();
    handler.persist_to("data/metadata.json").unwrap();

    let search_handler: &Arc<Mutex<SearchHandler<ExtensionIdentifier>>> = rocket.state().unwrap();
    search_handler.lock().unwrap().persist_to("data/search_index.json").unwrap();
}

#[get("/")]
fn home() -> &'static str {
    "You've found the basic implementation of the extframework ext-server! Go to https://github.com/extframework/ext-server to check it out."
}