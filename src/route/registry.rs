use std::fs::File;
use std::io;
use std::io::{Cursor, Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};

use rocket::{get, put, Route, routes, State};
use rocket::figment::Source::{Custom};
use rocket::fs::{FileServer, NamedFile, TempFile};
use rocket::futures::{AsyncReadExt, TryFutureExt};
use rocket::http::hyper::Version;
use rocket::http::Status;
use rocket::response::status;
use rocket::route::{Cloneable, Handler, Outcome};
use serde::de::Unexpected::Str;
use tempfile::{NamedTempFile, tempfile, tempfile_in};
use tokio::fs::create_dir_all;
use zip::read::ZipFile;
use zip::result::{ZipError, ZipResult};
use zip::ZipArchive;

use crate::auth::{Authorization, Authorizer};
use crate::metadata::MetadataHandler;
use crate::responses::{HandlerError, HttpResult};
use crate::search::ExtensionSearchHandler;
use crate::types::{ExtensionBundle, ExtensionIdentifier, ExtensionMetadata, ExtensionRuntimeModel, PartitionRuntimeModel, VersionType};

#[derive(Debug, Clone)]
pub struct ExtensionFileServer;

impl Into<Vec<Route>> for ExtensionFileServer {
    fn into(self) -> Vec<Route> {
        routes![
            get_object,
            put_object
        ]
    }
}

#[get("/registry/<path..>")]
async fn get_object(
    path: PathBuf,
    metadata_handler: &State<MetadataHandler>,
) -> HttpResult<NamedFile> {
    let path = Path::new("static").join(path);
    if !path.exists() {
        return Err(
            HandlerError::new(
                "File not found".into(),
                None,
                Status::NotFound,
            )
        );
    }

    if path.to_str().unwrap().ends_with("erm.json") {
        let file = File::open(&path)?;

        let erm: ExtensionRuntimeModel = serde_json::from_reader(&file).unwrap();

        metadata_handler.increment_download((&erm).into());
    }

    Ok(NamedFile::open(path).await?)
}

#[put("/registry", data = "<data>")]
async fn put_object(
    mut data: TempFile<'_>,
    authorized: Authorization,
    metadata_handler: &State<MetadataHandler>,
    search_handler: &ExtensionSearchHandler,
) -> HttpResult<()> {
    let file = NamedTempFile::new()?
        .into_temp_path();

    data.persist_to(&file).await?;

    let mut bundle = build_bundle_from(File::open(file)?).await?;
    validate_bundle(&bundle)?;

    let path = bundle.runtime_model.group_id.split(".").fold(PathBuf::from("static/"), |acc, it| {
        acc.join(it)
    }).join(bundle.runtime_model.name.clone()).join(bundle.runtime_model.version.clone());

    write_bundle(path, &mut bundle).await?;

    metadata_handler.new_version(
        (&bundle.runtime_model).into(),
        bundle.runtime_model.version.clone(),
    )?;

    let metadata = bundle.metadata;

    let identifier = ExtensionIdentifier {
        group: bundle.runtime_model.group_id.clone(),
        name: bundle.runtime_model.name.clone(),
    };

    let mut handler = search_handler.lock().unwrap();
    // Names will arbitrarily index with higher ranks so that search by name comes up first
    handler.index(
        metadata.name.as_str(),
        identifier.clone(),
        10,
    )?;

    // Description will arbitrarily index with lower ranks.
    handler.index(
        metadata.description.as_str(),
        identifier.clone(),
        1,
    )?;

    Ok(())
}

impl From<ZipError> for HandlerError {
    fn from(value: ZipError) -> Self {
        HandlerError::server_error(
            "Internal server error".into(),
            Some(value.to_string()),
        )
    }
}

async fn build_bundle_from(
    read: impl Read + Seek
) -> HttpResult<ExtensionBundle<Cursor<Vec<u8>>>> {
    let mut zip = ZipArchive::new(read)?;

    let runtime_model = zip.by_name("erm.json").map_err(|e| {
        if let ZipError::FileNotFound = e {
            HandlerError::new("Invalid extension bundle".into(), Some("No erm.json present in the bundle.".into()), Status::BadRequest)
        } else {
            e.into()
        }
    })?;
    let runtime_model: ExtensionRuntimeModel = serde_json::from_reader(runtime_model).map_err(|e| {
        HandlerError::new(
            "Invalid ERM packaged in Extension Bundle".into(),
            Some(e.to_string()),
            Status::BadRequest,
        )
    })?;

    let metadata = zip.by_name("metadata.json").map_err(|e| {
        if let ZipError::FileNotFound = e {
            HandlerError::new("Invalid extension bundle".into(), Some("No metadata.json present in the bundle.".into()), Status::BadRequest)
        } else {
            e.into()
        }
    })?;
    let metadata: ExtensionMetadata = serde_json::from_reader(metadata).map_err(|e| {
        HandlerError::new(
            "Invalid metadata packaged in Extension Bundle".into(),
            Some(e.to_string()),
            Status::BadRequest,
        )
    })?;

    // let partitions = runtime_model.partitions.iter().map(|partition_ref| -> HttpResult<PartitionRuntimeModel> {
    //     let prm = zip.by_name(format!("{}.json", partition_ref.name).as_str()).map_err(|e| {
    //         if let ZipError::FileNotFound = e {
    //             HandlerError::new("Invalid extension bundle".into(), format!("Partition {part} defined, however failed to find the file 'partitions/{part}.json'", part = partition_ref.name).into(), Status::BadRequest)
    //         } else {
    //             e.into()
    //         }
    //     })?;
    //     let prm: PartitionRuntimeModel = serde_json::from_reader(prm).map_err(|e| {
    //         HandlerError::new(
    //             format!("Invalid PRM for partition '{}' packaged in Extension Bundle", partition_ref.name).into(),
    //             Some(e.to_string()),
    //             Status::BadRequest,
    //         )
    //     })?;
    //     if let Err(e) = zip.by_name(format!("{}.jar", partition_ref.name).as_str()) {
    //         if let ZipError::FileNotFound = e {
    //             return Err(HandlerError::new("Invalid extension bundle".into(), format!("Partition {part} defined, however failed to find the file 'partitions/{part}.jar'", part = partition_ref.name).into(), Status::BadRequest));
    //         };
    //     };
    //
    //     Ok(prm)
    // }).collect::<HttpResult<Vec<PartitionRuntimeModel>>>()?;

    Ok(ExtensionBundle {
        runtime_model,
        metadata,
        files: (0..zip.len()).map(|it| {
            let mut file = zip.by_index(it)?;
            let mut cursor = Cursor::new(Vec::new());

            io::copy(&mut file, &mut cursor)?;

            Ok::<(Cursor<Vec<u8>>, String), HandlerError>((cursor, file.name().to_string()))
        }).collect::<HttpResult<Vec<_>>>()?,
    })
}

fn validate_bundle<'a>(
    extension_bundle: &ExtensionBundle<impl Read>
) -> HttpResult<()> {
    VersionType::classify(&extension_bundle.runtime_model.version)?;

    Ok(())
}

// fn into_ise<T>(inner: T) -> status::Custom<T> {
//     status::Custom(
//         Status::InternalServerError,
//         inner,
//     )
// }

impl From<io::Error> for HandlerError {
    fn from(value: io::Error) -> Self {
        HandlerError::server_error(
            "Internal server error".into(),
            Some(value.to_string()),
        )
    }
}

async fn write_bundle(
    path: PathBuf,
    bundle: &mut ExtensionBundle<impl Read + Seek>,
) -> HttpResult<()> {
    create_dir_all(&path).await?;

    for (read, name) in bundle.files.iter_mut() {
        let file_path =
            path.join(
                if !name.starts_with(".") {
                    format!(
                        "{}-{}-{}",
                        bundle.runtime_model.name,
                        bundle.runtime_model.version,
                        name,
                    )
                } else {
                    format!(
                        "{}-{}{}",
                        bundle.runtime_model.name,
                        bundle.runtime_model.version,
                        name
                    )
                }
            );

        read.seek(SeekFrom::Start(0)).expect("Failed to seek");
        let mut contents: Vec<u8> = Vec::new();
        read.read_to_end(&mut contents)?;

        std::fs::write(file_path, contents)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::fs::{File, read, exists};
    use std::fs;
    use std::io::Write;
    use std::ops::Deref;
    use std::path::{Path, PathBuf};
    use std::sync::{Arc, Mutex};

    use rocket::{Request, uri};
    use rocket::http::Header;
    use zip::write::SimpleFileOptions;
    use zip::ZipWriter;

    use crate::auth::Authorizer;
    use crate::metadata::MetadataHandler;
    use crate::route::registry::ExtensionFileServer;
    use crate::search::search::SearchHandler;
    use crate::types::{ExtensionIdentifier, ExtensionMetadata, ExtensionRuntimeModel, PartitionRuntimeModel};

    async fn make_zip() -> PathBuf {
        let partition_test1_prm = PartitionRuntimeModel {
            r#type: "test".into(),
            name: "test1".into(),
            repositories: vec![],
            dependencies: vec![],
            options: Default::default(),
        };

        let erm = ExtensionRuntimeModel {
            api_version: 0,
            group_id: "com.example".into(),
            name: "fishmonger".into(),
            version: "1.0".into(),
            repositories: vec![],
            parents: vec![],
            partitions: vec![
                partition_test1_prm.clone()
            ],
        };

        let metadata = ExtensionMetadata {
            name: "Fish Monger".into(),
            developers: vec!["Durgan McBroom".to_string()],
            icon: Default::default(),
            description: "A extension that does fish mongering".into(),
            tags: vec![],
            app: "test".into(),
        };


        if !Path::new("test/test.zip").exists() {
            if !exists("dir").unwrap() {
                fs::create_dir("test").unwrap();
            }
            let file = File::create("test/test.zip").unwrap();
            let mut zip = ZipWriter::new(file);

            zip.start_file("erm.json", SimpleFileOptions::default()).unwrap();
            zip.write_all(serde_json::to_vec(&erm).unwrap().deref()).unwrap();

            zip.start_file("metadata.json", SimpleFileOptions::default()).unwrap();
            zip.write_all(serde_json::to_vec(&metadata).unwrap().deref()).unwrap();

            zip.start_file("test1.json", SimpleFileOptions::default()).unwrap();
            zip.write_all(serde_json::to_vec(&partition_test1_prm).unwrap().deref()).unwrap();

            zip.start_file("test1.jar", SimpleFileOptions::default()).unwrap();
            zip.write_all("Hey this isnt a jar, but its close enough".as_bytes()).unwrap();

            zip.finish().unwrap();
        }

        return PathBuf::from("test/test.zip");
    }

    #[tokio::test]
    async fn test_create_zip() {
        make_zip().await;

        // client.put(uri!(super::get_object))
        //     .body()
        //     .dispatch()
        // ;
    }

    #[tokio::test]
    async fn test_put_bundle() {
        struct TestAuthorizer;

        impl Authorizer for TestAuthorizer {
            fn is_authorized(&self, request: &Request, token: &str) -> bool {
                return true;
            }
        }

        let zip_resource = make_zip().await;

        let client = rocket::local::asynchronous::Client::tracked(
            rocket::build()
                .mount("/", ExtensionFileServer)
                .manage(Arc::new(Mutex::new(Box::new(TestAuthorizer) as Box<dyn Authorizer>)))
                .manage(MetadataHandler::hydrate_cache("config/metadata.json").unwrap())
                .manage(Arc::new(Mutex::new(SearchHandler::<ExtensionIdentifier>::hydrate_cache("config/search_index.json").unwrap())))
        ).await.unwrap();

        let r = client.put(uri!(super::put_object))
            .header(Header::new("Authorization", "Bearer nothing"))
            .body(read(zip_resource).unwrap())
            .dispatch().await;

        println!("{}", r.into_string().await.unwrap_or("no body".to_string()));
        let metadata_handler: &MetadataHandler = client.rocket().state().unwrap();
        metadata_handler.persist_to("config/metadata.json").unwrap();

        let search_handler: &Arc<Mutex<SearchHandler<ExtensionIdentifier>>> = client.rocket().state().unwrap();
        search_handler.lock().unwrap().persist_to("config/search_index.json").unwrap();
    }
}