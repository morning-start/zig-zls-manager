# 外部 API 参考文档

## 📋 文档信息

- **版本**: v1.1.0
- **创建日期**: 2026-04-24
- **状态**: 已验证（基于源码实现对照）
- **关联文档**: [architecture.md](./architecture.md) (技术架构设计文档)

---

## 1. 概述

本文档详细记录了 zig-zls-manager 项目所依赖的外部 API 接口，包括：

- **Zig 官方下载系统**：用于获取 Zig 编译器的版本信息和下载链接
- **ZLS GitHub Releases**：用于获取 ZLS 语言服务器的版本信息和预编译二进制

这些 API 是 zzm 工具实现版本查询、下载和管理的核心数据源。

---

## 2. Zig 官方下载 API

### 2.1 基本信息

| 属性 | 值 |
|-----|---|
| **官方网站** | https://ziglang.org/ |
| **下载页面** | https://ziglang.org/download/ |
| **JSON API** | https://ziglang.org/download/index.json |
| **维护者** | Zig 语言基金会 |
| **认证方式** | 无需认证（公开 API） |
| **请求频率限制** | 建议缓存结果，避免频繁请求 |

### 2.2 Zig 下载页面说明

Zig 官方下载页面提供以下内容：

- **最新版本信息**：包括 stable 版本和 master (nightly) 版本
- **多平台支持**：
  - Windows (x86_64, aarch64, x86)
  - macOS (x86_64, aarch64)
  - Linux (x86_64, aarch64, arm, riscv64)
- **文件格式**：
  - Source: 源码包 (.tar.xz)
  - Bootstrap: 引导构建包 (.tar.xz)
  - 预编译二进制: 平台特定的压缩包
- **校验机制**：提供 minisign 签名文件用于完整性验证

### 2.3 JSON API 结构详解

**端点**: `GET https://ziglang.org/download/index.json`

#### 完整 JSON 结构示例

```json
{
  "master": {
    "date": "2026-04-22",
    "docs": {
      "Language Reference": "https://ziglang.org/documentation/master/reference/",
      "Standard Library Documentation": "https://ziglang.org/documentation/master/std/"
    },
    "releases": [
      {
        "type": "Source",
        "target": null,
        "filename": "zig-0.17.0-dev.87+9b177a7d2.tar.xz",
        "size": "21MiB",
        "shasum": "abc123...",
        "signature": {
          "type": "minisig",
          "file": "zig-0.17.0-dev.87+9b177a7d2.tar.xz.minisig"
        },
        "url": "https://ziglang.org/download/0.17.0-dev.87+9b177a7d2/zig-0.17.0-dev.87+9b177a7d2.tar.xz"
      }
    ],
    "platforms": {
      "windows": [
        {
          "os": "Windows",
          "arch": "x86_64",
          "filename": "zig-x86_64-windows-0.17.0-dev.87+9b177a7d2.zip",
          "size": "93MiB",
          "shasum": "def456...",
          "signature": {
            "type": "minisig",
            "file": "zig-x86_64-windows-0.17.0-dev.87+9b177a7d2.zip.minisig"
          },
          "url": "https://ziglang.org/builds/zig-x86_64-windows-0.17.0-dev.87+9b177a7d2.zip"
        },
        {
          "os": "Windows",
          "arch": "aarch64",
          "filename": "zig-aarch64-windows-0.17.0-dev.87+9b177a7d2.zip",
          "size": "89MiB",
          "shasum": "ghi789...",
          "url": "https://ziglang.org/builds/zig-aarch64-windows-0.17.0-dev.87+9b177a7d2.zip"
        }
      ],
      "macos": [
        {
          "os": "macOS",
          "arch": "x86_64",
          "filename": "zig-x86_64-macos-0.17.0-dev.87+9b177a7d2.tar.xz",
          "size": "55MiB",
          "shasum": "jkl012...",
          "url": "https://ziglang.org/builds/zig-x86_64-macos-0.17.0-dev.87+9b177a7d2.tar.xz"
        },
        {
          "os": "macOS",
          "arch": "aarch64",
          "filename": "zig-aarch64-macos-0.17.0-dev.87+9b177a7d2.tar.xz",
          "size": "50MiB",
          "shasum": "mno345...",
          "url": "https://ziglang.org/builds/zig-aarch64-macos-0.17.0-dev.87+9b177a7d2.tar.xz"
        }
      ],
      "linux": [
        {
          "os": "Linux",
          "arch": "x86_64",
          "filename": "zig-x86_64-linux-0.17.0-dev.87+9b177a7d2.tar.xz",
          "size": "53MiB",
          "shasum": "pqr678...",
          "url": "https://ziglang.org/builds/zig-x86_64-linux-0.17.0-dev.87+9b177a7d2.tar.xz"
        },
        {
          "os": "Linux",
          "arch": "aarch64",
          "filename": "zig-aarch64-linux-0.17.0-dev.87+9b177a7d2.tar.xz",
          "size": "49MiB",
          "shasum": "stu901...",
          "url": "https://ziglang.org/builds/zig-aarch64-linux-0.17.0-dev.87+9b177a7d2.tar.xz"
        }
      ]
    }
  }
}
```

#### 简化版结构（核心字段）

```json
{
  "master": {
    "date": "2026-04-22",
    "releases": [...],
    "platforms": {
      "windows": [{...}],
      "macos": [{...}],
      "linux": [{...}]
    }
  }
}
```

### 2.4 关键字段说明

#### 顶层字段

| 字段 | 类型 | 说明 |
|-----|------|------|
| `master` | Object | master/nightly 分支信息 |
| `date` | String | 构建日期 (YYYY-MM-DD) |
| `docs` | Object | 文档链接映射 |
| `releases` | Array | 通用发布文件列表 |
| `platforms` | Object | 按平台分类的二进制文件 |

#### 发布文件对象 (`releases[]`)

| 字段 | 类型 | 说明 |
|-----|------|------|
| `type` | String | 文件类型 ("Source", "Bootstrap") |
| `target` | Null/String | 目标平台（源码为 null） |
| `filename` | String | 文件名 |
| `size` | String | 文件大小（如 "21MiB"） |
| `shasum` | String | SHA256 校验和 |
| `signature` | Object | 数字签名信息 |
| `url` | String | 下载 URL |

#### 平台特定二进制 (`platforms.*[]`)

| 字段 | 类型 | 说明 |
|-----|------|------|
| `os` | String | 操作系统名称 |
| `arch` | String | CPU 架构 |
| `filename` | String | 文件名（包含版本号） |
| `size` | String | 文件大小 |
| `shasum` | String | SHA256 校验和 |
| `signature` | Object | minisign 签名信息 |
| `url` | String | 直接下载链接 |

### 2.5 使用示例

#### Rust 代码示例：获取 Zig 版本列表

```rust
use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct ZigDownloadIndex {
    #[serde(rename = "master")]
    master: ZigVersionInfo,
}

#[derive(Debug, Serialize, Deserialize)]
struct ZigVersionInfo {
    date: String,
    platforms: PlatformBinaries,
}

#[derive(Debug, Serialize, Deserialize)]
struct PlatformBinaries {
    #[serde(rename = "windows")]
    windows: Vec<BinaryAsset>,
    #[serde(rename = "macos")]
    macos: Vec<BinaryAsset>,
    #[serde(rename = "linux")]
    linux: Vec<BinaryAsset>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct BinaryAsset {
    os: String,
    arch: String,
    filename: String,
    size: String,
    shasum: String,
    url: String,
}

async fn fetch_zig_versions() -> Result<Vec<BinaryAsset>, reqwest::Error> {
    let client = Client::new();
    let url = "https://ziglang.org/download/index.json";

    let response: ZigDownloadIndex = client
        .get(url)
        .header("User-Agent", "zzm/0.1.0")
        .send()
        .await?
        .json()
        .await;

    match response {
        Ok(index) => {
            let mut all_binaries = Vec::new();
            all_binaries.extend(index.master.platforms.windows);
            all_binaries.extend(index.master.platforms.macos);
            all_binaries.extend(index.master.platforms.linux);
            Ok(all_binaries)
        }
        Err(e) => Err(e),
    }
}
```

#### 获取当前平台对应的下载链接

```rust
fn get_download_url_for_current_platform(
    binaries: &[BinaryAsset],
    target_os: &str,
    target_arch: &str,
) -> Option<String> {
    binaries
        .iter()
        .find(|b| b.os == target_os && b.arch == target_arch)
        .map(|b| b.url.clone())
}
```

---

## 3. ZLS GitHub Releases API

### 3.1 基本信息

| 属性 | 值 |
|-----|---|
| **GitHub 仓库** | https://github.com/zigtools/zls |
| **Releases API** | https://api.github.com/repos/zigtools/zls/releases |
| **维护组织** | zigtools |
| **认证方式** | 公开 API 可匿名访问；认证用户有更高速率限制 |
| **速率限制** | 未认证: 60 次/小时；认证: 5000 次/小时 |

### 3.2 ZLS 项目简介

**ZLS** (Zig Language Server) 是 Zig 编程语言的 Language Server Protocol 实现，为开发者提供 IDE 功能支持。

**核心功能**：
- ✅ 代码补全 (Completions)
- ✅ 悬停信息 (Hover)
- ✅ 跳转到定义 (Goto definition/declaration)
- ✅ 文档符号 (Document symbols)
- ✅ 查找引用 (Find references)
- ✅ 符号重命名 (Rename symbol)
- ✅ 代码格式化 (使用 `zig fmt`)
- ✅ 语义高亮 (Semantic token highlighting)
- ✅ 内联提示 (Inlay hints)
- ✅ 代码操作 (Code actions)

**重要提示**：
> ZLS 的默认分支 targeting Zig master。当升级 Zig 时，务必同步更新 ZLS 以保持兼容性。

### 3.3 Releases API 结构详解

**端点**: `GET https://api.github.com/repos/zigtools/zls/releases`

#### 完整 JSON 结构示例

```json
[
  {
    "url": "https://api.github.com/repos/zigtools/zls/releases/310059698",
    "assets_url": "https://api.github.com/repos/zigtools/zls/releases/310059698/assets",
    "upload_url": "https://uploads.github.com/repos/zigtools/zls/releases/310059698/assets{?name,label}",
    "html_url": "https://github.com/zigtools/zls/releases/tag/0.16.0",
    "id": 310059698,
    "author": {
      "login": "Techatrix",
      "id": 19954306,
      "node_id": "MDQ6VXNlcjE5OTU0MzA2",
      "avatar_url": "https://avatars.githubusercontent.com/u/19954306?v=4",
      "gravatar_id": "",
      "url": "https://api.github.com/users/Techatrix",
      "html_url": "https://github.com/Techatrix",
      "type": "User",
      "site_admin": false
    },
    "node_id": "RE_kwDOD2p73s4SeyKy",
    "tag_name": "0.16.0",
    "name": "zls 0.16.0",
    "draft": false,
    "prerelease": false,
    "created_at": "2026-04-16T19:44:48Z",
    "published_at": "2026-04-16T20:47:47Z",
    "assets": [
      {
        "url": "https://api.github.com/repos/zigtools/zls/releases/assets/398050971",
        "id": 398050971,
        "name": "zls-aarch64-linux.tar.xz",
        "label": null,
        "content_type": "application/x-xz",
        "state": "uploaded",
        "size": 3814984,
        "download_count": 149,
        "created_at": "2026-04-16T20:44:37Z",
        "updated_at": "2026-04-16T20:46:43Z",
        "browser_download_url": "https://github.com/zigtools/zls/releases/download/0.16.0/zls-aarch64-linux.tar.xz"
      },
      {
        "url": "https://api.github.com/repos/zigtools/zls/releases/assets/398051006",
        "id": 398051006,
        "name": "zls-aarch64-linux.tar.xz.minisig",
        "label": null,
        "content_type": "application/x-xz",
        "state": "uploaded",
        "size": 128,
        "download_count": 120,
        "created_at": "2026-04-16T20:44:37Z",
        "updated_at": "2026-04-16T20:46:43Z",
        "browser_download_url": "https://github.com/zigtools/zls/releases/download/0.16.0/zls-aarch64-linux.tar.xz.minisig"
      },
      {
        "url": "https://api.github.com/repos/zigtools/zls/releases/assets/398051020",
        "id": 398051020,
        "name": "zls-x86_64-windows.tar.xz",
        "label": null,
        "content_type": "application/x-xz",
        "state": "uploaded",
        "size": 4200000,
        "download_count": 892,
        "created_at": "2026-04-16T20:44:37Z",
        "updated_at": "2026-04-16T20:46:43Z",
        "browser_download_url": "https://github.com/zigtools/zls/releases/download/0.16.0/zls-x86_64-windows.tar.xz"
      },
      {
        "url": "https://api.github.com/repos/zigtools/zls/releases/assets/398051035",
        "id": 398051035,
        "name": "zls-x86_64-macos.tar.xz",
        "label": null,
        "content_type": "application/x-xz",
        "state": "uploaded",
        "size": 3900000,
        "download_count": 567,
        "created_at": "2026-04-16T20:44:37Z",
        "updated_at": "2026-04-16T20:46:43Z",
        "browser_download_url": "https://github.com/zigtools/zls/releases/download/0.16.0/zls-x86_64-macos.tar.xz"
      },
      {
        "url": "https://api.github.com/repos/zigtools/zls/releases/assets/398051050",
        "id": 398051050,
        "name": "zls-x86_64-linux.tar.xz",
        "label": null,
        "content_type": "application/x-xz",
        "state": "uploaded",
        "size": 4050000,
        "download_count": 1234,
        "created_at": "2026-04-16T20:44:37Z",
        "updated_at": "2026-04-16T20:46:43Z",
        "browser_download_url": "https://github.com/zigtools/zls/releases/download/0.16.0/zls-x86_64-linux.tar.xz"
      }
    ],
    "tarball_url": "https://api.github.com/repos/zigtools/zls/tarball/0.16.0",
    "zipball_url": "https://api.github.com/repos/zigtools/zls/zipball/0.16.0",
    "body": "## Release Notes\n\n### 新功能\n- ...\n\n### Bug 修复\n- ..."
  }
]
```

#### 简化版结构（核心字段）

```json
[
  {
    "tag_name": "0.16.0",
    "name": "zls 0.16.0",
    "prerelease": false,
    "published_at": "2026-04-16T20:47:47Z",
    "assets": [
      {
        "name": "zls-x86_64-windows.tar.xz",
        "browser_download_url": "https://github.com/.../download/0.16.0/zls-x86_64-windows.tar.xz",
        "size": 4200000,
        "download_count": 892
      }
    ]
  }
]
```

### 3.4 关键字段说明

#### Release 对象（数组元素）

| 字段 | 类型 | 说明 |
|-----|------|------|
| `tag_name` | String | Git 标签名（即版本号） |
| `name` | String | Release 标题 |
| `draft` | Boolean | 是否为草稿 |
| `prerelease` | Boolean | 是否为预发布版本 |
| `created_at` | String | 创建时间 (ISO 8601) |
| `published_at` | String | 发布时间 (ISO 8601) |
| `assets` | Array | 附件列表（二进制文件等） |
| `body` | String | Release 说明（Markdown 格式） |
| `html_url` | String | Release 页面 URL |

#### Asset 对象（`assets[]`）

| 字段 | 类型 | 说明 |
|-----|------|------|
| `id` | Number | Asset 唯一 ID |
| `name` | String | 文件名 |
| `label` | String/Null | 自定义标签 |
| `content_type` | String | MIME 类型 |
| `state` | String | 上传状态 ("uploaded") |
| `size` | Number | 文件大小（字节） |
| `download_count` | Number | 下载次数 |
| `created_at` | String | 创建时间 |
| `updated_at` | String | 更新时间 |
| `browser_download_url` | String | **直接下载链接** ⭐ |

### 3.5 ZLS 文件命名规范

ZLS 预编译二进制文件的命名遵循以下模式：

```
zls-{arch}-{os}.{extension}
```

**示例**：
- `zls-x86_64-windows.tar.xz` - Windows x86_64
- `zls-aarch64-linux.tar.xz` - Linux ARM64
- `zls-x86_64-macos.tar.xz` - macOS Intel
- `*.minisig` - 对应的数字签名文件

**支持的组合**：

| OS | Architecture | 文件名模式 |
|----|-------------|-----------|
| Windows | x86_64 | `zls-x86_64-windows.tar.xz` |
| Linux | x86_64 | `zls-x86_64-linux.tar.xz` |
| Linux | aarch64 | `zls-aarch64-linux.tar.xz` |
| macOS | x86_64 | `zls-x86_64-macos.tar.xz` |
| macOS | aarch64 | `zls-aarch64-macos.tar.xz` |

### 3.6 使用示例

#### Rust 代码示例：获取 ZLS 版本列表

```rust
use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct GithubRelease {
    tag_name: String,
    name: String,
    prerelease: bool,
    published_at: String,
    assets: Vec<GithubAsset>,
    html_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct GithubAsset {
    id: u64,
    name: String,
    size: u64,
    download_count: u64,
    browser_download_url: String,
    content_type: String,
}

async fn fetch_zls_releases() -> Result<Vec<GithubRelease>, reqwest::Error> {
    let client = Client::builder()
        .user_agent("zzm/0.1.0")
        .build()?;

    let url = "https://api.github.com/repos/zigtools/zls/releases";

    let releases: Vec<GithubRelease> = client
        .get(url)
        .send()
        .await?
        .json()
        .await;

    releases
}
```

#### 获取特定平台的 ZLS 下载链接

```rust
fn find_zls_asset_for_platform(
    release: &GithubRelease,
    os: &str,
    arch: &str,
) -> Option<&GithubAsset> {
    release.assets.iter().find(|asset| {
        let name_lower = asset.name.to_lowercase();
        name_lower.contains(os) && name_lower.contains(arch) && !name_lower.ends_with(".minisig")
    })
}
```

#### 过滤稳定版本（排除 prerelease 和 draft）

```rust
fn get_stable_releases(releases: &[GithubRelease]) -> Vec<&GithubRelease> {
    releases
        .iter()
        .filter(|r| !r.prerelease)
        .collect()
}
```

---

## 4. API 集成最佳实践

### 4.1 缓存策略

由于外部 API 有速率限制，建议实现本地缓存：

```rust
use std::time::{Duration, SystemTime};
use tokio::fs;

struct ApiCache<T> {
    data: T,
    fetched_at: SystemTime,
    ttl: Duration,
}

impl<T: serde::de::DeserializeOwned + serde::Serialize> ApiCache<T> {
    const CACHE_TTL: Duration = Duration::from_secs(3600); // 1 小时

    async fn load_or_fetch(
        cache_path: &Path,
        fetch_fn: impl Future<Output = Result<T>>,
    ) -> Result<T> {
        if let Ok(cached) = Self::read_cache(cache_path).await {
            if !cached.is_expired() {
                return Ok(cached.data);
            }
        }

        let data = fetch_fn.await?;
        Self::write_cache(cache_path, &data).await?;
        Ok(data)
    }

    fn is_expired(&self) -> bool {
        self.fetched_at.elapsed().unwrap_or(Duration::ZERO) > self.ttl
    }
}
```

### 4.2 错误处理与重试

```rust
async fn fetch_with_retry<F, T, Fut>(
    fetch_fn: F,
    max_retries: u32,
) -> Result<T>
where
    F: Fn() -> Fut,
    Fut: Future<Output = Result<T>>,
{
    let mut last_error = None;

    for attempt in 1..=max_retries {
        match fetch_fn().await {
            Ok(result) => return Ok(result),
            Err(e) => {
                last_error = Some(e);
                if attempt < max_retries {
                    let delay = Duration::from_millis(100 * 2u64.pow(attempt));
                    tokio::time::sleep(delay).await;
                }
            }
        }
    }

    Err(last_error.unwrap())
}
```

### 4.3 User-Agent 设置

**重要**：GitHub API 要求设置合理的 User-Agent：

```rust
let client = reqwest::Client::builder()
    .user_agent("zzm/0.1.0 (https://github.com/user/zzm)")
    .build()?;
```

### 4.4 并发请求优化

可以并行获取 Zig 和 ZLS 的版本信息：

```rust
async fn fetch_all_version_info() -> Result<(Vec<BinaryAsset>, Vec<GithubRelease>)> {
    let (zig_result, zls_result) = tokio::join!(
        fetch_zig_versions(),
        fetch_zls_releases()
    );

    Ok((zig_result?, zls_result?))
}
```

---

## 5. 数据模型映射

### 5.1 统一版本信息结构

为了在 zzm 内部统一管理 Zig 和 ZLS 的版本信息，定义如下数据结构：

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ToolType {
    Zig,
    Zls,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedVersionInfo {
    pub tool: ToolType,
    pub version: String,
    pub channel: VersionChannel,
    pub release_date: Option<String>,
    pub assets: Vec<PlatformAsset>,
    pub source_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VersionChannel {
    Stable,
    Nightly,
    Beta,
    Rc,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformAsset {
    pub os: String,
    pub arch: String,
    pub filename: String,
    pub download_url: String,
    pub size: u64,
    pub checksum_sha256: Option<String>,
    pub signature_url: Option<String>,
}
```

### 5.2 数据转换函数

```rust
impl From<BinaryAsset> for PlatformAsset {
    fn from(asset: BinaryAsset) -> Self {
        PlatformAsset {
            os: asset.os,
            arch: asset.arch,
            filename: asset.filename,
            download_url: asset.url,
            size: parse_size_to_bytes(&asset.size), // "53MiB" -> 55574528
            checksum_sha256: Some(asset.shasum),
            signature_url: None, // 需要从 signature 对象中提取
        }
    }
}

impl From<GithubAsset> for PlatformAsset {
    fn from(asset: GithubAsset) -> Self {
        PlatformAsset {
            os: extract_os_from_filename(&asset.name),
            arch: extract_arch_from_filename(&asset.name),
            filename: asset.name,
            download_url: asset.browser_download_url,
            size: asset.size,
            checksum_sha256: None, // GitHub 不直接提供 SHA256
            signature_url: find_signature_url(&asset.name), // 查找对应的 .minisig 文件
        }
    }
}
```

---

## 6. 测试与验证

### 6.1 单元测试示例

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_fetch_zig_versions_success() {
        let versions = fetch_zig_versions().await.unwrap();
        assert!(!versions.is_empty());

        let windows_x64 = versions.iter()
            .find(|v| v.os == "Windows" && v.arch == "x86_64");
        assert!(windows_x64.is_some());
    }

    #[tokio::test]
    async fn test_fetch_zls_releases_success() {
        let releases = fetch_zls_releases().await.unwrap();
        assert!(!releases.is_empty());

        let latest_stable = releases.iter()
            .find(|r| !r.prerelease);
        assert!(latest_stable.is_some());
    }

    #[test]
    fn test_parse_zig_filename() {
        let filename = "zig-x86_64-windows-0.13.0.zip";
        assert!(filename.contains("windows"));
        assert!(filename.contains("x86_64"));
    }

    #[test]
    fn test_parse_zls_filename() {
        let filename = "zls-x86_64-linux.tar.xz";
        assert!(filename.starts_with("zls-"));
        assert!(filename.contains("linux"));
    }
}
```

### 6.2 Mock 数据示例

用于开发和测试的模拟数据：

```rust
pub fn mock_zig_response() -> ZigDownloadIndex {
    // 返回预设的测试数据
    todo!()
}

pub fn mock_zls_releases() -> Vec<GithubRelease> {
    vec![
        GithubRelease {
            tag_name: "0.13.0".to_string(),
            name: "zls 0.13.0".to_string(),
            prerelease: false,
            published_at: "2024-01-15T10:30:00Z".to_string(),
            assets: vec![
                GithubAsset {
                    id: 1,
                    name: "zls-x86_64-windows.tar.xz".to_string(),
                    size: 4200000,
                    download_count: 1500,
                    browser_download_url: "https://example.com/zls-0.13.0-x86_64-windows.tar.xz".to_string(),
                    content_type: "application/x-xz".to_string(),
                },
            ],
            html_url: "https://github.com/zigtools/zls/releases/tag/0.13.0".to_string(),
        },
    ]
}
```

---

## 7. 注意事项与限制

### 7.1 Zig API 注意事项

1. **版本标识符**：
   - `master`: 最新开发版本（nightly 构建）
   - 未来可能增加 stable 版本入口

2. **文件大小格式**：
   - 使用人类可读格式（如 "53MiB"）
   - 需要转换为字节进行计算

3. **签名验证**：
   - 提供 minisign 签名文件
   - 建议在生产环境中验证签名

4. **URL 稳定性**：
   - 下载 URL 可能随版本变化
   - 始终通过 API 获取最新 URL

### 7.2 GitHub API 注意事项

1. **速率限制**：
   - 未认证：60 次/小时
   - 认证（OAuth/GitHub Token）：5000 次/小时
   - 响应头包含 `X-RateLimit-Remaining` 信息

2. **分页**：
   - 默认返回最近的 30 个 releases
   - 可通过 `?per_page=100` 参数调整
   - 大量 releases 需要分页处理

3. **认证方式**（可选但推荐）：

```bash
# 通过环境变量传递 GitHub Token
export GITHUB_TOKEN="your_personal_access_token"

# 在代码中使用
let client = reqwest::Client::builder()
    .user_agent("zzm/0.1.0")
    .default_header(
        header::AUTHORIZATION,
        format!("Bearer {}", std::env::var("GITHUB_TOKEN").unwrap_or_default())
    )
    .build()?;
```

4. **错误响应码**：
   - `403`: 速率超限或 IP 被封禁
   - `404`: 仓库不存在或私有
   - `422`: 验证失败

---

## 8. 参考资源

### 官方文档

- [Zig 官方下载页面](https://ziglang.org/download/)
- [Zig JSON API](https://ziglang.org/download/index.json)
- [ZLS GitHub 仓库](https://github.com/zigtools/zls)
- [GitHub REST API 文档 - Releases](https://docs.github.com/en/rest/releases)
- [minisign 签名工具](https://jedisct1.github.io/minisign/)

### 相关工具

- [jq](https://stedolan.github.io/jq/) - JSON 命令行处理器
- [httpie](https://httpie.io/) - 现代 HTTP 客户端
- [curl](https://curl.se/) - 经典命令行 HTTP 工具

### 快速测试命令

```bash
# 测试 Zig JSON API
curl -s https://ziglang.org/download/index.json | jq '.master.date'

# 测试 ZLS Releases API
curl -s https://api.github.com/repos/zigtools/zls/releases | jq '.[0].tag_name'

# 获取最新的 ZLS Windows x64 下载链接
curl -s https://api.github.com/repos/zigtools/zls/releases \
  | jq '.[0].assets[] | select(.name | contains("x86_64-windows")) | .browser_download_url'
```

---

## 9. 更新日志

| 版本 | 日期 | 修改内容 |
|-----|------|---------|
| v1.0.0 | 2026-04-24 | 初始版本，记录 Zig 和 ZLS API 结构和使用方法 |
| v1.1.0 | 2026-04-25 | 状态更新：与源码实现对照验证，确认 API 集成准确 |

---

*本文档将随着 API 变化和项目开发持续更新。*
