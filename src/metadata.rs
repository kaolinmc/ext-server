use std::collections::HashMap;
use std::fs::{create_dir_all, File};
use std::io;
use std::io::Write;
use std::ops::Deref;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use rocket::serde::Serialize;
use serde::Deserialize;

use crate::responses::HttpResult;
use crate::types::{ExtensionIdentifier, LatestVersion, VersionType};

pub struct MetadataHandler {
    inner: Arc<Mutex<Inner>>,
}

#[derive(Serialize, Deserialize)]
struct Inner {
    pub downloads: HashMap<String, u32>,
    pub latest: HashMap<String, LatestVersion>,
    pub versions: HashMap<String, Vec<String>>,
}

impl MetadataHandler {
    pub fn persist_to<T: Into<PathBuf>>(&self, path: T) -> Result<(), io::Error> {
        let path = path.into();
        if !Path::new(&path).exists() {
            if let Some(x) = path.parent() {
                create_dir_all(x.clone())?;
            }
            File::create(path.clone())?;
        };

        let mut file = std::fs::OpenOptions::new().write(true).truncate(true).open(path)?;

        let value = self.inner.lock().unwrap();
        let content = serde_json::to_vec(value.deref()).unwrap();

        file.write_all(content.deref())
    }

    pub fn hydrate_cache<T: Into<PathBuf>>(path: T) -> Result<MetadataHandler, io::Error> {
        let path = path.into();
        let inner = if Path::new(&path).exists() {
            let file = File::open(path)?;

            let inner: Inner = serde_json::from_reader(file).unwrap();

            inner
        } else {
            Inner {
                downloads: Default::default(),
                latest: Default::default(),
                versions: Default::default(),
            }
        };

        Ok(MetadataHandler {
            inner: Arc::new(Mutex::new(inner))
        })
    }

    pub fn increment_download(&self, d: ExtensionIdentifier) {
        let mut inner = self.inner.lock().unwrap();

        let downloads = &mut inner.downloads;
        let increment = downloads.get(&d.as_key()).unwrap_or(&0) + 1;
        downloads.insert(d.as_key(), increment);
    }

    pub fn new_version(&self, id: ExtensionIdentifier, version: String) -> HttpResult<()> {
        let mut inner = self.inner.lock().unwrap();
        let class = VersionType::classify(&version)?;

        if !inner.versions.contains_key(&id.as_key()) {
            let vec = Vec::new();
            inner.versions.insert(id.as_key(), vec);
        }

        let versions = inner.versions.get_mut(&id.as_key()).unwrap();
        versions.push(version.clone());

        let mut old = inner.latest.get_mut(&id.as_key())
            .map(|it| it.clone())
            .unwrap_or(
                LatestVersion {
                    release: None,
                    beta: None,
                    rc: None,
                }
            );

        match class {
            VersionType::Release => {
                old.release = Some(version)
            }
            VersionType::Beta => {
                old.beta = Some(version)
            }
            VersionType::ReleaseCandidate => {
                old.rc = Some(version)
            }
        }

        let latest = &mut inner.latest;
        latest.insert(id.as_key(), old);

        Ok(())
    }

    pub fn get_managed_metadata(
        &self,
        identifier: &ExtensionIdentifier,
    ) -> HttpResult<(u32, LatestVersion, Vec<String>)> {
        let mut inner = self.inner.lock().unwrap();

        Ok((
            inner.downloads.get(&identifier.as_key()).unwrap_or(&0).clone(),
            inner.latest.get(&identifier.as_key()).unwrap_or(&Default::default()).clone(),
            inner.versions.get(&identifier.as_key()).unwrap_or(&Vec::new()).clone()
        ))
    }

    pub fn extension_count(&self) -> u32 {
        let mut inner = self.inner.lock().unwrap();


        inner.versions.keys().len() as u32
    }
}