use crate::UpgradeError;

/// Policy governing allowed version transitions.
#[derive(Debug, Clone)]
pub struct VersionPolicy {
    /// If true, only allow major version zero (0.x.y) for pre-release.
    pub allow_prerelease: bool,
    /// If true, allow major version bumps.
    pub allow_major_bumps: bool,
    /// If true, require that minor and patch versions are non-decreasing.
    pub require_non_decreasing: bool,
}

impl Default for VersionPolicy {
    fn default() -> Self {
        Self {
            allow_prerelease: true,
            allow_major_bumps: true,
            require_non_decreasing: true,
        }
    }
}

/// A parsed semantic version.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct SemVer {
    pub major: u64,
    pub minor: u64,
    pub patch: u64,
}

impl SemVer {
    /// Parse a semver string like "1.2.3".
    pub fn parse(version: &str) -> Result<Self, UpgradeError> {
        let parts: Vec<&str> = version.trim().split('.').collect();
        if parts.len() != 3 {
            return Err(UpgradeError::InvalidSemver(format!(
                "expected MAJOR.MINOR.PATCH, got: {}",
                version
            )));
        }
        let major = parts[0]
            .parse::<u64>()
            .map_err(|_| UpgradeError::InvalidSemver(format!("invalid major: {}", parts[0])))?;
        let minor = parts[1]
            .parse::<u64>()
            .map_err(|_| UpgradeError::InvalidSemver(format!("invalid minor: {}", parts[1])))?;
        let patch = parts[2]
            .parse::<u64>()
            .map_err(|_| UpgradeError::InvalidSemver(format!("invalid patch: {}", parts[2])))?;
        Ok(Self {
            major,
            minor,
            patch,
        })
    }
}

impl std::fmt::Display for SemVer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

impl VersionPolicy {
    /// Check whether transitioning from `current` to `target` is allowed by this policy.
    pub fn check(&self, current: &SemVer, target: &SemVer) -> Result<(), UpgradeError> {
        // Disallow pre-release targets unless explicitly allowed.
        if !self.allow_prerelease && target.major == 0 {
            return Err(UpgradeError::VersionPolicyViolation(format!(
                "pre-release target {} not allowed",
                target
            )));
        }

        // Disallow major bumps unless allowed.
        if target.major > current.major && !self.allow_major_bumps {
            return Err(UpgradeError::VersionPolicyViolation(format!(
                "major bump from {} to {} not allowed",
                current, target
            )));
        }

        // Require non-decreasing versions.
        if self.require_non_decreasing && target < current {
            return Err(UpgradeError::VersionPolicyViolation(format!(
                "downgrade from {} to {} not allowed",
                current, target
            )));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_semver_parse() {
        let v = SemVer::parse("1.2.3").unwrap();
        assert_eq!(v.major, 1);
        assert_eq!(v.minor, 2);
        assert_eq!(v.patch, 3);
    }

    #[test]
    fn test_version_upgrade_allowed() {
        let policy = VersionPolicy::default();
        let current = SemVer::parse("1.0.0").unwrap();
        let target = SemVer::parse("1.1.0").unwrap();
        assert!(policy.check(&current, &target).is_ok());
    }

    #[test]
    fn test_downgrade_rejected() {
        let policy = VersionPolicy::default();
        let current = SemVer::parse("2.0.0").unwrap();
        let target = SemVer::parse("1.9.0").unwrap();
        assert!(policy.check(&current, &target).is_err());
    }
}
