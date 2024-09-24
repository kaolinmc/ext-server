use std::collections::HashMap;
use std::fmt::format;
use std::io::Read;

use rocket::http::Status;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::responses::{HandlerError, HttpResult};
use crate::types::VersionType::Release;

pub struct ExtensionBundle<T: Read> {
    pub runtime_model: ExtensionRuntimeModel,
    pub metadata: ExtensionMetadata,
    pub partitions: Vec<PartitionRuntimeModel>,
    pub files: Vec<(T, String)>
}

#[derive(PartialEq)]
pub enum VersionType {
    Release,
    Beta,
    ReleaseCandidate,
}

impl Serialize for VersionType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let version = Self::suffix(self);

        serializer.serialize_str(version)
    }
}

impl VersionType {
    pub fn classify<T: Into<String>>(version: T) -> HttpResult<VersionType> {
        let version = version.into();
        if let Some(pos) = version.find("-") {
            let suffix = &version[(pos + 1)..];
            let suffix = suffix.to_lowercase();
            match suffix.as_str() {
                "beta" => Ok(VersionType::Beta),
                "rc" => Ok(VersionType::Release),
                _ => Err(HandlerError::new(
                    "Invalid extension version".into(),
                    Some("Extension version suffix is invalid, must either end in '', '-BETA', or '-RC".into()),
                    Status::BadRequest,
                ))
            }
        } else {
            Ok(Release)
        }
    }

    pub fn suffix(&self) -> &'static str {
        match self {
            Release => "",
            VersionType::Beta => "beta",
            VersionType::ReleaseCandidate => "rc"
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct LatestVersion {
    pub release: Option<String>,
    pub beta: Option<String>,
    pub rc: Option<String>,
}

impl Default for LatestVersion {
    fn default() -> Self {
        LatestVersion {
            release: None,
            beta: None,
            rc: None,
        }
    }
}

#[derive(Serialize)]
pub struct VersionInfo {
    pub version: String,
    pub release_type: VersionType,
    pub metadata_path: String,
}

#[derive(Serialize)]
pub struct ManagedExtensionMetadata {
    pub downloads: u32,
    pub latest: LatestVersion,
    pub versions: Vec<VersionInfo>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct RepositoryMetadata {
    pub name: String,
    pub description: String,
    pub icon: Option<String>,
    pub extension_count: u32,
    pub app_ids: Vec<String>,
}

#[derive(Eq, PartialEq, Hash, Clone, Serialize, Deserialize)]
pub struct ExtensionIdentifier {
    pub group: String,
    pub name: String,
}

impl ExtensionIdentifier {
    pub fn as_key(&self) -> String {
        format!("{}:{}", self.group, self.name)
    }
}

impl From<&ExtensionRuntimeModel> for ExtensionIdentifier {
    fn from(value: &ExtensionRuntimeModel) -> Self {
        ExtensionIdentifier {
            group: value.group_id.clone(),
            name: value.name.clone(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ExtensionMetadata {
    pub name: String,
    pub developers: Vec<String>,
    pub icon: Option<String>,
    pub description: String,
    pub tags: Vec<String>,
    pub app: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ExtensionRuntimeModel {
    #[serde(alias = "apiVersion")]
    pub api_version: i32,
    #[serde(alias = "groupId")]
    pub group_id: String,
    pub name: String,
    pub version: String,
    pub repositories: Vec<HashMap<String, String>>,
    pub parents: Vec<ExtensionParent>,
    pub partitions: Vec<PartitionModelReference>,
}

// PartitionModelReference
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct PartitionModelReference {
    pub r#type: String,
    pub name: String,
}

// ExtensionParent
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct ExtensionParent {
    pub group: String,
    pub extension: String,
    pub version: String,
}

impl ExtensionParent {
    pub fn to_descriptor(&self) -> ExtensionDescriptor {
        ExtensionDescriptor {
            group: self.group.clone(),
            extension: self.extension.clone(),
            version: self.version.clone(),
        }
    }
}

// ExtensionDescriptor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtensionDescriptor {
    pub group: String,
    pub extension: String,
    pub version: String,
}

impl ExtensionDescriptor {
    pub fn parse_descriptor(descriptor: &str) -> Self {
        let parts: Vec<&str> = descriptor.split(':').collect();
        ExtensionDescriptor {
            group: parts[0].to_string(),
            extension: parts[1].to_string(),
            version: parts[2].to_string(),
        }
    }
}

// ExtensionRepository
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtensionRepository {
    pub r#type: String,
    pub settings: HashMap<String, String>,
}

// PartitionRuntimeModel
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartitionRuntimeModel {
    pub r#type: String,
    pub name: String,
    pub repositories: Vec<ExtensionRepository>,
    pub dependencies: Vec<HashMap<String, String>>,
    pub options: HashMap<String, String>,
}

#[derive(Serialize)]
pub struct SearchResponse {
    pub result: Vec<ExtensionIdentifier>,
}