use std::fs;
use std::path::{Path, PathBuf};

use crate::utils::error::ZzmError;

/// 文件系统操作模块
///
/// 提供 tar.gz / zip 解压、文件权限设置等操作
///
/// 解压文件到指定目录
///
/// 自动检测压缩格式（.tar.xz, .tar.gz, .zip）并选择合适的解压方式
pub fn extract_archive(archive_path: &Path, dest_dir: &Path) -> Result<PathBuf, ZzmError> {
    if !archive_path.exists() {
        return Err(ZzmError::ExtractionFailed {
            path: archive_path.to_string_lossy().to_string(),
            reason: "文件不存在".to_string(),
        });
    }

    // 确保目标目录存在
    fs::create_dir_all(dest_dir).map_err(ZzmError::Io)?;

    let filename = archive_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("");

    let result = if filename.ends_with(".tar.xz") {
        extract_tar_xz(archive_path, dest_dir)
    } else if filename.ends_with(".tar.gz") || filename.ends_with(".tgz") {
        extract_tar_gz(archive_path, dest_dir)
    } else if filename.ends_with(".zip") {
        extract_zip(archive_path, dest_dir)
    } else {
        return Err(ZzmError::ExtractionFailed {
            path: filename.to_string(),
            reason: "不支持的压缩格式（支持 .tar.xz, .tar.gz, .zip）".to_string(),
        });
    };

    result?;

    // 查找解压后的根目录（通常压缩包内有一个顶层目录）
    let extracted_root = find_extracted_root(dest_dir, filename);
    Ok(extracted_root)
}

/// 解压 .tar.xz 文件
fn extract_tar_xz(archive_path: &Path, dest_dir: &Path) -> Result<(), ZzmError> {
    tracing::debug!(
        "解压 tar.xz: {} -> {}",
        archive_path.display(),
        dest_dir.display()
    );

    let file = fs::File::open(archive_path).map_err(ZzmError::Io)?;
    let decoder = xz2::read::XzDecoder::new(file);
    let mut archive = tar::Archive::new(decoder);

    // 安全解压：防止路径遍历攻击
    archive.set_preserve_permissions(true);
    unpack_tar_archive(&mut archive, dest_dir)
}

/// 解压 .tar.gz 文件
fn extract_tar_gz(archive_path: &Path, dest_dir: &Path) -> Result<(), ZzmError> {
    tracing::debug!(
        "解压 tar.gz: {} -> {}",
        archive_path.display(),
        dest_dir.display()
    );

    let file = fs::File::open(archive_path).map_err(ZzmError::Io)?;
    let decoder = flate2::read::GzDecoder::new(file);
    let mut archive = tar::Archive::new(decoder);

    archive.set_preserve_permissions(true);
    unpack_tar_archive(&mut archive, dest_dir)
}

/// 安全解压 tar 归档
fn unpack_tar_archive<R: std::io::Read>(
    archive: &mut tar::Archive<R>,
    dest_dir: &Path,
) -> Result<(), ZzmError> {
    for entry_result in archive.entries().map_err(|e| ZzmError::ExtractionFailed {
        path: dest_dir.to_string_lossy().to_string(),
        reason: format!("无法读取归档条目: {e}"),
    })? {
        let mut entry = entry_result.map_err(|e| ZzmError::ExtractionFailed {
            path: dest_dir.to_string_lossy().to_string(),
            reason: format!("归档条目读取错误: {e}"),
        })?;

        let entry_path_owned =
            entry
                .path()
                .map(|p| p.to_path_buf())
                .map_err(|e| ZzmError::ExtractionFailed {
                    path: dest_dir.to_string_lossy().to_string(),
                    reason: format!("无效的条目路径: {e}"),
                })?;

        // 安全检查：防止路径遍历攻击
        if entry_path_owned
            .to_str()
            .is_some_and(|s| s.starts_with("..") || s.starts_with('/') || s.contains(".."))
        {
            tracing::warn!("跳过可疑路径: {}", entry_path_owned.display());
            continue;
        }

        let path_for_error = entry_path_owned.to_string_lossy().to_string();
        entry
            .unpack_in(dest_dir)
            .map_err(|e| ZzmError::ExtractionFailed {
                path: path_for_error,
                reason: format!("解压失败: {e}"),
            })?;
    }

    Ok(())
}

/// 解压 .zip 文件
fn extract_zip(archive_path: &Path, dest_dir: &Path) -> Result<(), ZzmError> {
    tracing::debug!(
        "解压 zip: {} -> {}",
        archive_path.display(),
        dest_dir.display()
    );

    let file = fs::File::open(archive_path).map_err(ZzmError::Io)?;
    let mut archive = zip::ZipArchive::new(file).map_err(|e| ZzmError::ExtractionFailed {
        path: archive_path.to_string_lossy().to_string(),
        reason: format!("无法打开 zip 文件: {e}"),
    })?;

    for i in 0..archive.len() {
        let mut file = archive
            .by_index(i)
            .map_err(|e| ZzmError::ExtractionFailed {
                path: archive_path.to_string_lossy().to_string(),
                reason: format!("读取 zip 条目失败: {e}"),
            })?;

        let outpath = match file.enclosed_name() {
            Some(path) => dest_dir.join(path),
            None => continue,
        };

        // 安全检查：防止路径遍历
        if outpath.to_str().is_some_and(|s| s.contains("..")) {
            tracing::warn!("跳过可疑路径: {}", outpath.display());
            continue;
        }

        if file.is_dir() {
            fs::create_dir_all(&outpath).map_err(ZzmError::Io)?;
        } else {
            if let Some(p) = outpath.parent()
                && !p.exists()
            {
                fs::create_dir_all(p).map_err(ZzmError::Io)?;
            }
            let mut outfile = fs::File::create(&outpath).map_err(ZzmError::Io)?;
            std::io::copy(&mut file, &mut outfile).map_err(ZzmError::Io)?;
        }

        // 设置 Unix 可执行权限
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if let Some(mode) = file.unix_mode() {
                // 如果文件有执行权限位，设置之
                if mode & 0o111 != 0 {
                    fs::set_permissions(&outpath, fs::Permissions::from_mode(mode))
                        .map_err(ZzmError::Io)?;
                }
            }
        }
    }

    Ok(())
}

/// 查找解压后的根目录
///
/// 大多数 Zig/ZLS 压缩包内有一个顶层目录，如 zig-x86_64-windows-0.13.0/
/// 本函数查找并返回该目录路径
fn find_extracted_root(dest_dir: &Path, archive_filename: &str) -> PathBuf {
    // 尝试根据文件名推断顶层目录名
    let expected_prefix = archive_filename
        .trim_end_matches(".tar.xz")
        .trim_end_matches(".tar.gz")
        .trim_end_matches(".tgz")
        .trim_end_matches(".zip");

    let expected_dir = dest_dir.join(expected_prefix);
    if expected_dir.is_dir() {
        return expected_dir;
    }

    // 如果没有找到预期目录，返回 dest_dir 本身
    // （或者查找 dest_dir 中唯一的子目录）
    if let Ok(entries) = fs::read_dir(dest_dir) {
        let dirs: Vec<_> = entries.flatten().filter(|e| e.path().is_dir()).collect();

        if dirs.len() == 1 {
            return dirs[0].path();
        }
    }

    dest_dir.to_path_buf()
}

/// 设置文件为可执行（Unix）
#[cfg(unix)]
pub fn set_executable(path: &Path) -> Result<(), ZzmError> {
    use std::os::unix::fs::PermissionsExt;
    let mut perms = fs::metadata(path).map_err(ZzmError::Io)?.permissions();
    perms.set_mode(perms.mode() | 0o755);
    fs::set_permissions(path, perms).map_err(ZzmError::Io)
}

/// 设置文件为可执行（Windows - 无操作）
#[cfg(not(unix))]
pub fn set_executable(_path: &Path) -> Result<(), ZzmError> {
    // Windows 上不需要设置可执行权限
    Ok(())
}

/// 递归删除目录
pub fn remove_dir_all(path: &Path) -> Result<(), ZzmError> {
    if path.exists() {
        fs::remove_dir_all(path).map_err(ZzmError::Io)?;
    }
    Ok(())
}

/// 重新组织解压后的文件
///
/// 压缩包通常包含一个顶层目录（如 `zig-x86_64-windows-0.13.0/`），
/// 需要将子目录的内容移动到版本目录的根部。
///
/// - 如果 `extracted_root` 和 `version_dir` 相同，无需操作
/// - 如果 `extracted_root` 是 `version_dir` 的子目录，将内容上移
pub fn reorganize_extracted_files(
    extracted_root: &Path,
    version_dir: &Path,
    temp_suffix: &str,
) -> Result<(), ZzmError> {
    // 如果 extracted_root 和 version_dir 相同，说明文件已经在正确位置
    if extracted_root == version_dir {
        return Ok(());
    }

    // extracted_root 是 version_dir 的子目录（如 version_dir/zig-platform-version/）
    // 需要将子目录的内容移动到 version_dir 中
    if extracted_root.starts_with(version_dir) && extracted_root.is_dir() {
        // 在临时目录中操作
        let temp_dir = version_dir.with_extension(temp_suffix);
        if temp_dir.exists() {
            fs::remove_dir_all(&temp_dir).map_err(ZzmError::Io)?;
        }

        // 将 extracted_root 重命名为 temp_dir
        fs::rename(extracted_root, &temp_dir).map_err(ZzmError::Io)?;

        // 将 temp_dir 中的内容移动到 version_dir
        for entry in fs::read_dir(&temp_dir).map_err(ZzmError::Io)? {
            let entry = entry.map_err(ZzmError::Io)?;
            let dest = version_dir.join(entry.file_name());
            fs::rename(entry.path(), dest).map_err(ZzmError::Io)?;
        }

        // 清理临时目录
        let _ = fs::remove_dir(&temp_dir);
    }

    Ok(())
}

// ========== 单元测试 ==========

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_extract_nonexistent_file() {
        let result = extract_archive(
            Path::new("/nonexistent/file.tar.gz"),
            Path::new("/tmp/test"),
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_extract_unsupported_format() {
        let temp_dir = tempfile::tempdir().unwrap();
        let file_path = temp_dir.path().join("test.rar");
        fs::write(&file_path, b"test").unwrap();

        let result = extract_archive(&file_path, temp_dir.path());
        assert!(result.is_err());
        if let Err(ZzmError::ExtractionFailed { reason, .. }) = result {
            assert!(reason.contains("不支持的压缩格式"));
        }
    }

    #[test]
    fn test_extract_zip_basic() {
        let temp_dir = tempfile::tempdir().unwrap();
        let zip_path = temp_dir.path().join("test.zip");

        // 创建一个简单的 zip 文件
        let zip_file = fs::File::create(&zip_path).unwrap();
        let mut zip_writer = zip::ZipWriter::new(zip_file);
        let options = zip::write::SimpleFileOptions::default();

        zip_writer.start_file("hello.txt", options).unwrap();
        zip_writer.write_all(b"Hello, World!").unwrap();
        zip_writer.finish().unwrap();

        let dest = temp_dir.path().join("extracted");
        fs::create_dir_all(&dest).unwrap();

        let result = extract_archive(&zip_path, &dest);
        assert!(result.is_ok());

        let extracted_file = dest.join("hello.txt");
        assert!(extracted_file.exists());
        let content = fs::read_to_string(&extracted_file).unwrap();
        assert_eq!(content, "Hello, World!");
    }

    #[test]
    fn test_remove_dir_all_nonexistent() {
        let result = remove_dir_all(Path::new("/nonexistent/dir"));
        assert!(result.is_ok());
    }

    #[test]
    fn test_extract_zip_with_subdirectory() {
        let temp_dir = tempfile::tempdir().unwrap();
        let zip_path = temp_dir.path().join("test_subdir.zip");

        let zip_file = fs::File::create(&zip_path).unwrap();
        let mut zip_writer = zip::ZipWriter::new(zip_file);
        let options = zip::write::SimpleFileOptions::default();

        zip_writer.start_file("subdir/hello.txt", options).unwrap();
        zip_writer.write_all(b"Hello from subdir!").unwrap();
        zip_writer
            .start_file("subdir/nested/deep.txt", options)
            .unwrap();
        zip_writer.write_all(b"Deep nested").unwrap();
        zip_writer.finish().unwrap();

        let dest = temp_dir.path().join("extracted");
        fs::create_dir_all(&dest).unwrap();

        let result = extract_archive(&zip_path, &dest);
        assert!(result.is_ok());

        assert!(dest.join("subdir/hello.txt").exists());
        assert!(dest.join("subdir/nested/deep.txt").exists());
    }

    #[test]
    fn test_extract_zip_preserves_content() {
        let temp_dir = tempfile::tempdir().unwrap();
        let zip_path = temp_dir.path().join("content_test.zip");

        let content = b"Binary data: \x00\x01\x02\xff";

        let zip_file = fs::File::create(&zip_path).unwrap();
        let mut zip_writer = zip::ZipWriter::new(zip_file);
        let options = zip::write::SimpleFileOptions::default();

        zip_writer.start_file("binary.bin", options).unwrap();
        zip_writer.write_all(content).unwrap();
        zip_writer.finish().unwrap();

        let dest = temp_dir.path().join("extracted");
        fs::create_dir_all(&dest).unwrap();

        let result = extract_archive(&zip_path, &dest);
        assert!(result.is_ok());

        let extracted = fs::read(dest.join("binary.bin")).unwrap();
        assert_eq!(extracted, content);
    }

    #[test]
    fn test_remove_dir_all_with_content() {
        let temp_dir = tempfile::tempdir().unwrap();
        let sub_dir = temp_dir.path().join("to_remove");
        fs::create_dir_all(&sub_dir).unwrap();
        fs::write(sub_dir.join("file.txt"), b"content").unwrap();

        assert!(sub_dir.exists());
        let result = remove_dir_all(&sub_dir);
        assert!(result.is_ok());
        assert!(!sub_dir.exists());
    }

    #[test]
    fn test_find_extracted_root_with_subdirectory() {
        let temp_dir = tempfile::tempdir().unwrap();
        let dest = temp_dir.path().join("dest");
        fs::create_dir_all(&dest).unwrap();

        // 创建子目录
        let sub = dest.join("zig-x86_64-windows-0.13.0");
        fs::create_dir_all(&sub).unwrap();

        let root = find_extracted_root(&dest, "zig-x86_64-windows-0.13.0.zip");
        assert_eq!(root, sub);
    }

    #[test]
    fn test_find_extracted_root_no_subdirectory() {
        let temp_dir = tempfile::tempdir().unwrap();
        let dest = temp_dir.path().join("dest");
        fs::create_dir_all(&dest).unwrap();

        // 直接放文件（没有子目录）
        fs::write(dest.join("zig.exe"), "binary").unwrap();

        let root = find_extracted_root(&dest, "zig-x86_64-windows-0.13.0.zip");
        assert_eq!(root, dest);
    }

    #[test]
    fn test_set_executable_no_error() {
        // 在任何平台上都不应返回错误
        let temp_dir = tempfile::tempdir().unwrap();
        let file_path = temp_dir.path().join("test_binary");
        fs::write(&file_path, b"binary").unwrap();

        let result = set_executable(&file_path);
        assert!(result.is_ok());
    }
}
