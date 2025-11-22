//! Task scheduling and work distribution.
//!
//! Implements work-stealing scheduler based on Blumofe & Leiserson (1999).

use crate::error::{RepartirError, Result};
use crate::task::{ExecutionResult, Task, TaskId};
use std::collections::{BinaryHeap, HashMap};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info};
use uuid::Uuid;

/// Unique identifier for a worker.
///
/// Per Iron Lotus Framework case study (Section 12.3),
/// we use UUIDs instead of indices to prevent invalidation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WorkerId(Uuid);

impl WorkerId {
    /// Creates a new random worker ID.
    #[must_use]
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for WorkerId {
    fn default() -> Self {
        Self::new()
    }
}

/// Task wrapper for priority queue.
///
/// Implements `Ord` to enable priority-based scheduling.
#[derive(Debug)]
struct PriorityTask {
    task: Task,
}

impl PartialEq for PriorityTask {
    fn eq(&self, other: &Self) -> bool {
        self.task.priority() == other.task.priority()
    }
}

impl Eq for PriorityTask {}

impl PartialOrd for PriorityTask {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PriorityTask {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // BinaryHeap is a max-heap, so reverse order for min-heap behavior
        self.task.priority().cmp(&other.task.priority())
    }
}

/// Simple scheduler for task distribution.
///
/// This is a v1.0 implementation. Future versions will implement
/// true work-stealing with per-worker deques.
pub struct Scheduler {
    /// Priority queue of pending tasks.
    queue: Arc<RwLock<BinaryHeap<PriorityTask>>>,
    /// Maximum queue capacity.
    max_queue_size: usize,
    /// Task results indexed by task ID.
    results: Arc<RwLock<HashMap<TaskId, ExecutionResult>>>,
}

impl Scheduler {
    /// Creates a new scheduler with default capacity.
    #[must_use]
    pub fn new() -> Self {
        Self::with_capacity(10_000)
    }

    /// Creates a scheduler with specified queue capacity.
    #[must_use]
    pub fn with_capacity(max_queue_size: usize) -> Self {
        info!("Scheduler initialized with capacity {max_queue_size}");
        Self {
            queue: Arc::new(RwLock::new(BinaryHeap::new())),
            max_queue_size,
            results: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Submits a task to the scheduler.
    ///
    /// # Errors
    ///
    /// Returns an error if the queue is full.
    pub async fn submit(&self, task: Task) -> Result<TaskId> {
        let task_id = task.id();
        let task_priority = task.priority();

        {
            let mut queue = self.queue.write().await;

            if queue.len() >= self.max_queue_size {
                return Err(RepartirError::QueueFull {
                    capacity: self.max_queue_size,
                });
            }

            debug!("Scheduling task {task_id} with priority {task_priority:?}");
            queue.push(PriorityTask { task });
        } // Drop queue lock early

        Ok(task_id)
    }

    /// Retrieves the next task from the queue.
    ///
    /// Returns `None` if the queue is empty.
    pub async fn next_task(&self) -> Option<Task> {
        let mut queue = self.queue.write().await;
        queue.pop().map(|pt| pt.task)
    }

    /// Returns the number of pending tasks.
    pub async fn pending_count(&self) -> usize {
        self.queue.read().await.len()
    }

    /// Stores a task result.
    pub async fn store_result(&self, result: ExecutionResult) {
        let task_id = result.task_id();
        debug!("Storing result for task {task_id}");
        self.results.write().await.insert(task_id, result);
    }

    /// Retrieves a task result.
    pub async fn get_result(&self, task_id: TaskId) -> Option<ExecutionResult> {
        self.results.read().await.get(&task_id).cloned()
    }

    /// Removes a task result from storage.
    pub async fn remove_result(&self, task_id: TaskId) -> Option<ExecutionResult> {
        self.results.write().await.remove(&task_id)
    }

    /// Clears all pending tasks and results.
    pub async fn clear(&self) {
        self.queue.write().await.clear();
        self.results.write().await.clear();
        info!("Scheduler cleared");
    }
}

impl Default for Scheduler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use crate::task::{Backend, Priority};
    use std::time::Duration;

    #[tokio::test]
    async fn test_scheduler_submit_and_next() {
        let scheduler = Scheduler::new();

        let task = Task::builder()
            .binary("/bin/echo")
            .arg("test")
            .backend(Backend::Cpu)
            .build()
            .unwrap();

        let task_id = scheduler.submit(task).await.unwrap();
        assert_eq!(scheduler.pending_count().await, 1);

        let next = scheduler.next_task().await;
        assert!(next.is_some());
        assert_eq!(next.unwrap().id(), task_id);
        assert_eq!(scheduler.pending_count().await, 0);
    }

    #[tokio::test]
    async fn test_scheduler_priority_ordering() {
        let scheduler = Scheduler::new();

        // Submit tasks in reverse priority order
        let low = Task::builder()
            .binary("/bin/echo")
            .arg("low")
            .backend(Backend::Cpu)
            .priority(Priority::Low)
            .build()
            .unwrap();

        let high = Task::builder()
            .binary("/bin/echo")
            .arg("high")
            .backend(Backend::Cpu)
            .priority(Priority::High)
            .build()
            .unwrap();

        let normal = Task::builder()
            .binary("/bin/echo")
            .arg("normal")
            .backend(Backend::Cpu)
            .priority(Priority::Normal)
            .build()
            .unwrap();

        scheduler.submit(low).await.unwrap();
        scheduler.submit(high).await.unwrap();
        scheduler.submit(normal).await.unwrap();

        // Should return in priority order: High, Normal, Low
        let first = scheduler.next_task().await.unwrap();
        assert_eq!(first.priority(), Priority::High);

        let second = scheduler.next_task().await.unwrap();
        assert_eq!(second.priority(), Priority::Normal);

        let third = scheduler.next_task().await.unwrap();
        assert_eq!(third.priority(), Priority::Low);
    }

    #[tokio::test]
    async fn test_scheduler_queue_full() {
        let scheduler = Scheduler::with_capacity(2);

        let task1 = Task::builder()
            .binary("/bin/echo")
            .arg("1")
            .backend(Backend::Cpu)
            .build()
            .unwrap();

        let task2 = Task::builder()
            .binary("/bin/echo")
            .arg("2")
            .backend(Backend::Cpu)
            .build()
            .unwrap();

        let task3 = Task::builder()
            .binary("/bin/echo")
            .arg("3")
            .backend(Backend::Cpu)
            .build()
            .unwrap();

        assert!(scheduler.submit(task1).await.is_ok());
        assert!(scheduler.submit(task2).await.is_ok());

        // Third task should fail (queue full)
        let result = scheduler.submit(task3).await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            RepartirError::QueueFull { .. }
        ));
    }

    #[tokio::test]
    async fn test_scheduler_result_storage() {
        let scheduler = Scheduler::new();

        let task_id = TaskId::new();
        let result = ExecutionResult::new(
            task_id,
            0,
            b"output".to_vec(),
            b"".to_vec(),
            Duration::from_secs(1),
        );

        scheduler.store_result(result.clone()).await;

        let retrieved = scheduler.get_result(task_id).await;
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().task_id(), task_id);

        let removed = scheduler.remove_result(task_id).await;
        assert!(removed.is_some());

        assert!(scheduler.get_result(task_id).await.is_none());
    }

    #[tokio::test]
    async fn test_scheduler_clear() {
        let scheduler = Scheduler::new();

        let task = Task::builder()
            .binary("/bin/echo")
            .arg("test")
            .backend(Backend::Cpu)
            .build()
            .unwrap();

        scheduler.submit(task).await.unwrap();
        assert_eq!(scheduler.pending_count().await, 1);

        scheduler.clear().await;
        assert_eq!(scheduler.pending_count().await, 0);
    }
}
