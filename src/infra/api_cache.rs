use std::marker::PhantomData;
use std::path::PathBuf;
use std::time::Duration;

use serde::Serialize;
use serde::de::DeserializeOwned;

use crate::utils::error::ZzmError;

/// 泛型 API 缓存层
///
/// 统一了之前 `ZigApiClient` 和 `ZlsApiClient` 中各自实现的
/// `load_from_cache` / `save_to_cache` 逻辑。
pub struct ApiCache<T: Serialize + DeserializeOwned> {
    cache_dir: PathBuf,
    filename: String,
    ttl: Duration,
    _marker: PhantomData<T>,
}

impl<T: Serialize + DeserializeOwned> ApiCache<T> {
    /// 创建新的缓存实例
    pub fn new(cache_dir: PathBuf, filename: &str, ttl: Duration) -> Self {
        Self {
            cache_dir,
            filename: filename.to_string(),
            ttl,
            _marker: PhantomData,
        }
    }

    /// 获取缓存文件路径
    fn cache_path(&self) -> PathBuf {
        self.cache_dir.join(&self.filename)
    }

    /// 从缓存加载数据（如果未过期）
    pub fn load(&self) -> Option<T> {
        let path = self.cache_path();
        if !path.exists() {
            return None;
        }

        // 检查缓存文件修改时间
        let metadata = std::fs::metadata(&path).ok()?;
        let modified = metadata.modified().ok()?;
        let elapsed = modified.elapsed().ok()?;

        if elapsed > self.ttl {
            tracing::debug!("缓存已过期: {}", self.filename);
            return None;
        }

        let content = std::fs::read_to_string(&path).ok()?;
        let data: T = serde_json::from_str(&content).ok()?;
        tracing::debug!("从缓存加载: {}", self.filename);
        Some(data)
    }

    /// 将数据写入缓存
    pub fn save(&self, data: &T) -> Result<(), ZzmError> {
        // 确保缓存目录存在
        if !self.cache_dir.exists() {
            std::fs::create_dir_all(&self.cache_dir).map_err(ZzmError::Io)?;
        }

        let path = self.cache_path();
        let content = serde_json::to_string_pretty(data)?;
        std::fs::write(&path, content)?;
        tracing::debug!("数据已缓存: {}", self.filename);
        Ok(())
    }
}

// ========== 单元测试 ==========

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    struct TestData {
        value: String,
        count: u32,
    }

    #[test]
    fn test_cache_save_and_load() {
        let temp_dir = tempfile::tempdir().unwrap();
        let cache: ApiCache<TestData> = ApiCache::new(
            temp_dir.path().to_path_buf(),
            "test.json",
            Duration::from_secs(3600),
        );

        let data = TestData {
            value: "hello".to_string(),
            count: 42,
        };

        cache.save(&data).unwrap();
        let loaded = cache.load().unwrap();
        assert_eq!(loaded, data);
    }

    #[test]
    fn test_cache_load_nonexistent() {
        let temp_dir = tempfile::tempdir().unwrap();
        let cache: ApiCache<TestData> = ApiCache::new(
            temp_dir.path().to_path_buf(),
            "nonexistent.json",
            Duration::from_secs(3600),
        );

        assert!(cache.load().is_none());
    }

    #[test]
    fn test_cache_creates_dir() {
        let temp_dir = tempfile::tempdir().unwrap();
        let nested = temp_dir.path().join("nested").join("dir");
        let cache: ApiCache<TestData> =
            ApiCache::new(nested, "test.json", Duration::from_secs(3600));

        let data = TestData {
            value: "test".to_string(),
            count: 1,
        };

        cache.save(&data).unwrap();
        assert!(cache.load().is_some());
    }
}
