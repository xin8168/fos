    #[tokio::test]
    async fn test_job_creation() {
        let cron = CronExpression::parse("0 * * * * *").unwrap();

        let handler: Arc<dyn JobHandler> = Arc::new(ClosureJobHandler::new(|| {
            Box::pin(async move {
                JobResult {
                    success: true,
                    error: None,
                    execution_time_ms: 100,
                }
            })
        }));

        let job = Job::new("test-job".to_string(), "Test Job".to_string(), cron, handler);

        job.calculate_next_run(Utc::now()).await;

        assert_eq!(job.id, "test-job");
        assert_eq!(job.name, "Test Job");
        assert_eq!(job.max_retries, 3);
    }

    #[tokio::test]
    async fn test_job_execution() {
        let cron = CronExpression::parse("0 * * * * *").unwrap();

        let handler: Arc<dyn JobHandler> = Arc::new(ClosureJobHandler::new(|| {
            Box::pin(async move {
                JobResult {
                    success: true,
                    error: None,
                    execution_time_ms: 100,
                }
            })
        }));

        let job = Job::new("test-job".to_string(), "Test Job".to_string(), cron, handler);

        job.calculate_next_run(Utc::now()).await;

        // 执行任务
        let result = job.execute().await;

        assert!(result.success);

        // 检查统计
        let stats = job.get_stats().await;
        assert_eq!(stats.total_runs, 1);
        assert_eq!(stats.success_count, 1);
    }

    #[tokio::test]
    async fn test_job_with_description() {
        let cron = CronExpression::parse("0 * * * * *").unwrap();

        let handler: Arc<dyn JobHandler> = Arc::new(ClosureJobHandler::new(|| {
            Box::pin(async move {
                JobResult {
                    success: true,
                    error: None,
                    execution_time_ms: 0,
                }
            })
        }));

        let job = Job::new("test-job".to_string(), "Test Job".to_string(), cron, handler)
            .with_description("A test job".to_string());

        assert_eq!(job.description, Some("A test job".to_string()));
    }

    #[tokio::test]
    async fn test_job_with_max_retries() {
        let cron = CronExpression::parse("0 * * * * *").unwrap();

        let handler: Arc<dyn JobHandler> = Arc::new(ClosureJobHandler::new(|| {
            Box::pin(async move {
                JobResult {
                    success: false,
                    error: Some("Test error".to_string()),
                    execution_time_ms: 0,
                }
            })
        }));

        let job = Job::new("test-job".to_string(), "Test Job".to_string(), cron, handler)
            .with_max_retries(5);

        assert_eq!(job.max_retries, 5);
    }

    #[tokio::test]
    async fn test_job_cancel() {
        let cron = CronExpression::parse("0 * * * * *").unwrap();

        let handler: Arc<dyn JobHandler> = Arc::new(ClosureJobHandler::new(|| {
            Box::pin(async move {
                JobResult {
                    success: true,
                    error: None,
                    execution_time_ms: 0,
                }
            })
        }));

        let job = Job::new("test-job".to_string(), "Test Job".to_string(), cron, handler);

        job.cancel().await;

        let status = job.get_status().await;
        assert_eq!(status, JobStatus::Cancelled);
    }

    #[tokio::test]
    async fn test_job_stats_tracking() {
        let cron = CronExpression::parse("0 * * * *").unwrap();

        let handler: Arc<dyn JobHandler> = Arc::new(ClosureJobHandler::new(|| {
            Box::pin(async move {
                JobResult {
                    success: true,
                    error: None,
                    execution_time_ms: 100,
                }
            })
        }));

        let job = Job::new("test-job".to_string(), "Test Job".to_string(), cron, handler);

        // 执行几次
        for _ in 0..5 {
            job.execute().await;
        }

        let stats = job.get_stats().await;
        assert_eq!(stats.total_runs, 5);
        assert_eq!(stats.success_count, 5);
        assert!(stats.avg_execution_time_ms > 0);
    }
}
