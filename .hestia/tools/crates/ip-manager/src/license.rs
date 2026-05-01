use crate::error::IpError;
use crate::types::{IpCore, LicenseClassification};

pub struct LicenseChecker;

impl LicenseChecker {
    pub fn classify(license_name: &str) -> LicenseClassification {
        let oss_licenses = [
            "MIT",
            "Apache-2.0",
            "BSD-2-Clause",
            "BSD-3-Clause",
            "GPL-2.0",
            "GPL-3.0",
            "ISC",
            "CC0-1.0",
        ];
        let proprietary_keywords = ["FlexLM", "seat-restricted", "node-locked"];

        if oss_licenses
            .iter()
            .any(|l| l.eq_ignore_ascii_case(license_name))
        {
            LicenseClassification::Oss {
                license: license_name.to_string(),
            }
        } else if proprietary_keywords
            .iter()
            .any(|k| license_name.contains(k))
        {
            LicenseClassification::VendorProprietary {
                terms_accepted: false,
            }
        } else {
            LicenseClassification::Unknown
        }
    }

    pub fn validate(ip: &IpCore) -> Result<(), IpError> {
        match &ip.license {
            LicenseClassification::Oss { .. } => Ok(()),
            LicenseClassification::VendorProprietary { terms_accepted: true } => Ok(()),
            LicenseClassification::VendorProprietary { terms_accepted: false } => {
                Err(IpError::LicenseNotAccepted(ip.id.clone()))
            }
            LicenseClassification::Unknown => {
                Err(IpError::LicenseNotAccepted(format!(
                    "Unknown license for {}",
                    ip.id
                )))
            }
        }
    }
}