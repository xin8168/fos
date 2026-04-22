# FOS Backup Module

## Core Responsibilities
- Execute data backups
- Backup recovery management
- Backup policy configuration

## Components

### backup.rs
- `BackupItem`: Core backup entity with status management
- `BackupType`: Full, Incremental, Differential
- `BackupStatus`: Creating, Created, Verifying, Completed, Failed, Expired
- `BackupPlan`: Scheduled backup configuration
- `BackupSchedule`: Once, Cron, Interval, Daily, Weekly, Monthly
- `BackupRetention`: Retention policies (KeepLatest, Daily, Weekly, Monthly)

### scheduler.rs
- `BackupScheduler`: Task scheduler with notification system
- `BackupExecutor`: Trait for backup execution
- `BackupNotification`: Event notification system

### config.rs
- `Config`: Backup configuration (backup_dir, retention_days)

### error.rs
- `Error`: Error types (Config, Backup, Internal)

## Testing
- Unit tests: 8 passing
- Coverage: BackupItem creation, status transitions, expiration
- Schedule calculations: Daily, Weekly, Monthly
- Scheduler registration

## Dependencies
- chrono: Date/time operations
- uuid: Unique identifiers
- tokio: Async runtime
- async-trait: Async trait support
- gethostname: Hostname retrieval
- serde: Serialization
