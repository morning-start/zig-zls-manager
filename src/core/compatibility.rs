use crate::output::console_output;

/// 兼容性状态
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompatibilityStatus {
    /// 完全兼容
    Compatible,
    /// 可能兼容（次要版本匹配，补丁版本不同）
    LikelyCompatible { reason: String },
    /// 不兼容
    Incompatible { reason: String },
    /// 未知（无法判断）
    Unknown { reason: String },
}

/// 兼容性检查器
///
/// 基于 Zig/ZLS 版本对应规则检查版本兼容性
pub struct CompatibilityChecker;

impl CompatibilityChecker {
    /// 检查 Zig 版本与 ZLS 版本的兼容性
    ///
    /// 兼容性规则：
    /// - ZLS 的版本号通常与 Zig 的版本号一一对应（如 Zig 0.13.0 ↔ ZLS 0.13.0）
    /// - 主版本号+次版本号匹配即为兼容（如 Zig 0.13.0 ↔ ZLS 0.13.1）
    /// - master/nightly 版本的 ZLS 兼容 master/nightly 版本的 Zig
    pub fn check(zig_version: &str, zls_version: &str) -> CompatibilityStatus {
        // 处理特殊版本号
        if zig_version == "master" || zig_version == "nightly" {
            if zls_version == "master" || zls_version == "nightly" {
                return CompatibilityStatus::Compatible;
            }
            return CompatibilityStatus::Incompatible {
                reason: format!(
                    "Zig {} (nightly) 应搭配 ZLS master/nightly 版本，当前 ZLS {} 是稳定版",
                    zig_version, zls_version
                ),
            };
        }

        if zls_version == "master" || zls_version == "nightly" {
            return CompatibilityStatus::LikelyCompatible {
                reason: format!(
                    "ZLS {} (nightly) 通常兼容最新的 Zig 版本，但与 Zig {} 可能不完全匹配",
                    zls_version, zig_version
                ),
            };
        }

        // 解析 Zig 版本
        let zig_parts = Self::parse_version_parts(zig_version);
        let zls_parts = Self::parse_version_parts(zls_version);

        match (zig_parts, zls_parts) {
            (Some(zig), Some(zls)) => {
                // 精确匹配
                if zig.major == zls.major && zig.minor == zls.minor && zig.patch == zls.patch {
                    return CompatibilityStatus::Compatible;
                }

                // 主版本号+次版本号匹配
                if zig.major == zls.major && zig.minor == zls.minor {
                    return CompatibilityStatus::LikelyCompatible {
                        reason: format!(
                            "Zig {}.{}.{} 与 ZLS {}.{}.{} 次版本号匹配，补丁版本不同",
                            zig.major, zig.minor, zig.patch,
                            zls.major, zls.minor, zls.patch
                        ),
                    };
                }

                // 主版本号不匹配
                CompatibilityStatus::Incompatible {
                    reason: format!(
                        "Zig {}.{}.{} 与 ZLS {}.{}.{} 版本号不匹配",
                        zig.major, zig.minor, zig.patch,
                        zls.major, zls.minor, zls.patch
                    ),
                }
            }
            _ => CompatibilityStatus::Unknown {
                reason: format!(
                    "无法解析版本号: Zig={}, ZLS={}",
                    zig_version, zls_version
                ),
            },
        }
    }

    /// 检查兼容性并输出警告（不阻止操作）
    #[allow(dead_code)] // 预留: 在 use/switch 命令中调用
    pub fn check_and_warn(zig_version: &str, zls_version: &str) {
        match Self::check(zig_version, zls_version) {
            CompatibilityStatus::Compatible => {}
            CompatibilityStatus::LikelyCompatible { reason } => {
                console_output::print_warning(&format!(
                    "Zig {} 与 ZLS {} 可能不完全兼容: {}",
                    zig_version, zls_version, reason
                ));
            }
            CompatibilityStatus::Incompatible { reason } => {
                console_output::print_warning(&format!(
                    "Zig {} 与 ZLS {} 不兼容: {}",
                    zig_version, zls_version, reason
                ));
            }
            CompatibilityStatus::Unknown { reason } => {
                console_output::print_warning(&format!(
                    "无法确认 Zig {} 与 ZLS {} 的兼容性: {}",
                    zig_version, zls_version, reason
                ));
            }
        }
    }

    /// 获取推荐搭配的 ZLS 版本
    ///
    /// 根据内置规则推荐与给定 Zig 版本最匹配的 ZLS 版本
    pub fn recommended_zls_version(zig_version: &str) -> Option<String> {
        // 对于稳定版，推荐同版本号的 ZLS
        if zig_version != "master" && zig_version != "nightly" {
            let parts = Self::parse_version_parts(zig_version);
            if let Some(p) = parts {
                return Some(format!("{}.{}.{}", p.major, p.minor, p.patch));
            }
        }
        // 对于 nightly，推荐 master
        if zig_version == "master" || zig_version == "nightly" {
            return Some("master".to_string());
        }
        None
    }

    /// 解析版本号字符串为 (major, minor, patch)
    fn parse_version_parts(version: &str) -> Option<VersionParts> {
        let version = version.trim_start_matches('v');
        let parts: Vec<&str> = version.split('.').collect();
        if parts.len() >= 2 {
            let major = parts[0].parse::<u64>().ok()?;
            let minor = parts[1].parse::<u64>().ok()?;
            let patch = if parts.len() >= 3 {
                parts[2].parse::<u64>().unwrap_or(0)
            } else {
                0
            };
            Some(VersionParts {
                major,
                minor,
                patch,
            })
        } else {
            None
        }
    }
}

/// 版本号组成部分
struct VersionParts {
    major: u64,
    minor: u64,
    patch: u64,
}

// ========== 单元测试 ==========

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exact_match_compatible() {
        let status = CompatibilityChecker::check("0.13.0", "0.13.0");
        assert_eq!(status, CompatibilityStatus::Compatible);
    }

    #[test]
    fn test_minor_match_likely_compatible() {
        let status = CompatibilityChecker::check("0.13.0", "0.13.1");
        assert!(matches!(status, CompatibilityStatus::LikelyCompatible { .. }));
    }

    #[test]
    fn test_major_mismatch_incompatible() {
        let status = CompatibilityChecker::check("0.13.0", "0.12.0");
        assert!(matches!(status, CompatibilityStatus::Incompatible { .. }));
    }

    #[test]
    fn test_nightly_compatible() {
        let status = CompatibilityChecker::check("master", "master");
        assert_eq!(status, CompatibilityStatus::Compatible);
    }

    #[test]
    fn test_nightly_with_stable_incompatible() {
        let status = CompatibilityChecker::check("master", "0.13.0");
        assert!(matches!(status, CompatibilityStatus::Incompatible { .. }));
    }

    #[test]
    fn test_recommended_zls_version() {
        assert_eq!(
            CompatibilityChecker::recommended_zls_version("0.13.0"),
            Some("0.13.0".to_string())
        );
        assert_eq!(
            CompatibilityChecker::recommended_zls_version("master"),
            Some("master".to_string())
        );
    }
}