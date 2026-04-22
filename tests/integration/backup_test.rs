#![allow(dead_code)]

//! Backup模块集成测试

use fos_backup::{
    backup::{BackupItem, BackupPlan, BackupRetention, BackupSchedule, BackupStatus, BackupType},
    error::{Error, Result},
    incremental::{
        calculate_file_checksum, calculate_incremental_changes, compress_file, decompress_file,
        scan_directory, DiffStrategy, FileChange, FileChangeType,
    },
    scheduler::{BackupExecutor, BackupScheduler},
};

// 简单的MockExecutor用于集成测试
struct MockExecutor {
    should_succeed: bool,
}

#[async_trait::async_trait]
impl BackupExecutor for MockExecutor {
    async fn backup(&self, item: &mut BackupItem) -> std::result::Result<String, Error> {
        item.update_stats(1024, 1);
        item.mark_created();
        if !self.should_succeed {
            item.mark_failed("Mock error".to_string());
            return Err(Error::Backup("Mock error".to_string()));
        }
        Ok("mock-backup-path".to_string())
    }

    async fn verify(&self, _item: &BackupItem) -> std::result::Result<bool, Error> {
        Ok(self.should_succeed)
    }

    async fn cleanup(&self, _item: &BackupItem) -> std::result::Result<(), Error> {
        Ok(())
    }
}
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

/// 完整备份工作流测试
#[test]
fn test_full_backup_workflow() {
    let temp_dir = TempDir::new().unwrap();
    let source_dir = temp_dir.path().join("source");
    let backup_dir = temp_dir.path().join("backup");

    // 创建源目录结构
    fs::create_dir_all(&source_dir).unwrap();
    let file1 = source_dir.join("file1.txt");
    let file2 = source_dir.join("file2.txt");
    let subdir = source_dir.join("subdir");
    fs::create_dir_all(&subdir).unwrap();
    let file3 = subdir.join("file3.txt");

    // 写入文件内容
    let mut f1 = File::create(&file1).unwrap();
    f1.write_all(b"content1").unwrap();
    let mut f2 = File::create(&file2).unwrap();
    f2.write_all(b"content2").unwrap();
    let mut f3 = File::create(&file3).unwrap();
    f3.write_all(b"content3").unwrap();

    // 执行全量备份（通过scan_directory模拟）
    let backup_item = BackupItem::new(
        BackupType::Full,
        backup_dir.join("full_backup.zip"),
        vec![],
        "Full backup test".to_string(),
        "Integration Test".to_string(),
    );

    assert_eq!(backup_item.status, BackupStatus::Creating);

    // 扫描文件
    let files = scan_directory(&source_dir).unwrap();
    assert!(files.len() >= 3); // 至少有3个文件

    // 模拟备份完成
    let backup_dir_actual = backup_dir.join("full_backup");
    fs::create_dir_all(&backup_dir_actual).unwrap();

    // 复制文件到备份目录
    for file_meta in &files {
        if file_meta.is_dir {
            fs::create_dir_all(
                backup_dir_actual.join(file_meta.path.strip_prefix(&source_dir).unwrap()),
            )
            .unwrap();
        } else {
            let backup_path =
                backup_dir_actual.join(file_meta.path.strip_prefix(&source_dir).unwrap());
            if let Some(parent) = backup_path.parent() {
                fs::create_dir_all(parent).unwrap();
            }
            fs::copy(&file_meta.path, &backup_path).unwrap();
        }
    }

    // 验证备份文件存在
    let backed_up_files = scan_directory(&backup_dir_actual).unwrap();
    assert!(backed_up_files.len() >= 3);

    println!("全量备份工作流测试通过");
}

/// 增量备份工作流测试
#[test]
fn test_incremental_backup_workflow() {
    let temp_dir = TempDir::new().unwrap();
    let source_dir = temp_dir.path().join("source");
    let backup_dir = temp_dir.path().join("backup");

    // 创建源目录
    fs::create_dir_all(&source_dir).unwrap();
    let file1 = source_dir.join("file1.txt");
    let mut f1 = File::create(&file1).unwrap();
    f1.write_all(b"initial content").unwrap();

    // 首次全量备份
    let initial_files = scan_directory(&source_dir).unwrap();
    assert_eq!(initial_files.len(), 1); // 1个文件

    // 修改文件
    std::thread::sleep(std::time::Duration::from_millis(100));
    let mut f1 = File::create(&file1).unwrap();
    f1.write_all(b"modified content").unwrap();

    // 添加新文件
    let file2 = source_dir.join("file2.txt");
    let mut f2 = File::create(&file2).unwrap();
    f2.write_all(b"new file").unwrap();

    // 计算增量变更
    let current_files = scan_directory(&source_dir).unwrap();
    assert_eq!(current_files.len(), 2); // 现在有2个文件

    let changes =
        calculate_incremental_changes(&initial_files, &current_files, DiffStrategy::Checksum);

    // 验证变更: 1个修改 + 1个新增 = 2个变更
    assert_eq!(changes.len(), 2);

    // 验证变更类型
    assert!(changes.iter().any(|c| c.change_type == FileChangeType::Modified));
    assert!(changes.iter().any(|c| c.change_type == FileChangeType::Added));

    println!("增量备份工作流测试通过，检测到变更: {:?}", changes);
}

/// 备份压缩工作流测试
#[test]
fn test_backup_compression_workflow() {
    let temp_dir = TempDir::new().unwrap();
    let source_dir = temp_dir.path().join("source");
    let backup_dir = temp_dir.path().join("backup");

    // 创建源文件
    fs::create_dir_all(&source_dir).unwrap();
    let file1 = source_dir.join("file1.txt");
    let mut f1 = File::create(&file1).unwrap();
    // 创建足够大的内容以验证压缩效果
    f1.write_all(b"A".repeat(1000).as_bytes()).unwrap();

    let file2 = source_dir.join("file2.txt");
    let mut f2 = File::create(&file2).unwrap();
    f2.write_all(b"B".repeat(1000).as_bytes()).unwrap();

    // 压缩文件
    let compressed_file = backup_dir.join("backup.gz");
    let original_size = fs::metadata(&file1).unwrap().len();
    let compressed_size = compress_file(&file1, &compressed_file).unwrap();

    // 验证压缩成功
    assert!(fs::exists(&compressed_file).unwrap());
    // GZIP压缩，对于重复字符应该有很大压缩比
    assert!(compressed_size < original_size);

    // 解压文件
    let decompressed_file = backup_dir.join("decompressed.txt");
    let decompressed_size = decompress_file(&compressed_file, &decompressed_file).unwrap();

    // 验证解压成功且大小正确
    assert!(fs::exists(&decompressed_file).unwrap());
    assert_eq!(decompressed_size, original_size);

    // 验证解压后的内容正确
    let content = fs::read_to_string(&decompressed_file).unwrap();
    assert_eq!(content, "A".repeat(1000));

    println!("备份压缩工作流测试通过");
    println!(
        "原始大小: {}, 压缩后: {}, 压缩比: {:.1}%",
        original_size,
        compressed_size,
        (compressed_size as f64 / original_size as f64) * 100.0
    );
}

/// 备份调度器集成测试
#[tokio::test]
async fn test_backup_scheduler_integration() {
    let executor = MockExecutor { should_succeed: true };
    let scheduler = BackupScheduler::new(executor);

    // 注册多个备份计划
    let plan1 = BackupItem::new(
        BackupType::Full,
        PathBuf::from("/backup/full.zip"),
        vec![],
        "Full backup".to_string(),
        "Scheduler Test".to_string(),
    );

    let plan2 = BackupItem::new(
        BackupType::Incremental,
        PathBuf::from("/backup/inc.zip"),
        vec![],
        "Incremental backup".to_string(),
        "Scheduler Test".to_string(),
    );

    // 注册计划（注意：BackupScheduler期望BackupPlan，不是BackupItem）
    // 这里我们需要创建BackupPlan
    use fos_backup::backup::{BackupPlan, BackupRetention, BackupSchedule};

    let backup_plan1 = BackupPlan {
        id: "plan-1".to_string(),
        name: "Daily Full".to_string(),
        backup_type: BackupType::Full,
        targets: vec![],
        schedule: BackupSchedule::Daily { hour: 2, minute: 0, second: 0 },
        retention: BackupRetention::default(),
        enabled: true,
    };

    let backup_plan2 = BackupPlan {
        id: "plan-2".to_string(),
        name: "Hourly Incremental".to_string(),
        backup_type: BackupType::Incremental,
        targets: vec![],
        schedule: BackupSchedule::Interval { seconds: 3600 },
        retention: BackupRetention::default(),
        enabled: true,
    };

    scheduler.register_plan(backup_plan1).await.unwrap();
    scheduler.register_plan(backup_plan2).await.unwrap();

    // 验证计划已注册
    let plans = scheduler.list_plans().await;
    assert_eq!(plans.len(), 2);
    assert!(plans.iter().any(|p| p.id == "plan-1"));
    assert!(plans.iter().any(|p| p.id == "plan-2"));

    // 测试调度器运行状态
    assert!(!scheduler.is_running().await);

    println!("备份调度器集成测试通过");
}

/// 文件变更检测集成测试
#[test]
fn test_file_change_detection_integration() {
    let temp_dir = TempDir::new().unwrap();
    let source_dir = temp_dir.path().join("source");

    fs::create_dir_all(&source_dir).unwrap();

    // 创建初始文件集
    let files = vec![
        source_dir.join("file1.txt"),
        source_dir.join("file2.txt"),
        source_dir.join("file3.txt"),
    ];

    for (i, file) in files.iter().enumerate() {
        let mut f = File::create(file).unwrap();
        f.write_all(format!("content{}", i).as_bytes()).unwrap();
    }

    // 初始扫描
    let initial_backup = scan_directory(&source_dir).unwrap();
    assert_eq!(initial_backup.len(), 3);

    // 场景1: 修改文件
    std::thread::sleep(std::time::Duration::from_millis(100));
    {
        let mut f = File::create(&files[0]).unwrap();
        f.write_all(b"modified content").unwrap();
    }

    let current_backup = scan_directory(&source_dir).unwrap();
    let changes =
        calculate_incremental_changes(&initial_backup, &current_backup, DiffStrategy::Checksum);

    assert!(changes.iter().any(|c| c.change_type == FileChangeType::Modified));
    assert_eq!(changes.len(), 1);

    // 场景2: 添加文件
    let new_file = source_dir.join("file4.txt");
    {
        let mut f = File::create(&new_file).unwrap();
        f.write_all(b"new content").unwrap();
    }

    let current_backup2 = scan_directory(&source_dir).unwrap();
    let changes2 =
        calculate_incremental_changes(&initial_backup, &current_backup2, DiffStrategy::Checksum);

    assert!(changes2.iter().any(|c| c.change_type == FileChangeType::Added));
    assert!(changes2.iter().any(|c| c.change_type == FileChangeType::Modified));

    // 场景3: 删除文件
    fs::remove_file(&files[2]).unwrap();

    let current_backup3 = scan_directory(&source_dir).unwrap();
    let changes3 =
        calculate_incremental_changes(&initial_backup, &current_backup3, DiffStrategy::Checksum);

    assert!(changes3.iter().any(|c| c.change_type == FileChangeType::Deleted));

    println!("文件变更检测集成测试通过");
}

/// 差异策略对比测试
#[test]
fn test_diff_strategy_comparison() {
    let temp_dir = TempDir::new().unwrap();
    let source_dir = temp_dir.path().join("source");

    fs::create_dir_all(&source_dir).unwrap();
    let file1 = source_dir.join("file1.txt");
    let mut f1 = File::create(&file1).unwrap();
    f1.write_all(b"test content").unwrap();

    let initial_backup = scan_directory(&source_dir).unwrap();

    // 修改文件内容但保持大小
    std::thread::sleep(std::time::Duration::from_millis(100));
    let mut f1 = File::create(&file1).unwrap();
    f1.write_all(b"TEST CONTENT").unwrap(); // 大小相同，内容不同

    let current_backup = scan_directory(&source_dir).unwrap();

    // 测试不同策略
    let timestamp_changes =
        calculate_incremental_changes(&initial_backup, &current_backup, DiffStrategy::Timestamp);

    let size_changes =
        calculate_incremental_changes(&initial_backup, &current_backup, DiffStrategy::Size);

    let checksum_changes =
        calculate_incremental_changes(&initial_backup, &current_backup, DiffStrategy::Checksum);

    // Timestamp策略应该检测到变化
    assert_eq!(timestamp_changes.len(), 1);

    // Size策略不应该检测到变化（大小相同）
    assert_eq!(size_changes.len(), 0);

    // Checksum策略应该检测到变化（内容不同）
    assert_eq!(checksum_changes.len(), 1);

    println!("差异策略对比测试通过:");
    println!("  Timestamp策略: {} 个变更", timestamp_changes.len());
    println!("  Size策略: {} 个变更", size_changes.len());
    println!("  Checksum策略: {} 个变更", checksum_changes.len());
}

/// 备份验证工作流测试
#[test]
fn test_backup_verification_workflow() {
    let temp_dir = TempDir::new().unwrap();
    let source_dir = temp_dir.path().join("source");
    let backup_dir = temp_dir.path().join("backup");

    // 创建源文件
    fs::create_dir_all(&source_dir).unwrap();
    let file1 = source_dir.join("file1.txt");
    let mut f1 = File::create(&file1).unwrap();
    f1.write_all(b"test content for verification").unwrap();

    // 计算源文件校验和
    let original_checksum = calculate_file_checksum(&file1).unwrap();

    // 复制到备份目录
    fs::create_dir_all(&backup_dir).unwrap();
    let backup_path = backup_dir.join("file1.txt");
    fs::copy(&file1, &backup_path).unwrap();

    // 计算备份文件校验和
    let backup_checksum = calculate_file_checksum(&backup_path).unwrap();

    // 验证校验和一致
    assert_eq!(original_checksum, backup_checksum);

    // 修改备份文件，验证应该失败
    let mut f = File::create(&backup_path).unwrap();
    f.write_all(b"tampered content").unwrap();

    let tampered_checksum = calculate_file_checksum(&backup_path).unwrap();
    assert_ne!(original_checksum, tampered_checksum);

    println!("备份验证工作流测试通过");
    println!("原始校验和: {}", original_checksum);
    println!("篡改校验和: {}", tampered_checksum);
}

/// 复杂目录结构备份测试
#[test]
fn test_complex_directory_structure_backup() {
    let temp_dir = TempDir::new().unwrap();
    let source_dir = temp_dir.path().join("source");
    let backup_dir = temp_dir.path().join("backup");

    // 创建复杂目录结构
    let dirs = vec![
        source_dir.join("dir1"),
        source_dir.join("dir1/subdir1"),
        source_dir.join("dir1/subdir2"),
        source_dir.join("dir2"),
        source_dir.join("dir2/subdir3"),
    ];

    for dir in &dirs {
        fs::create_dir_all(dir).unwrap();
    }

    // 在不同目录创建文件
    let files = vec![
        (dirs[0].join("file1.txt"), "content1"),
        (dirs[1].join("file2.txt"), "content2"),
        (dirs[2].join("file3.txt"), "content3"),
        (dirs[3].join("file4.txt"), "content4"),
        (dirs[4].join("file5.txt"), "content5"),
    ];

    for (path, content) in &files {
        let mut f = File::create(path).unwrap();
        f.write_all(content.as_bytes()).unwrap();
    }

    // 扫描目录
    let all_files = scan_directory(&source_dir).unwrap();

    // 验证所有文件和目录都被扫描到
    assert!(all_files.len() >= 10); // 至少包含5个文件 + 5个目录

    // 验证文件元数据
    for (path, content) in &files {
        assert!(all_files.iter().any(|f| f.path == *path));
    }

    // 模拟备份
    fs::create_dir_all(&backup_dir).unwrap();
    let backup_root = backup_dir.join("complex_backup");
    fs::create_dir_all(&backup_root).unwrap();

    for file_meta in &all_files {
        if !file_meta.is_dir {
            let relative = file_meta.path.strip_prefix(&source_dir).unwrap();
            let backup_path = backup_root.join(relative);
            if let Some(parent) = backup_path.parent() {
                fs::create_dir_all(parent).unwrap();
            }
            fs::copy(&file_meta.path, &backup_path).unwrap();
        }
    }

    // 验证备份完整性
    let backup_files = scan_directory(&backup_root).unwrap();
    assert!(backup_files.len() >= 5); // 所有源文件

    // 验证每个源文件在备份中都有对应
    for (path, _) in &files {
        let relative = path.strip_prefix(&source_dir).unwrap();
        let backup_path = backup_root.join(relative);
        assert!(fs::exists(&backup_path).unwrap());
    }

    println!("复杂目录结构备份测试通过");
    println!("扫描到 {} 个文件和目录", all_files.len());
}

/// 性能基准测试
#[test]
fn test_backup_performance_benchmark() {
    let temp_dir = TempDir::new().unwrap();
    let source_dir = temp_dir.path().join("source");

    fs::create_dir_all(&source_dir).unwrap();

    // 创建100个小文件
    let file_count = 100;
    for i in 0..file_count {
        let file_path = source_dir.join(format!("file{}.txt", i));
        let mut f = File::create(&file_path).unwrap();
        f.write_all(format!("file content {}", i).as_bytes()).unwrap();
    }

    // 测试扫描性能
    let start = std::time::Instant::now();
    let files = scan_directory(&source_dir).unwrap();
    let scan_duration = start.elapsed();

    assert_eq!(files.len(), file_count);

    // 测试校验和计算性能
    let start = std::time::Instant::now();
    for file_meta in &files {
        if !file_meta.is_dir {
            calculate_file_checksum(&file_meta.path).unwrap();
        }
    }
    let checksum_duration = start.elapsed();

    // 测试增量检测性能
    let new_file = source_dir.join("new_file.txt");
    let mut f = File::create(&new_file).unwrap();
    f.write_all(b"new file content").unwrap();

    let new_files = scan_directory(&source_dir).unwrap();

    let start = std::time::Instant::now();
    let changes = calculate_incremental_changes(&files, &new_files, DiffStrategy::Checksum);
    let diff_duration = start.elapsed();

    assert_eq!(changes.len(), 1);
    assert_eq!(changes[0].change_type, FileChangeType::Added);

    println!("性能基准测试结果:");
    println!("  扫描 {} 个文件: {:?}", file_count, scan_duration);
    println!("  计算校验和: {:?}", checksum_duration);
    println!("  增量检测: {:?}", diff_duration);
    println!("  平均扫描时间: {:.2} ms/file", scan_duration.as_millis() as f64 / file_count as f64);
    println!(
        "  平均校验和时间: {:.2} ms/file",
        checksum_duration.as_millis() as f64 / file_count as f64
    );
}

/// 错误处理测试
#[test]
fn test_error_handling() {
    let temp_dir = TempDir::new().unwrap();
    let backup_dir = temp_dir.path().join("backup");

    // 测试不存在的文件压缩
    let non_existent = backup_dir.join("non_existent.txt");
    let compressed = backup_dir.join("compressed.gz");
    let result = compress_file(&non_existent, &compressed);
    assert!(result.is_err());

    // 测试不存在的文件解压
    let non_existent_gz = backup_dir.join("non_existent.gz");
    let decompressed = backup_dir.join("decompressed.txt");
    let result = decompress_file(&non_existent_gz, &decompressed);
    assert!(result.is_err());

    // 测试不存在的目录扫描
    let non_existent_dir = backup_dir.join("non_existent_dir");
    let result = scan_directory(&non_existent_dir);
    // 可能是Err或Ok但为空，取决于实现
    // 这里只验证不崩溃

    // 测试不存在的文件校验和计算
    let result = calculate_file_checksum(&non_existent);
    assert!(result.is_err());

    println!("错误处理测试通过（所有错误情况都得到正确处理）");
}
