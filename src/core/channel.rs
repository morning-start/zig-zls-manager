use serde::{Deserialize, Serialize};

/// 工具版本通道
///
/// 统一 Zig 和 ZLS 的通道概念，替代之前分散的
/// `infra::zig_api::ZigChannel` 和 `infra::zls_api::ZlsChannel`。
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Channel {
    /// 稳定发布版
    Stable,
    /// 开发版 / nightly（Zig 的 master）
    Nightly,
    /// 预发布版（ZLS 的 prerelease）
    Prerelease,
}

impl Channel {
    /// 转为显示字符串
    pub fn as_str(&self) -> &'static str {
        match self {
            Channel::Stable => "stable",
            Channel::Nightly => "nightly",
            Channel::Prerelease => "prerelease",
        }
    }
}

impl std::fmt::Display for Channel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

// ========== 单元测试 ==========

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_channel_as_str() {
        assert_eq!(Channel::Stable.as_str(), "stable");
        assert_eq!(Channel::Nightly.as_str(), "nightly");
        assert_eq!(Channel::Prerelease.as_str(), "prerelease");
    }

    #[test]
    fn test_channel_display() {
        assert_eq!(format!("{}", Channel::Stable), "stable");
        assert_eq!(format!("{}", Channel::Nightly), "nightly");
        assert_eq!(format!("{}", Channel::Prerelease), "prerelease");
    }

    #[test]
    fn test_channel_equality() {
        assert_eq!(Channel::Stable, Channel::Stable);
        assert_eq!(Channel::Nightly, Channel::Nightly);
        assert_ne!(Channel::Stable, Channel::Nightly);
        assert_ne!(Channel::Nightly, Channel::Prerelease);
    }

    #[test]
    fn test_channel_serde() {
        let stable = Channel::Stable;
        let json = serde_json::to_string(&stable).unwrap();
        assert!(json.contains("Stable"));

        let nightly = Channel::Nightly;
        let json = serde_json::to_string(&nightly).unwrap();
        assert!(json.contains("Nightly"));

        let prerelease = Channel::Prerelease;
        let json = serde_json::to_string(&prerelease).unwrap();
        assert!(json.contains("Prerelease"));
    }
}
