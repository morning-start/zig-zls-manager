//! 集成测试：索引序列化/迁移 + 项目配置 + 兼容性检查 + 工具数据结构

use std::collections::HashMap;

use zzm::core::channel::Channel;
use zzm::core::compatibility::{CompatibilityChecker, CompatibilityStatus};
use zzm::core::project::{CompatibilityMode, ProjectConfig};
use zzm::core::tool_manager::{DownloadedAsset, ToolKind};
use zzm::infra::path_manager::{InstalledIndex, ToolExtraData, ToolIndexEntry};

// ========== InstalledIndex 序列化/迁移 ==========

#[test]
fn test_installed_index_new_format_roundtrip() {
    let mut index = InstalledIndex::default();
    let entry = ToolIndexEntry {
        version: "0.13.0".to_string(),
        install_path: "/home/.zzm/versions/zig/0.13.0".into(),
        installed_at: "2026-04-26T00:00:00Z".to_string(),
        extra: ToolExtraData::Zig {
            channel: Channel::Stable,
        },
    };
    index.get_versions_mut(ToolKind::Zig).push(entry);
    index.set_active(ToolKind::Zig, Some("0.13.0".to_string()));

    let json = serde_json::to_string(&index).unwrap();
    let parsed: InstalledIndex = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.get_versions(ToolKind::Zig).len(), 1);
    assert_eq!(parsed.get_active(ToolKind::Zig), Some("0.13.0"));
}

#[test]
fn test_installed_index_legacy_migration() {
    let legacy_json = r#"{
        "zig_versions": [{
            "version": "0.13.0",
            "install_path": "/home/.zzm/versions/zig/0.13.0",
            "installed_at": "2026-04-26T00:00:00Z",
            "channel": "stable"
        }],
        "zls_versions": [{
            "version": "0.13.0",
            "install_path": "/home/.zzm/versions/zls/0.13.0",
            "installed_at": "2026-04-26T00:00:00Z",
            "zig_version": "0.13.0"
        }],
        "active_zig": "0.13.0",
        "active_zls": "0.13.0"
    }"#;

    let index = InstalledIndex::from_json_str(legacy_json).unwrap();
    assert_eq!(index.get_versions(ToolKind::Zig).len(), 1);
    assert_eq!(index.get_versions(ToolKind::Zls).len(), 1);
    assert_eq!(index.get_active(ToolKind::Zig), Some("0.13.0"));
    assert_eq!(index.get_active(ToolKind::Zls), Some("0.13.0"));

    // Zig entry should have channel
    let zig_entry = &index.get_versions(ToolKind::Zig)[0];
    assert!(matches!(zig_entry.extra, ToolExtraData::Zig { channel: Channel::Stable }));

    // ZLS entry should have zig_version
    let zls_entry = &index.get_versions(ToolKind::Zls)[0];
    assert!(matches!(&zls_entry.extra, ToolExtraData::Zls { zig_version } if zig_version.as_deref() == Some("0.13.0")));
}

#[test]
fn test_installed_index_empty_json() {
    let index = InstalledIndex::from_json_str("{}").unwrap();
    assert!(index.get_versions(ToolKind::Zig).is_empty());
    assert!(index.get_versions(ToolKind::Zls).is_empty());
    assert_eq!(index.get_active(ToolKind::Zig), None);
}

#[test]
fn test_installed_index_multiple_versions() {
    let mut index = InstalledIndex::default();

    for v in ["0.13.0", "0.14.0", "0.15.0"] {
        index.get_versions_mut(ToolKind::Zig).push(ToolIndexEntry {
            version: v.to_string(),
            install_path: format!("/home/.zzm/versions/zig/{v}").into(),
            installed_at: "2026-04-26T00:00:00Z".to_string(),
            extra: ToolExtraData::Zig {
                channel: Channel::Stable,
            },
        });
    }
    index.set_active(ToolKind::Zig, Some("0.14.0".to_string()));

    assert_eq!(index.get_versions(ToolKind::Zig).len(), 3);
    assert_eq!(index.get_active(ToolKind::Zig), Some("0.14.0"));

    // 删除一个版本
    index.get_versions_mut(ToolKind::Zig).retain(|e| e.version != "0.14.0");
    assert_eq!(index.get_versions(ToolKind::Zig).len(), 2);
}

// ========== ProjectConfig 序列化 ==========

#[test]
fn test_project_config_json_roundtrip() {
    let config = ProjectConfig {
        zig: "0.13.0".to_string(),
        zls: Some("0.13.0".to_string()),
        compatibility: CompatibilityMode::Strict,
        notes: Some("test project".to_string()),
    };

    let json = serde_json::to_string(&config).unwrap();
    let parsed: ProjectConfig = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.zig, "0.13.0");
    assert_eq!(parsed.zls, Some("0.13.0".to_string()));
    assert_eq!(parsed.compatibility, CompatibilityMode::Strict);
    assert_eq!(parsed.notes, Some("test project".to_string()));
}

#[test]
fn test_project_config_minimal() {
    let json = r#"{"zig":"0.15.0"}"#;
    let config: ProjectConfig = serde_json::from_str(json).unwrap();
    assert_eq!(config.zig, "0.15.0");
    assert_eq!(config.zls, None);
    assert_eq!(config.compatibility, CompatibilityMode::Strict); // default
    assert_eq!(config.notes, None);
}

#[test]
fn test_project_config_compatibility_modes() {
    for mode in [CompatibilityMode::Strict, CompatibilityMode::Loose, CompatibilityMode::Auto] {
        let config = ProjectConfig {
            zig: "0.13.0".to_string(),
            zls: None,
            compatibility: mode.clone(),
            notes: None,
        };
        let json = serde_json::to_string(&config).unwrap();
        let parsed: ProjectConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.compatibility, mode);
    }
}

#[test]
fn test_project_config_file_persistence() {
    let dir = tempfile::tempdir().unwrap();
    let config_path = dir.path().join(".zzmrc");

    let config = ProjectConfig {
        zig: "0.16.0".to_string(),
        zls: Some("0.16.0".to_string()),
        compatibility: CompatibilityMode::Auto,
        notes: None,
    };

    // 写入
    let json = serde_json::to_string_pretty(&config).unwrap();
    std::fs::write(&config_path, &json).unwrap();

    // 读取
    let content = std::fs::read_to_string(&config_path).unwrap();
    let loaded: ProjectConfig = serde_json::from_str(&content).unwrap();
    assert_eq!(loaded.zig, "0.16.0");
    assert_eq!(loaded.zls, Some("0.16.0".to_string()));
    assert_eq!(loaded.compatibility, CompatibilityMode::Auto);
}

// ========== 兼容性检查 ==========

#[test]
fn test_compatibility_exact_match() {
    let status = CompatibilityChecker::check("0.13.0", "0.13.0");
    assert!(matches!(status, CompatibilityStatus::Compatible));
}

#[test]
fn test_compatibility_minor_match() {
    let status = CompatibilityChecker::check("0.13.0", "0.13.1");
    assert!(matches!(status, CompatibilityStatus::LikelyCompatible { .. }));
}

#[test]
fn test_compatibility_incompatible() {
    let status = CompatibilityChecker::check("0.12.0", "0.13.0");
    assert!(matches!(status, CompatibilityStatus::Incompatible { .. }));
}

#[test]
fn test_compatibility_nightly_pair() {
    let status = CompatibilityChecker::check("master", "master");
    assert!(matches!(status, CompatibilityStatus::Compatible));

    let status = CompatibilityChecker::check("nightly", "master");
    assert!(matches!(status, CompatibilityStatus::Compatible));
}

#[test]
fn test_compatibility_nightly_stable_mismatch() {
    let status = CompatibilityChecker::check("master", "0.13.0");
    assert!(matches!(status, CompatibilityStatus::Incompatible { .. }));
}

#[test]
fn test_compatibility_stable_nightly() {
    let status = CompatibilityChecker::check("0.13.0", "master");
    assert!(matches!(status, CompatibilityStatus::LikelyCompatible { .. }));
}

// ========== ToolKind HashMap key ==========

#[test]
fn test_tool_kind_as_hashmap_key() {
    let mut map = HashMap::new();
    map.insert(ToolKind::Zig, vec!["0.13.0"]);
    map.insert(ToolKind::Zls, vec!["0.13.0"]);

    assert_eq!(map.get(&ToolKind::Zig), Some(&vec!["0.13.0"]));
    assert_eq!(map.get(&ToolKind::Zls), Some(&vec!["0.13.0"]));
}

// ========== DownloadedAsset 结构验证 ==========

#[test]
fn test_downloaded_asset_fields() {
    let asset = DownloadedAsset {
        resolved: "0.13.0".to_string(),
        archive_path: std::path::PathBuf::from("/cache/zig-0.13.0.tar.xz"),
        channel: Channel::Stable,
        shasum: "abc123".to_string(),
    };

    assert_eq!(asset.resolved, "0.13.0");
    assert_eq!(
        asset.archive_path,
        std::path::PathBuf::from("/cache/zig-0.13.0.tar.xz")
    );
    assert!(matches!(asset.channel, Channel::Stable));
    assert_eq!(asset.shasum, "abc123");
}