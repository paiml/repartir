#![allow(clippy::unwrap_used, clippy::expect_used)]
//! Integration tests for repartir
//!
//! These tests verify end-to-end workflows combining multiple components.

use repartir::executor::cpu::CpuExecutor;
use repartir::executor::Executor;
use repartir::scheduler::Scheduler;
use repartir::task::{Backend, Priority, Task};
use std::time::Duration;

#[tokio::test]
async fn test_cpu_executor_basic_workflow() {
    let executor = CpuExecutor::new();

    #[cfg(unix)]
    {
        let task = Task::builder()
            .binary("/bin/echo")
            .arg("integration test")
            .backend(Backend::Cpu)
            .build()
            .unwrap();

        let result = executor.execute(task).await.unwrap();
        assert!(result.is_success());
        assert_eq!(result.stdout_str().unwrap().trim(), "integration test");
    }
}

#[tokio::test]
async fn test_scheduler_cpu_executor_workflow() {
    let scheduler = Scheduler::new();
    let executor = CpuExecutor::new();

    #[cfg(unix)]
    {
        // Submit multiple tasks
        let mut task_ids = Vec::new();
        for i in 0..5 {
            let task = Task::builder()
                .binary("/bin/echo")
                .arg(format!("task_{i}"))
                .backend(Backend::Cpu)
                .build()
                .unwrap();

            let task_id = scheduler.submit(task).await.unwrap();
            task_ids.push(task_id);
        }

        // Execute tasks
        for _ in 0..5 {
            if let Some(task) = scheduler.next_task().await {
                let result = executor.execute(task).await.unwrap();
                assert!(result.is_success());
                scheduler.store_result(result).await;
            }
        }

        // Retrieve results
        for task_id in task_ids {
            let result = scheduler.get_result(task_id).await;
            assert!(result.is_some());
            assert!(result.unwrap().is_success());
        }
    }
}

#[tokio::test]
async fn test_priority_scheduling_integration() {
    let scheduler = Scheduler::new();

    #[cfg(unix)]
    {
        // Submit tasks in reverse priority order
        let low_task = Task::builder()
            .binary("/bin/echo")
            .arg("low")
            .backend(Backend::Cpu)
            .priority(Priority::Low)
            .build()
            .unwrap();
        scheduler.submit(low_task).await.unwrap();

        let normal_task = Task::builder()
            .binary("/bin/echo")
            .arg("normal")
            .backend(Backend::Cpu)
            .priority(Priority::Normal)
            .build()
            .unwrap();
        scheduler.submit(normal_task).await.unwrap();

        let high_task = Task::builder()
            .binary("/bin/echo")
            .arg("high")
            .backend(Backend::Cpu)
            .priority(Priority::High)
            .build()
            .unwrap();
        scheduler.submit(high_task).await.unwrap();

        // Tasks should be retrieved in priority order
        let task1 = scheduler.next_task().await.unwrap();
        assert_eq!(task1.priority(), Priority::High);

        let task2 = scheduler.next_task().await.unwrap();
        assert_eq!(task2.priority(), Priority::Normal);

        let task3 = scheduler.next_task().await.unwrap();
        assert_eq!(task3.priority(), Priority::Low);
    }
}

#[tokio::test]
async fn test_task_timeout_integration() {
    let executor = CpuExecutor::new();

    #[cfg(unix)]
    {
        let task = Task::builder()
            .binary("/bin/sleep")
            .arg("5")
            .backend(Backend::Cpu)
            .timeout(Duration::from_millis(100))
            .build()
            .unwrap();

        let result = executor.execute(task).await;
        assert!(result.is_err());
    }
}

#[tokio::test]
async fn test_task_with_environment_variables() {
    let executor = CpuExecutor::new();

    #[cfg(unix)]
    {
        let task = Task::builder()
            .binary("/bin/sh")
            .arg("-c")
            .arg("echo $MY_VAR")
            .env_var("MY_VAR", "integration_test_value")
            .backend(Backend::Cpu)
            .build()
            .unwrap();

        let result = executor.execute(task).await.unwrap();
        assert!(result.is_success());
        assert_eq!(
            result.stdout_str().unwrap().trim(),
            "integration_test_value"
        );
    }
}

#[tokio::test]
async fn test_concurrent_task_execution() {
    use std::sync::Arc;

    let scheduler = Arc::new(Scheduler::with_capacity(100));
    let _executor = CpuExecutor::new();

    #[cfg(unix)]
    {
        // Submit 10 tasks concurrently
        let mut handles = Vec::new();
        for i in 0..10 {
            let task = Task::builder()
                .binary("/bin/echo")
                .arg(format!("concurrent_{i}"))
                .backend(Backend::Cpu)
                .build()
                .unwrap();

            scheduler.submit(task).await.unwrap();
        }

        // Execute tasks concurrently
        for _ in 0..10 {
            let scheduler_clone = Arc::clone(&scheduler);
            let handle = tokio::spawn(async move {
                if let Some(task) = scheduler_clone.next_task().await {
                    let executor = CpuExecutor::new();
                    executor.execute(task).await
                } else {
                    Err(repartir::error::RepartirError::InvalidTask {
                        reason: "No task available".to_string(),
                    })
                }
            });
            handles.push(handle);
        }

        // Wait for all tasks
        for handle in handles {
            let result = handle.await.unwrap();
            assert!(result.is_ok());
        }
    }
}

#[tokio::test]
async fn test_scheduler_clear_integration() {
    let scheduler = Scheduler::new();

    #[cfg(unix)]
    {
        // Submit tasks
        for i in 0..5 {
            let task = Task::builder()
                .binary("/bin/echo")
                .arg(format!("task_{i}"))
                .backend(Backend::Cpu)
                .build()
                .unwrap();
            scheduler.submit(task).await.unwrap();
        }

        assert_eq!(scheduler.pending_count().await, 5);

        // Clear scheduler
        scheduler.clear().await;
        assert_eq!(scheduler.pending_count().await, 0);
    }
}

#[tokio::test]
async fn test_error_handling_integration() {
    let executor = CpuExecutor::new();

    // Test nonexistent binary
    let task = Task::builder()
        .binary("/nonexistent/binary/path")
        .backend(Backend::Cpu)
        .build()
        .unwrap();

    let result = executor.execute(task).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_non_zero_exit_code_integration() {
    let executor = CpuExecutor::new();

    #[cfg(unix)]
    {
        let task = Task::builder()
            .binary("/bin/sh")
            .arg("-c")
            .arg("exit 123")
            .backend(Backend::Cpu)
            .build()
            .unwrap();

        let result = executor.execute(task).await.unwrap();
        assert!(!result.is_success());
        assert_eq!(result.exit_code(), 123);
    }
}
