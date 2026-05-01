use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum IpFileType {
    Rtl,
    Testbench,
    Doc,
    Constraint,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum IpLanguage {
    Verilog,
    SystemVerilog,
    Vhdl,
    Chisel,
    Amaranth,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct IpFile {
    pub path: String,
    pub file_type: IpFileType,
    pub language: IpLanguage,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct IpParameter {
    pub name: String,
    pub param_type: String,
    pub default: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct IpDependency {
    pub ip_id: String,
    #[serde(
        serialize_with = "serialize_version_req",
        deserialize_with = "deserialize_version_req"
    )]
    pub version_req: semver::VersionReq,
    pub optional: bool,
}

fn serialize_version_req<S>(v: &semver::VersionReq, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_str(&v.to_string())
}

fn deserialize_version_req<'de, D>(deserializer: D) -> Result<semver::VersionReq, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    semver::VersionReq::parse(&s).map_err(serde::de::Error::custom)
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum LicenseClassification {
    Oss { license: String },
    VendorProprietary { terms_accepted: bool },
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct IpCore {
    pub id: String,
    #[serde(
        serialize_with = "serialize_version",
        deserialize_with = "deserialize_version"
    )]
    pub version: semver::Version,
    pub vendor: String,
    pub library: String,
    pub device_families: Vec<String>,
    pub supported_languages: Vec<String>,
    pub dependencies: Vec<IpDependency>,
    pub files: Vec<IpFile>,
    pub parameters: Vec<IpParameter>,
    pub license: LicenseClassification,
}

fn serialize_version<S>(v: &semver::Version, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_str(&v.to_string())
}

fn deserialize_version<'de, D>(deserializer: D) -> Result<semver::Version, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    semver::Version::parse(&s).map_err(serde::de::Error::custom)
}