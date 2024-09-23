use std::fmt::format;
use std::fs::metadata;
use std::ops::Deref;
use std::path::PathBuf;

use rocket::{get, Route, routes, State};
use rocket::http::Status;
use rocket::route::Handler;
use rocket::serde::json::Json;

use crate::metadata::MetadataHandler;
use crate::responses::{HandlerError, HttpResult};
use crate::types::{ExtensionIdentifier, ManagedExtensionMetadata, RepositoryMetadata, VersionInfo, VersionType};

pub struct ExtensionMetadataServer;

impl Into<Vec<Route>> for ExtensionMetadataServer {
    fn into(self) -> Vec<Route> {
        routes![
            get_managed_metadata,
            get_metadata
        ]
    }
}

#[get("/metadata")]
fn get_metadata(
    metadata: &State<RepositoryMetadata>,
    metadata_handler: &State<MetadataHandler>
) -> Json<RepositoryMetadata> {
    Json(RepositoryMetadata {
        name: metadata.name.clone(),
        description: metadata.description.clone(),
        icon: metadata.icon.clone(),
        extension_count: metadata_handler.extension_count(),
        app_ids: metadata.app_ids.clone(),
    })
}

#[get("/metadata/<path..>")]
fn get_managed_metadata(
    path: PathBuf,
    metadata_handler: &State<MetadataHandler>,
) -> HttpResult<Json<ManagedExtensionMetadata>> {
    let group_dots =
        path.parent().ok_or(HandlerError::new(
        "Invalid extension path".into(), None, Status::BadRequest,
    ))?.iter().map(|t| t.to_str().unwrap())
        .map(|str| format!("{}.", str))
        .collect::<String>();

    let group_dots = if let Some(x) = group_dots.strip_suffix(".") {
        x
    } else {
        ""
    }.to_string();

    let name = path.file_name().ok_or(HandlerError::new(
        "Invalid extension path".into(), None, Status::BadRequest,
    ))?.to_str().unwrap();

    let identifier = ExtensionIdentifier {
        group: group_dots,
        name: name.to_string(),
    };

    let (downloads, latest, versions) = metadata_handler.get_managed_metadata(&identifier)?;

    Ok(Json(ManagedExtensionMetadata {
        downloads,
        latest,
        versions: versions.iter().map(|it| {
            Ok::<VersionInfo, HandlerError>(VersionInfo {
                version: it.clone(),
                release_type: VersionType::classify(it.clone())?,
                metadata_path: format!(
                    "/registry/{}/{}-{}-metadata.json",
                    path.to_str().unwrap(),
                    name,
                    it
                ),
            })
        }).collect::<Result<Vec<_>, HandlerError>>()?,
    }))
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use rocket::uri;

    use crate::metadata::MetadataHandler;
    use crate::route::metadata::ExtensionMetadataServer;
    use crate::types::RepositoryMetadata;

    #[tokio::test]
    async fn test_get_repository_metadata() {
        let client = rocket::local::asynchronous::Client::tracked(
            rocket::build()
                .mount("/", ExtensionMetadataServer)
                .manage(MetadataHandler::hydrate_cache("config/metadata.json").unwrap())
                .manage(RepositoryMetadata {
                    name: "A test repository".to_string(),
                    description: "A cool description".to_string(),
                    icons: Default::default(),
                    extension_count: 0,
                    app_ids: vec![],
                })
        ).await.unwrap();

        let r = client.get(uri!(super::get_metadata))
            .dispatch().await;

        println!("{}", r.into_string().await.unwrap_or("no body".to_string()));

        let handler : &MetadataHandler = client.rocket().state().unwrap();
        handler.persist_to("config/metadata.json").unwrap();
    }

    #[tokio::test]
    async fn test_get_managed_metadata() {
        let client = rocket::local::asynchronous::Client::tracked(
            rocket::build()
                .mount("/", ExtensionMetadataServer)
                .manage(MetadataHandler::hydrate_cache("config/metadata.json").unwrap())
                .manage(RepositoryMetadata {
                    name: "A test repository".to_string(),
                    description: "A cool description".to_string(),
                    icons: Default::default(),
                    extension_count: 0,
                    app_ids: vec![],
                })
        ).await.unwrap();

        let r = client.get("/metadata/com/example/testing")
            .dispatch().await;

        println!("{}", r.into_string().await.unwrap_or("no body".to_string()));

        let handler : &MetadataHandler = client.rocket().state().unwrap();
        handler.persist_to("config/metadata.json").unwrap();
    }
}