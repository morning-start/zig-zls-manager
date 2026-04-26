use crate::core::callbacks::InstallCallbacks;
use crate::utils::version::Version;

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
                    "Zig {zig_version} (nightly) 应搭配 ZLS master/nightly 版本，当前 ZLS {zls_version} 是稳定版"
                ),
            };
        }

        if zls_version == "master" || zls_version == "nightly" {
            return CompatibilityStatus::LikelyCompatible {
                reason: format!(
                    "ZLS {zls_version} (nightly) 通常兼容最新的 Zig 版本，但与 Zig {zig_version} 可能不完全匹配"
                ),
            };
        }

        // 使用统一的 Version 解析
        let zig_ver: Result<Version, _> = zig_version.trim_start_matches('v').parse();
        let zls_ver: Result<Version, _> = zls_version.trim_start_matches('v').parse();

        match (zig_ver, zls_ver) {
            (Ok(zig), Ok(zls)) => {
                // 精确匹配
                if zig.major == zls.major && zig.minor == zls.minor && zig.patch == zls.patch {
                    return CompatibilityStatus::Compatible;
                }

                // 主版本号+次版本号匹配
                if zig.major == zls.major && zig.minor == zls.minor {
                    return CompatibilityStatus::LikelyCompatible {
                        reason: format!(
                            "Zig {}.{}.{} 与 ZLS {}.{}.{} 次版本号匹配，补丁版本不同",
                            zig.major, zig.minor, zig.patch, zls.major, zls.minor, zls.patch
                        ),
                    };
                }

                // 主版本号不匹配
                CompatibilityStatus::Incompatible {
                    reason: format!(
                        "Zig {}.{}.{} 与 ZLS {}.{}.{} 版本号不匹配",
                        zig.major, zig.minor, zig.patch, zls.major, zls.minor, zls.patch
                    ),
                }
            }
            _ => CompatibilityStatus::Unknown {
                reason: format!("无法解析版本号: Zig={zig_version}, ZLS={zls_version}"),
            },
        }
    }

    /// 检查兼容性并输出警告（不阻止操作）
    #[allow(dead_code)] // 预留: 在 use/switch 命令中调用
    pub fn check_and_warn(zig_version: &str, zls_version: &str, callbacks: &InstallCallbacks) {
        match Self::check(zig_version, zls_version) {
            CompatibilityStatus::Compatible => {}
            CompatibilityStatus::LikelyCompatible { reason } => {
                (callbacks.on_warning)(&format!(
                    "Zig {zig_version} 与 ZLS {zls_version} 可能不完全兼容: {reason}"
                ));
            }
            CompatibilityStatus::Incompatible { reason } => {
                (callbacks.on_warning)(&format!(
                    "Zig {zig_version} 与 ZLS {zls_version} 不兼容: {reason}"
                ));
            }
            CompatibilityStatus::Unknown { reason } => {
                (callbacks.on_warning)(&format!(
                    "无法确认 Zig {zig_version} 与 ZLS {zls_version} 的兼容性: {reason}"
                ));
            }
        }
    }

    /// 检查兼容性并输出警告（使用默认控制台输出）
    ///
    /// 便捷方法，适用于不需要自定义回调的场景
    #[allow(dead_code)]
    pub fn check_and_warn_console(zig_version: &str, zls_version: &str) {
        Self::check_and_warn(zig_version, zls_version, &InstallCallbacks::console());
    }

    /// 获取推荐搭配的 ZLS 版本
    ///
    /// 根据内置规则推荐与给定 Zig 版本最匹配的 ZLS 版本
    pub fn recommended_zls_version(zig_version: &str) -> Option<String> {
        // 对于稳定版，推荐同版本号的 ZLS
        if zig_version != "master" && zig_version != "nightly" {
            let ver: Result<Version, _> = zig_version.trim_start_matches('v').parse();
            if let Ok(v) = ver {
                return Some(v.to_string());
            }
        }
        // 对于 nightly，推荐 master
        if zig_version == "master" || zig_version == "nightly" {
            return Some("master".to_string());
        }
        None
    }
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
        assert!(matches!(
            status,
            CompatibilityStatus::LikelyCompatible { .. }
        ));
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

    #[test]
    fn test_nightly_zls_with_stable_zig() {
        // ZLS nightly 通常兼容最新的 Zig 稳定版
        let status = CompatibilityChecker::check("0.13.0", "master");
        assert!(matches!(
            status,
            CompatibilityStatus::LikelyCompatible { .. }
        ));
    }

    #[test]
    fn test_stable_zig_with_nightly_zls() {
        let status = CompatibilityChecker::check("0.13.0", "nightly");
        assert!(matches!(
            status,
            CompatibilityStatus::LikelyCompatible { .. }
        ));
    }

    #[test]
    fn test_minor_version_mismatch() {
        // 次版本号不匹配 → 不兼容
        let status = CompatibilityChecker::check("0.13.0", "0.14.0");
        assert!(matches!(status, CompatibilityStatus::Incompatible { .. }));
    }

    #[test]
    fn test_v_prefix_version() {
        // 支持 v 前缀的版本号
        let status = CompatibilityChecker::check("v0.13.0", "v0.13.0");
        assert_eq!(status, CompatibilityStatus::Compatible);
    }

    #[test]
    fn test_two_part_version() {
        // 两段版本号（如 0.13）
        let status = CompatibilityChecker::check("0.13", "0.13.0");
        assert_eq!(status, CompatibilityStatus::Compatible);
    }

    #[test]
    fn test_unparseable_version() {
        // 无法解析的版本号 → Unknown
        let status = CompatibilityChecker::check("abc", "0.13.0");
        assert!(matches!(status, CompatibilityStatus::Unknown { .. }));
    }

    #[test]
    fn test_recommended_zls_version_nightly() {
        assert_eq!(
            CompatibilityChecker::recommended_zls_version("nightly"),
            Some("master".to_string())
        );
    }

    #[test]
    fn test_recommended_zls_version_two_part() {
        assert_eq!(
            CompatibilityChecker::recommended_zls_version("0.13"),
            Some("0.13.0".to_string())
        );
    }

    #[test]
    fn test_patch_difference_likely_compatible() {
        let status = CompatibilityChecker::check("0.13.1", "0.13.2");
        assert!(matches!(
            status,
            CompatibilityStatus::LikelyCompatible { .. }
        ));
    }

    #[test]
    fn test_compatible_status_debug() {
        let status = CompatibilityStatus::Compatible;
        let debug_str = format!("{:?}", status);
        assert!(debug_str.contains("Compatible"));
    }
}
