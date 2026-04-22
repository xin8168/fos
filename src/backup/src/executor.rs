#![allow(dead_code)]

//! 文件系统备份执行器实现

use crate::{
    backup::BackupItem,
    error::{Error, Result},
    incremental::{
        calculate_incremental_changes, scan_directory, DiffStrategy, FileChange, FileChangeType,
    },
};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use tracing::{debug, info, warn};

/// 文件系统备份执行器
pub struct FsBackupExecutor {
    /// 压缩级别 1-9（1最快，9最小）
    compression_level: u8,
    /// 差异检测策略
    diff_strategy: DiffStrategy,
    /// 上次备份的文件清单（路径 -> 元数据）
    last_backup_files: std::sync::RwLock<HashMap<PathBuf, crate::incremental::FileMetadata>>,
}

impl FsBackupExecutor {
    /// 创建新的文件系统备份执行器
    pub fn new() -> Self {
        Self {
            compression_level: 6,
            diff_strategy: DiffStrategy::Combined,
            last_backup_files: std::sync::RwLock::new(HashMap::new()),
        }
    }

    /// 设置压缩级别
    pub fn with_compression_level(mut self, level: u8) -> Self {
        debug!("设置压缩级别: {}", level);
        self.compression_level = level.min(9).max(1);
        self
    }

    /// 设置差异检测策略
    pub fn with_diff_strategy(mut self, strategy: DiffStrategy) -> Self {
        debug!("设置差异检测策略: {:?}", strategy);
        self.diff_strategy = strategy;
        self
    }

    /// 确保目标目录存在
    fn ensure_backup_dir(path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| Error::Backup(format!("创建备份目录失败 {:?}: {}", parent, e)))?;
        }
        Ok(())
    }

    /// 全量备份
    fn perform_full_backup(
        &self,
        source_dir: &Path,
        backup_item: &mut BackupItem,
    ) -> Result<Vec<FileChange>> {
        info!("开始全量备份: {:?}", source_dir);

        // 扫描源目录
        let files = scan_directory(source_dir)
            .map_err(|e| Error::Backup(format!("扫描目录失败: {}", e)))?;

        let mut changes = Vec::new();
        let mut total_size = 0u64;
        let mut file_count = 0usize;

        // 复制所有文件到备份位置
        for file_meta in &files {
            if file_meta.is_dir {
                continue;
            }

            // 计算备份目标路径
            let relative_path = file_meta
                .path
                .strip_prefix(source_dir)
                .map_err(|e| Error::Backup(format!("计算相对路径失败: {}", e)))?;

            let backup_path = backup_item.path.join(relative_path);

            // 确保目录存在
            if let Some(parent) = backup_path.parent() {
                fs::create_dir_all(parent)
                    .map_err(|e| Error::Backup(format!("创建目录失败 {:?}: {}", parent, e)))?;
            }

            // 复制文件
            fs::copy(&file_meta.path, &backup_path)
                .map_err(|e| Error::Backup(format!("复制文件失败 {:?}: {}", file_meta.path, e)))?;

            total_size += file_meta.size;
            file_count += 1;

            changes.push(FileChange {
                change_type: FileChangeType::Added,
                metadata: Some(file_meta.clone()),
                old_metadata: None,
            });

            debug!("已备份文件: {:?} ({} bytes)", file_meta.path, file_meta.size);
        }

        // 更新备份项统计信息
        backup_item.update_stats(total_size, file_count);

        // 更新上次备份清单
        {
            let mut last_backup = self.last_backup_files.write().unwrap();
            last_backup.clear();
            for file_meta in &files {
                last_backup.insert(file_meta.path.clone(), file_meta.clone());
            }
        }

        info!("全量备份完成: {} 个文件, {} bytes", file_count, total_size);
        Ok(changes)
    }

    /// 增量备份
    fn perform_incremental_backup(
        &self,
        source_dir: &Path,
        backup_item: &mut BackupItem,
    ) -> Result<Vec<FileChange>> {
        info!("开始增量备份: {:?}", source_dir);

        // 获取上次备份清单
        let previous_files: Vec<crate::incremental::FileMetadata> = {
            let last_backup = self.last_backup_files.read().unwrap();
            last_backup.values().cloned().collect()
        };

        if previous_files.is_empty() {
            warn!("上次备份清单为空，执行首次全量备份");
            return self.perform_full_backup(source_dir, backup_item);
        }

        // 扫描当前文件
        let current_files = scan_directory(source_dir)
            .map_err(|e| Error::Backup(format!("扫描目录失败: {}", e)))?;

        // 计算增量变更
        let changes =
            calculate_incremental_changes(&previous_files, &current_files, self.diff_strategy);

        let mut total_size = 0u64;
        let mut file_count = 0usize;

        // 只备份变更的文件
        for change in &changes {
            match change.change_type {
                FileChangeType::Added | FileChangeType::Modified => {
                    if let Some(metadata) = &change.metadata {
                        if metadata.is_dir {
                            continue;
                        }

                        let relative_path = metadata
                            .path
                            .strip_prefix(source_dir)
                            .map_err(|e| Error::Backup(format!("计算相对路径失败: {}", e)))?;

                        let backup_path = backup_item.path.join(relative_path);

                        // 确保目录存在
                        if let Some(parent) = backup_path.parent() {
                            fs::create_dir_all(parent).map_err(|e| {
                                Error::Backup(format!("创建目录失败 {:?}: {}", parent, e))
                            })?;
                        }

                        // 复制文件
                        fs::copy(&metadata.path, &backup_path).map_err(|e| {
                            Error::Backup(format!("复制文件失败 {:?}: {}", metadata.path, e))
                        })?;

                        total_size += metadata.size;
                        file_count += 1;
                    }
                },
                FileChangeType::Deleted => {
                    // 记录删除，但在备份中我们不删除文件
                    debug!("文件已删除: {:?}", change.old_metadata.as_ref().map(|m| &m.path));
                },
                FileChangeType::Unchanged => {
                    // 无需处理
                },
            }
        }

        // 更新备份项统计信息
        backup_item.update_stats(total_size, file_count);

        // 更新上次备份清单
        {
            let mut last_backup = self.last_backup_files.write().unwrap();
            last_backup.clear();
            for file_meta in &current_files {
                last_backup.insert(file_meta.path.clone(), file_meta.clone());
            }
        }

        info!("增量备份完成: {} 个变更文件, {} bytes", file_count, total_size);
        Ok(changes)
    }

    /// 差异备份
    fn perform_differential_backup(
        &self,
        source_dir: &Path,
        base_backup_path: &Path,
        backup_item: &mut BackupItem,
    ) -> Result<Vec<FileChange>> {
        info!("开始差异备份: {:?}", source_dir);

        // 扫描基础备份文件
        let base_files = if let Some(base_files) = scan_directory(base_backup_path).ok() {
            base_files
        } else {
            warn!("无法扫描基础备份，执行全量备份");
            return self.perform_full_backup(source_dir, backup_item);
        };

        // 扫描当前文件
        let current_files = scan_directory(source_dir)
            .map_err(|e| Error::Backup(format!("扫描目录失败: {}", e)))?;

        // 计算差异变更（与基础备份相比）
        let mut base_map: HashMap<PathBuf, crate::incremental::FileMetadata> =
            base_files.iter().map(|f| (f.path.clone(), f.clone())).collect();

        // 将基础路径替换为源路径
        let paths: Vec<PathBuf> = base_map.keys().cloned().collect();
        for path in paths {
            if let Ok(relative) = path.strip_prefix(base_backup_path) {
                if let Some(new_path) = base_map.remove(&path) {
                    let source_path = source_dir.join(relative);
                    let mut new_meta = new_path;
                    new_meta.path = source_path.clone();
                    base_map.insert(source_path, new_meta);
                }
            }
        }

        let changes = calculate_incremental_changes(
            &base_map.values().cloned().collect::<Vec<_>>(),
            &current_files,
            self.diff_strategy,
        );

        let mut total_size = 0u64;
        let mut file_count = 0usize;

        // 备份差异文件
        for change in &changes {
            if matches!(change.change_type, FileChangeType::Added | FileChangeType::Modified) {
                if let Some(metadata) = &change.metadata {
                    if metadata.is_dir {
                        continue;
                    }

                    let relative_path = metadata
                        .path
                        .strip_prefix(source_dir)
                        .map_err(|e| Error::Backup(format!("计算相对路径失败: {}", e)))?;

                    let backup_path = backup_item.path.join(relative_path);

                    if let Some(parent) = backup_path.parent() {
                        fs::create_dir_all(parent).map_err(|e| {
                            Error::Backup(format!("创建目录失败 {:?}: {}", parent, e))
                        })?;
                    }

                    fs::copy(&metadata.path, &backup_path).map_err(|e| {
                        Error::Backup(format!("复制文件失败 {:?}: {}", metadata.path, e))
                    })?;

                    total_size += metadata.size;
                    file_count += 1;
                }
            }
        }

        backup_item.update_stats(total_size, file_count);

        info!("差异备份完成: {} 个差异文件, {} bytes", file_count, total_size);
        Ok(changes)
    }
}

impl Default for FsBackupExecutor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fs_backup_executor_creation() {
        let executor = FsBackupExecutor::new();
        assert_eq!(executor.compression_level, 6);
        assert_eq!(executor.diff_strategy, DiffStrategy::Combined);
    }

    #[test]
    fn test_fs_backup_executor_with_options() {
        let executor = FsBackupExecutor::new()
            .with_compression_level(9)
            .with_diff_strategy(DiffStrategy::Checksum);

        assert_eq!(executor.compression_level, 9);
        assert_eq!(executor.diff_strategy, DiffStrategy::Checksum);
    }

    #[test]
    fn test_fs_backup_executor_default() {
        let executor = FsBackupExecutor::default();
        assert_eq!(executor.compression_level, 6);
    }
}
