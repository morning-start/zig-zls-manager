use regex::Regex;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

use crate::utils::error::ZzmError;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Version {
    pub major: u64,
    pub minor: u64,
    pub patch: u64,
    pub pre_release: Option<String>,
}

impl Version {
    #[allow(dead_code)] // 预留: 版本构造 API
    pub fn new(major: u64, minor: u64, patch: u64) -> Self {
        Self {
            major,
            minor,
            patch,
            pre_release: None,
        }
    }

    #[allow(dead_code)] // 预留: 预发布版本构建
    pub fn with_pre(mut self, pre: &str) -> Self {
        self.pre_release = Some(pre.to_string());
        self
    }

    #[allow(dead_code)] // 预留: 版本分类查询
    pub fn is_stable(&self) -> bool {
        self.pre_release.is_none()
    }
}

impl std::fmt::Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)?;
        if let Some(ref pre) = self.pre_release {
            write!(f, "-{}", pre)?;
        }
        Ok(())
    }
}

impl FromStr for Version {
    type Err = crate::utils::error::ZzmError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let s = s.trim();

        match s.to_lowercase().as_str() {
            "master" | "nightly" => {
                return Ok(Self {
                    major: 0,
                    minor: 0,
                    patch: 0,
                    pre_release: Some("master".to_string()),
                });
            }
            "stable" => {
                return Ok(Self {
                    major: 0,
                    minor: 0,
                    patch: 0,
                    pre_release: Some("stable".to_string()),
                });
            }
            _ => {}
        }

        let re = Regex::new(r"^(\d+)\.(\d+)(?:\.(\d+))?(?:-(.+))?$").map_err(|_| {
            ZzmError::InvalidVersion {
                version: s.to_string(),
            }
        })?;

        let caps = re.captures(s).ok_or_else(|| ZzmError::InvalidVersion {
            version: s.to_string(),
        })?;

        let major = caps[1]
            .parse::<u64>()
            .map_err(|_| ZzmError::InvalidVersion {
                version: s.to_string(),
            })?;

        let minor = caps[2]
            .parse::<u64>()
            .map_err(|_| ZzmError::InvalidVersion {
                version: s.to_string(),
            })?;

        let patch = caps
            .get(3)
            .map(|m| m.as_str().parse::<u64>())
            .transpose()
            .map_err(|_| ZzmError::InvalidVersion {
                version: s.to_string(),
            })?
            .unwrap_or(0);

        let pre_release = caps.get(4).map(|m| m.as_str().to_string());

        Ok(Self {
            major,
            minor,
            patch,
            pre_release,
        })
    }
}

pub fn resolve_version(input: &str) -> std::result::Result<String, ZzmError> {
    let input = input.trim();

    match input {
        "master" | "nightly" | "stable" | "latest" => Ok(input.to_string()),
        _ => {
            if let Ok(version) = Version::from_str(input) {
                if version.patch == 0 && !input.contains(".0") {
                    Ok(format!("{}.0", input))
                } else {
                    Ok(input.to_string())
                }
            } else if input.starts_with('.') {
                let ver = format!("0{}", input);
                resolve_version(&ver)
            } else {
                Err(ZzmError::InvalidVersion {
                    version: input.to_string(),
                })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_parse() {
        let v = Version::from_str("0.13.0").unwrap();
        assert_eq!(v.major, 0);
        assert_eq!(v.minor, 13);
        assert_eq!(v.patch, 0);
    }

    #[test]
    fn test_version_display() {
        let v = Version::new(0, 13, 0);
        assert_eq!(v.to_string(), "0.13.0");
    }

    #[test]
    fn test_special_versions() {
        let v = Version::from_str("master").unwrap();
        assert_eq!(v.pre_release, Some("master".to_string()));

        let v = Version::from_str("stable").unwrap();
        assert_eq!(v.pre_release, Some("stable".to_string()));
    }

    #[test]
    fn test_resolve_version() {
        assert_eq!(resolve_version("0.13").unwrap(), "0.13.0");
        assert_eq!(resolve_version("0.13.0").unwrap(), "0.13.0");
        assert_eq!(resolve_version("master").unwrap(), "master");
        assert_eq!(resolve_version(".13").unwrap(), "0.13.0");
    }

    #[test]
    fn test_version_parse_with_pre_release() {
        let v = Version::from_str("0.13.0-dev").unwrap();
        assert_eq!(v.major, 0);
        assert_eq!(v.minor, 13);
        assert_eq!(v.patch, 0);
        assert_eq!(v.pre_release, Some("dev".to_string()));
    }

    #[test]
    fn test_version_display_with_pre_release() {
        let v = Version::new(0, 13, 0).with_pre("rc1");
        assert_eq!(v.to_string(), "0.13.0-rc1");
    }

    #[test]
    fn test_version_is_stable() {
        let stable = Version::new(0, 13, 0);
        assert!(stable.is_stable());

        let pre = Version::new(0, 13, 0).with_pre("dev");
        assert!(!pre.is_stable());

        let master: Version = "master".parse().unwrap();
        assert!(!master.is_stable());
    }

    #[test]
    fn test_version_new_constructor() {
        let v = Version::new(1, 2, 3);
        assert_eq!(v.major, 1);
        assert_eq!(v.minor, 2);
        assert_eq!(v.patch, 3);
        assert!(v.pre_release.is_none());
    }

    #[test]
    fn test_version_with_pre_builder() {
        let v = Version::new(1, 2, 3).with_pre("beta.1");
        assert_eq!(v.pre_release, Some("beta.1".to_string()));
    }

    #[test]
    fn test_version_parse_invalid() {
        assert!(Version::from_str("abc").is_err());
        assert!(Version::from_str("").is_err());
        assert!(Version::from_str("0.").is_err());
        assert!(Version::from_str(".13.").is_err());
        assert!(Version::from_str("1").is_err());
    }

    #[test]
    fn test_version_parse_two_part() {
        let v = Version::from_str("0.13").unwrap();
        assert_eq!(v.major, 0);
        assert_eq!(v.minor, 13);
        assert_eq!(v.patch, 0);
        assert!(v.pre_release.is_none());
    }

    #[test]
    fn test_version_parse_nightly() {
        let v = Version::from_str("nightly").unwrap();
        assert_eq!(v.pre_release, Some("master".to_string()));
        assert_eq!(v.major, 0);
    }

    #[test]
    fn test_version_equality() {
        let v1 = Version::new(0, 13, 0);
        let v2 = Version::new(0, 13, 0);
        assert_eq!(v1, v2);

        let v3 = Version::new(0, 13, 1);
        assert_ne!(v1, v3);
    }

    #[test]
    fn test_version_ordering() {
        let v1 = Version::new(0, 12, 0);
        let v2 = Version::new(0, 13, 0);
        let v3 = Version::new(1, 0, 0);

        assert!(v1 < v2);
        assert!(v2 < v3);
        assert!(v1 < v3);
    }

    #[test]
    fn test_resolve_version_latest() {
        assert_eq!(resolve_version("latest").unwrap(), "latest");
    }

    #[test]
    fn test_resolve_version_stable() {
        assert_eq!(resolve_version("stable").unwrap(), "stable");
    }

    #[test]
    fn test_resolve_version_nightly() {
        assert_eq!(resolve_version("nightly").unwrap(), "nightly");
    }

    #[test]
    fn test_resolve_version_dot_prefix() {
        assert_eq!(resolve_version(".13").unwrap(), "0.13.0");
        assert_eq!(resolve_version(".12.1").unwrap(), "0.12.1");
    }

    #[test]
    fn test_resolve_version_invalid() {
        assert!(resolve_version("abc").is_err());
        assert!(resolve_version("").is_err());
        assert!(resolve_version("0.").is_err());
    }

    #[test]
    fn test_version_serde_roundtrip() {
        let v = Version::new(0, 13, 0);
        let json = serde_json::to_string(&v).unwrap();
        let parsed: Version = serde_json::from_str(&json).unwrap();
        assert_eq!(v, parsed);
    }

    #[test]
    fn test_version_hash_consistency() {
        use std::collections::HashSet;
        let v1 = Version::new(0, 13, 0);
        let v2 = Version::new(0, 13, 0);
        let v3 = Version::new(0, 13, 1);

        let mut set = HashSet::new();
        set.insert(v1.clone());
        set.insert(v2.clone());
        set.insert(v3.clone());
        assert_eq!(set.len(), 2);
    }
}
