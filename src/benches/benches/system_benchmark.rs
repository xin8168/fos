//! 全链路性能基准测试
//!
//! 对FOS核心模块进行性能基准测试

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use fos_cache::LocalCache;
use fos_lock::LockManager;
use fos_rollback::Snapshot;
use std::sync::Arc;

/// 缓存性能测试
fn benchmark_cache(c: &mut Criterion) {
    let mut group = c.benchmark_group("cache_operations");

    for size in [100, 1_000, 10_000].iter() {
        group.bench_with_input(BenchmarkId::new("set", size), size, |b, size| {
            let rt = tokio::runtime::Runtime::new().unwrap();
            let cache = LocalCache::new();

            b.iter(|| {
                rt.block_on(async {
                    let key = format!("bench_key_{}", black_box(*size));
                    let value = format!("bench_value_{}", black_box(*size));
                    cache.set(key, value, None).await;
                })
            });
        });

        group.bench_with_input(BenchmarkId::new("get", size), size, |b, size| {
            let rt = tokio::runtime::Runtime::new().unwrap();

            b.iter(|| {
                rt.block_on(async {
                    let cache = LocalCache::new();
                    let key = format!("bench_key_{}", black_box(*size));
                    let value = format!("bench_value_{}", black_box(*size));
                    cache.set(key.clone(), value, None).await;
                    cache.get(&key).await
                })
            });
        });

        group.bench_with_input(BenchmarkId::new("exists", size), size, |b, size| {
            let rt = tokio::runtime::Runtime::new().unwrap();

            b.iter(|| {
                rt.block_on(async {
                    let cache = LocalCache::new();
                    let key = format!("bench_key_{}", black_box(*size));
                    cache.set(key.clone(), "value".to_string(), None).await;
                    cache.exists(&key).await
                })
            });
        });

        group.bench_with_input(BenchmarkId::new("keys", size), size, |b, size| {
            let rt = tokio::runtime::Runtime::new().unwrap();
            let cache = Arc::new(LocalCache::new());

            rt.block_on(async {
                for i in 0..*size {
                    cache.set(format!("key_{}", i), i, None).await;
                }
            });

            b.iter(|| rt.block_on(async { cache.keys().await }));
        });
    }

    group.finish();
}

/// 分布式锁性能测试
fn benchmark_lock(c: &mut Criterion) {
    let mut group = c.benchmark_group("lock_operations");

    let lock_manager = LockManager::with_defaults();

    group.bench_function("try_lock", |b| {
        b.iter(|| {
            let key = format!(
                "bench_lock_{}",
                black_box(
                    std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_nanos()
                )
            );
            let owner = format!("bench_owner_{}", black_box(1));
            lock_manager.try_lock(&key, &owner)
        });
    });

    group.bench_function("unlock", |b| {
        b.iter_with_setup(
            || {
                let key = format!(
                    "bench_lock_{}",
                    std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_nanos()
                );
                let owner = "bench_owner".to_string();
                lock_manager.try_lock(&key, &owner).unwrap();
                (key, owner)
            },
            |(key, owner)| lock_manager.unlock(&key, &owner),
        );
    });

    group.bench_function("is_locked", |b| {
        b.iter_with_setup(
            || {
                let key = format!(
                    "bench_lock_{}",
                    std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_nanos()
                );
                (key.clone(), key.clone())
            },
            |(key1, key2)| {
                lock_manager.is_locked(&key1);
                lock_manager.is_locked(&key2)
            },
        );
    });

    group.bench_function("get_lock", |b| {
        b.iter_with_setup(
            || {
                let key = format!(
                    "bench_lock_{}",
                    std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_nanos()
                );
                let owner = "bench_owner".to_string();
                lock_manager.try_lock(&key, &owner).unwrap();
                key
            },
            |key| lock_manager.get_lock(&key),
        );
    });

    group.finish();
}

/// 快照创建性能测试
fn benchmark_snapshot(c: &mut Criterion) {
    let mut group = c.benchmark_group("snapshot_operations");

    for data_size in [10, 100, 500].iter() {
        group.bench_with_input(
            BenchmarkId::new("create_snapshot", data_size),
            data_size,
            |b, data_size| {
                let mut data = serde_json::Map::new();
                for i in 0..*data_size {
                    data.insert(
                        format!("key_{}", i),
                        serde_json::Value::String(format!("value_{}", i)),
                    );
                }

                let json_data = serde_json::Value::Object(data);

                b.iter(|| {
                    Snapshot::new(
                        format!(
                            "bench_operation_{}",
                            black_box(
                                std::time::SystemTime::now()
                                    .duration_since(std::time::UNIX_EPOCH)
                                    .unwrap()
                                    .as_nanos()
                            )
                        ),
                        fos_rollback::SnapshotType::Full,
                        json_data.clone(),
                    )
                });
            },
        );
    }

    group.finish();
}

/// 并发操作性能测试
fn benchmark_concurrent_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("concurrent_operations");

    for thread_count in [4, 8, 16].iter() {
        group.bench_with_input(
            BenchmarkId::new("concurrent_cache_set_get", thread_count),
            thread_count,
            |b, thread_count| {
                let rt = tokio::runtime::Runtime::new().unwrap();
                let cache = Arc::new(LocalCache::new());

                b.iter(|| {
                    rt.block_on(async {
                        let mut handles = vec![];

                        for i in 0..*thread_count {
                            let cache_clone = cache.clone();
                            let handle = tokio::spawn(async move {
                                let key = format!("concurrent_key_{}", i);
                                cache_clone.set(key.clone(), format!("value_{}", i), None).await;
                                cache_clone.get(&key).await
                            });
                            handles.push(handle);
                        }

                        for handle in handles {
                            handle.await.unwrap();
                        }
                    })
                });
            },
        );
    }

    group.finish();
}

/// 内存分配和序列化性能测试
fn benchmark_memory(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_operations");

    for data_size in [10, 100, 500].iter() {
        group.bench_with_input(
            BenchmarkId::new("json_serialize", data_size),
            data_size,
            |b, data_size| {
                let mut data = serde_json::Map::new();
                for i in 0..*data_size {
                    data.insert(
                        format!("key_{}", i),
                        serde_json::Value::String(format!("value_{}", i)),
                    );
                }

                b.iter(|| serde_json::to_string(&black_box(&data)));
            },
        );

        group.bench_with_input(
            BenchmarkId::new("json_deserialize", data_size),
            data_size,
            |b, data_size| {
                let mut data = serde_json::Map::new();
                for i in 0..*data_size {
                    data.insert(
                        format!("key_{}", i),
                        serde_json::Value::String(format!("value_{}", i)),
                    );
                }

                let json_str = serde_json::to_string(&data).unwrap();

                b.iter(|| {
                    let _: serde_json::Value = serde_json::from_str(&black_box(&json_str)).unwrap();
                });
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    benchmark_cache,
    benchmark_lock,
    benchmark_snapshot,
    benchmark_concurrent_operations,
    benchmark_memory
);
criterion_main!(benches);
