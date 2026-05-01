use thiserror::Error;

#[derive(Debug, Error)]
pub enum IpError {
    #[error("IP core not found: {0}")]
    NotFound(String),
    #[error("circular dependency detected: {0}")]
    CircularDependency(String),
    #[error("license not accepted for IP: {0}")]
    LicenseNotAccepted(String),
    #[error("version conflict for {ip_id}: required {required}, found {found}")]
    VersionConflict {
        ip_id: String,
        required: String,
        found: String,
    },
    #[error("duplicate IP core: {0}")]
    DuplicateIp(String),
}