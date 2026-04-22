//! 增量备份和差异备份实现

use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use tracing::debug;

/// 文件变更类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FileChangeType {
    /// 新增文件
    Added,
    /// 修改的文件
    Modified,
    /// 删除的文件
    Deleted,
    /// 未变更
    Unchanged,
}

/// 文件元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileMetadata {
    /// 文件路径
    pub path: PathBuf,
    /// 文件大小（字节）
    pub size: u64,
    /// 最后修改时间
    pub modified: chrono::DateTime<chrono::Utc>,
    /// SHA256 校验和
    pub checksum: String,
    /// 是否是目录
    pub is_dir: bool,
}

/// 文件变更记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileChange {
    /// 变更类型
    pub change_type: FileChangeType,
    /// 文件元数据（新增/修改时有值）
    pub metadata: Option<FileMetadata>,
    /// 旧元数据（删除/修改时有值）
    pub old_metadata: Option<FileMetadata>,
}

/// 增量备份清单
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IncrementalManifest {
    /// 基础备份ID（全量备份ID）
    pub base_backup_id: String,
    /// 变更文件列表
    pub changes: Vec<FileChange>,
    /// 备份时间
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// 差异备份策略
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DiffStrategy {
    /// 基于时间戳比较
    Timestamp,
    /// 基于文件大小比较
    Size,
    /// 基于校验和比较
    Checksum,
    /// 综合判断（时间+大小+校验和）
    Combined,
}

impl FileMetadata {
    /// 从文件路径读取元数据
    pub fn from_path(path: &Path) -> Result<Self> {
        let metadata = fs::metadata(path)
            .map_err(|e| Error::Internal(format!("读取文件元数据失败 {:?}: {}", path, e)))?;

        let checksum =
            if metadata.is_file() { calculate_file_checksum(path)? } else { String::new() };

        let modified = metadata
            .modified()
            .map_err(|e| Error::Internal(format!("读取文件修改时间失败 {:?}: {}", path, e)))?;

        Ok(Self {
            path: path.to_path_buf(),
            size: metadata.len(),
            modified: chrono::DateTime::<chrono::Utc>::from(modified),
            checksum,
            is_dir: metadata.is_dir(),
        })
    }

    /// 检查文件是否被修改
    pub fn is_modified(&self, other: &FileMetadata, strategy: DiffStrategy) -> bool {
        match strategy {
            DiffStrategy::Timestamp => self.modified != other.modified,
            DiffStrategy::Size => self.size != other.size,
            DiffStrategy::Checksum => self.checksum != other.checksum,
            DiffStrategy::Combined => {
                self.modified != other.modified
                    || self.size != other.size
                    || self.checksum != other.checksum
            },
        }
    }
}

/// 计算文件SHA256校验和
pub fn calculate_file_checksum(path: &Path) -> Result<String> {
    let mut file = fs::File::open(path)
        .map_err(|e| Error::Internal(format!("打开文件失败 {:?}: {}", path, e)))?;

    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 8192];

    loop {
        let n = std::io::Read::read(&mut file, &mut buffer)
            .map_err(|e| Error::Internal(format!("读取文件内容失败 {:?}: {}", path, e)))?;

        if n == 0 {
            break;
        }

        hasher.update(&buffer[..n]);
    }

    Ok(format!("{:x}", hasher.finalize()))
}

/// 扫描目录获取所有文件元数据
pub fn scan_directory(root: &Path) -> Result<Vec<FileMetadata>> {
    let mut files = Vec::new();

    fn scan_recursive(path: &Path, files: &mut Vec<FileMetadata>) -> Result<()> {
        let entries = fs::read_dir(path)
            .map_err(|e| Error::Internal(format!("读取目录失败 {:?}: {}", path, e)))?;

        for entry in entries {
            let entry =
                entry.map_err(|e| Error::Internal(format!("读取目录项失败 {:?}: {}", path, e)))?;
            let entry_path = entry.path();

            let metadata = FileMetadata::from_path(&entry_path)?;
            files.push(metadata);

            if entry_path.is_dir() {
                scan_recursive(&entry_path, files)?;
            }
        }

        Ok(())
    }

    scan_recursive(root, &mut files)?;
    Ok(files)
}

/// 计算增量变更
pub fn calculate_incremental_changes(
    previous_files: &[FileMetadata],
    current_files: &[FileMetadata],
    strategy: DiffStrategy,
) -> Vec<FileChange> {
    let mut changes = Vec::new();

    // 创建路径到元数据的映射
    let previous_map: HashMap<PathBuf, FileMetadata> =
        previous_files.iter().map(|m| (m.path.clone(), m.clone())).collect();

    let current_map: HashMap<PathBuf, FileMetadata> =
        current_files.iter().map(|m| (m.path.clone(), m.clone())).collect();

    // 检测新增和修改的文件
    for (path, current_meta) in &current_map {
        if let Some(previous_meta) = previous_map.get(path) {
            // 检测是否修改
            if current_meta.is_modified(previous_meta, strategy) {
                debug!("文件已修改: {:?}", path);
                changes.push(FileChange {
                    change_type: FileChangeType::Modified,
                    metadata: Some(current_meta.clone()),
                    old_metadata: Some(previous_meta.clone()),
                });
            }
        } else {
            // 新增文件
            debug!("新增文件: {:?}", path);
            changes.push(FileChange {
                change_type: FileChangeType::Added,
                metadata: Some(current_meta.clone()),
                old_metadata: None,
            });
        }
    }

    // 检测删除的文件
    for (path, previous_meta) in &previous_map {
        if !current_map.contains_key(path) {
            debug!("删除文件: {:?}", path);
            changes.push(FileChange {
                change_type: FileChangeType::Deleted,
                metadata: None,
                old_metadata: Some(previous_meta.clone()),
            });
        }
    }

    changes
}

/// 压缩文件
pub fn compress_file(source: &Path, destination: &Path) -> Result<u64> {
    fs::create_dir_all(destination.parent().unwrap_or(Path::new(".")))
        .map_err(|e| Error::Internal(format!("创建目录失败 {:?}: {}", destination, e)))?;

    let source_file = fs::File::open(source)
        .map_err(|e| Error::Internal(format!("打开源文件失败 {:?}: {}", source, e)))?;

    let dest_file = fs::File::create(destination)
        .map_err(|e| Error::Internal(format!("创建目标文件失败 {:?}: {}", destination, e)))?;

    let mut encoder = flate2::write::GzEncoder::new(dest_file, flate2::Compression::default());
    std::io::copy(&mut &source_file, &mut encoder)
        .map_err(|e| Error::Internal(format!("压缩文件失败 {:?}: {}", source, e)))?;

    let compressed_file = encoder
        .finish()
        .map_err(|e| Error::Internal(format!("压缩完成失败 {:?}: {}", source, e)))?;

    Ok(compressed_file
        .metadata()
        .map_err(|e| Error::Internal(format!("读取压缩文件元数据失败 {:?}: {}", destination, e)))?
        .len())
}

/// 解压文件
pub fn decompress_file(source: &Path, destination: &Path) -> Result<u64> {
    fs::create_dir_all(destination.parent().unwrap_or(Path::new(".")))
        .map_err(|e| Error::Internal(format!("创建目录失败 {:?}: {}", destination, e)))?;

    let source_file = fs::File::open(source)
        .map_err(|e| Error::Internal(format!("打开压缩文件失败 {:?}: {}", source, e)))?;

    let mut dest_file = fs::File::create(destination)
        .map_err(|e| Error::Internal(format!("创建目标文件失败 {:?}: {}", destination, e)))?;

    let mut decoder = flate2::read::GzDecoder::new(source_file);
    let size = std::io::copy(&mut decoder, &mut dest_file)
        .map_err(|e| Error::Internal(format!("解压文件失败 {:?}: {}", source, e)))?;

    Ok(size)
}

/// 合并增量备份为新的全量备份
pub fn merge_incremental_backups(
    _base_backup: &Path,
    _incremental_backups: &[&IncrementalManifest],
    _output_path: &Path,
) -> Result<u64> {
    // TODO: 实现备份合并逻辑
    // 1. 从base_backup解压基础文件
    // 2. 按时间顺序应用所有增量变更
    // 3. 压缩结果到output_path
    Err(Error::Internal("待实现".to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn test_file_metadata_from_path() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        let mut file = File::create(&file_path).unwrap();
        file.write_all(b"test content").unwrap();

        let metadata = FileMetadata::from_path(&file_path).unwrap();
        assert_eq!(metadata.size, 12);
        assert!(!metadata.is_dir);
        assert!(!metadata.checksum.is_empty());
    }

    #[test]
    fn test_calculate_file_checksum() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("checksum_test.txt");
        let mut file = File::create(&file_path).unwrap();
        file.write_all(b"checksum test").unwrap();

        let checksum = calculate_file_checksum(&file_path).unwrap();
        assert_eq!(checksum.len(), 64); // SHA256 hash is 64 hex chars
    }

    #[test]
    fn test_detect_file_changes() {
        let temp_dir = TempDir::new().unwrap();

        // 创建测试文件
        let file1 = temp_dir.path().join("file1.txt");
        {
            let mut file = File::create(&file1).unwrap();
            file.write_all(b"original content").unwrap();
        }

        let previous = vec![FileMetadata::from_path(&file1).unwrap()];

        // 修改文件
        {
            let mut file = File::create(&file1).unwrap();
            file.write_all(b"modified content").unwrap();
        }

        let current = vec![FileMetadata::from_path(&file1).unwrap()];

        let changes = calculate_incremental_changes(&previous, &current, DiffStrategy::Checksum);
        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].change_type, FileChangeType::Modified);
    }

    #[test]
    fn test_detect_added_file() {
        let temp_dir = TempDir::new().unwrap();

        // 创建第一个文件
        let file1 = temp_dir.path().join("file1.txt");
        {
            let mut file = File::create(&file1).unwrap();
            file.write_all(b"content").unwrap();
        }

        let previous = vec![FileMetadata::from_path(&file1).unwrap()];

        // 添加第二个文件
        let file2 = temp_dir.path().join("file2.txt");
        {
            let mut file = File::create(&file2).unwrap();
            file.write_all(b"new content").unwrap();
        }

        let current = vec![
            FileMetadata::from_path(&file1).unwrap(),
            FileMetadata::from_path(&file2).unwrap(),
        ];

        let changes = calculate_incremental_changes(&previous, &current, DiffStrategy::Checksum);
        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].change_type, FileChangeType::Added);
    }

    #[test]
    fn test_detect_deleted_file() {
        let temp_dir = TempDir::new().unwrap();

        // 创建文件
        let file1 = temp_dir.path().join("file1.txt");
        let file2 = temp_dir.path().join("file2.txt");
        {
            let mut f1 = File::create(&file1).unwrap();
            f1.write_all(b"content1").unwrap();
            let mut f2 = File::create(&file2).unwrap();
            f2.write_all(b"content2").unwrap();
        }

        let previous = vec![
            FileMetadata::from_path(&file1).unwrap(),
            FileMetadata::from_path(&file2).unwrap(),
        ];

        // 删除第二个文件
        fs::remove_file(&file2).unwrap();

        let current = vec![FileMetadata::from_path(&file1).unwrap()];

        let changes = calculate_incremental_changes(&previous, &current, DiffStrategy::Checksum);
        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].change_type, FileChangeType::Deleted);
    }

    #[test]
    fn test_diff_strategy_timestamp() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("strategy_test.txt");
        let mut file = File::create(&file_path).unwrap();
        file.write_all(b"content").unwrap();

        let meta1 = FileMetadata::from_path(&file_path).unwrap();

        // 等待一小段时间确保修改时间变化
        std::thread::sleep(std::time::Duration::from_millis(100));

        {
            let mut f = File::create(&file_path).unwrap();
            f.write_all(b"content").unwrap(); // 相同内容
        }

        let meta2 = FileMetadata::from_path(&file_path).unwrap();

        // Timestamp策略应该检测到变化
        assert!(meta1.is_modified(&meta2, DiffStrategy::Timestamp));

        // Checksum策略不应该检测到变化
        assert!(!meta1.is_modified(&meta2, DiffStrategy::Checksum));
    }

    #[test]
    fn test_scan_directory() {
        let temp_dir = TempDir::new().unwrap();

        // 创建目录结构
        let dir1 = temp_dir.path().join("dir1");
        let dir2 = dir1.join("dir2");
        fs::create_dir_all(&dir2).unwrap();

        // 创建文件
        let file1 = temp_dir.path().join("file1.txt");
        let file2 = dir2.join("file2.txt");
        {
            let mut f1 = File::create(&file1).unwrap();
            f1.write_all(b"file1").unwrap();
            let mut f2 = File::create(&file2).unwrap();
            f2.write_all(b"file2").unwrap();
        }

        let files = scan_directory(temp_dir.path()).unwrap();

        // 应该找到所有文件和目录
        assert!(files.len() >= 4); // temp_dir, dir1, dir2, file1, file2
        assert!(files.iter().any(|f| f.path == file1 || f.path == file2));
    }
}
