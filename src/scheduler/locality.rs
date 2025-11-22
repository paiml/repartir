//! Locality-aware scheduling for data-intensive distributed computing.
//!
//! This module provides scheduling strategies that consider data locality
//! to minimize network transfers and improve performance.
//!
//! # Example
//!
//! ```rust,no_run
//! use repartir::scheduler::locality::LocalityScheduler;
//! use repartir::state::StateStore;
//! use repartir::task::{Task, Backend};
//! use std::path::PathBuf;
//!
//! # async fn example() -> repartir::error::Result<()> {
//! let state = StateStore::new(PathBuf::from("./state"))?;
//! let scheduler = LocalityScheduler::new(state);
//!
//! // Task with data dependencies
//! let task = Task::builder()
//!     .binary("/bin/worker")
//!     .backend(Backend::Cpu)
//!     .data_dependency("tensor_batch_42")
//!     .build()?;
//!
//! // Submit with automatic locality optimization
//! let task_id = scheduler.submit(task).await?;
//! # Ok(())
//! # }
//! ```

use crate::error::Result;
use crate::scheduler::Scheduler;
use crate::state::{StateStore, WorkerId};
use crate::task::{Task, TaskId};
use std::collections::HashMap;
use std::sync::Arc;

/// Worker affinity score (0.0 = no affinity, 1.0 = perfect affinity)
pub type AffinityScore = f64;

/// Locality-aware scheduler that optimizes task placement based on data location.
pub struct LocalityScheduler {
    /// Base scheduler for task management
    scheduler: Arc<Scheduler>,
    /// State store for data locality tracking
    state: Arc<StateStore>,
}

impl LocalityScheduler {
    /// Create a new locality-aware scheduler
    ///
    /// # Arguments
    ///
    /// * `state` - State store for tracking data locations
    pub fn new(state: StateStore) -> Self {
        Self {
            scheduler: Arc::new(Scheduler::new()),
            state: Arc::new(state),
        }
    }

    /// Create with custom base scheduler
    pub fn with_scheduler(scheduler: Scheduler, state: StateStore) -> Self {
        Self {
            scheduler: Arc::new(scheduler),
            state: Arc::new(state),
        }
    }

    /// Submit a task with automatic locality optimization
    ///
    /// If the task has data dependencies, the scheduler will calculate
    /// worker affinity scores and prefer workers with cached data.
    ///
    /// # Arguments
    ///
    /// * `task` - Task to submit
    ///
    /// # Returns
    ///
    /// Returns the task ID
    ///
    /// # Errors
    ///
    /// Returns error if submission fails
    pub async fn submit(&self, task: Task) -> Result<TaskId> {
        let data_deps = task.data_dependencies();

        if data_deps.is_empty() {
            // No locality hints, use standard scheduling
            self.scheduler.submit(task).await
        } else {
            // Calculate worker affinity based on data locality
            let affinity = self.calculate_affinity(data_deps).await?;

            // For now, just submit to base scheduler
            // In v2.1, this will integrate with worker selection
            let task_id = self.scheduler.submit(task).await?;

            // Log affinity for future optimization
            if !affinity.is_empty() {
                tracing::debug!(
                    "Task {} submitted with locality affinity: {:?}",
                    task_id,
                    affinity
                );
            }

            Ok(task_id)
        }
    }

    /// Calculate worker affinity scores based on data dependencies
    ///
    /// # Arguments
    ///
    /// * `data_deps` - List of data dependencies
    ///
    /// # Returns
    ///
    /// Returns a map of worker IDs to affinity scores
    ///
    /// # Errors
    ///
    /// Returns error if data location query fails
    pub async fn calculate_affinity(
        &self,
        data_deps: &[String],
    ) -> Result<HashMap<WorkerId, AffinityScore>> {
        if data_deps.is_empty() {
            return Ok(HashMap::new());
        }

        // Query which workers have which data items
        let worker_counts = self.state.locate_data_batch(data_deps).await?;

        // Calculate affinity score: items_present / total_items
        let total_items = data_deps.len() as f64;
        let affinity: HashMap<WorkerId, AffinityScore> = worker_counts
            .into_iter()
            .map(|(worker, count)| (worker, count as f64 / total_items))
            .collect();

        Ok(affinity)
    }

    /// Get the next task from the scheduler
    ///
    /// In v2.1, this will be enhanced to return the best task
    /// for a specific worker based on locality.
    pub async fn next_task(&self) -> Option<Task> {
        self.scheduler.next_task().await
    }

    /// Get the next task optimized for a specific worker
    ///
    /// This will scan the queue and prefer tasks that have
    /// data dependencies cached on the specified worker.
    ///
    /// # Arguments
    ///
    /// * `worker_id` - Worker identifier
    ///
    /// # Returns
    ///
    /// Returns the best task for this worker, or None if queue is empty
    pub async fn next_task_for_worker(&self, worker_id: &WorkerId) -> Option<Task> {
        // v2.1: Implement priority queue scan with locality scoring
        // For now, just use standard next_task
        let _ = worker_id; // Suppress unused warning
        self.scheduler.next_task().await
    }

    /// Store task result and update locality tracking
    ///
    /// # Arguments
    ///
    /// * `result` - Execution result to store
    pub async fn store_result(&self, result: crate::task::ExecutionResult) -> Result<()> {
        let task_id = result.task_id();

        // Store in base scheduler
        self.scheduler.store_result(result.clone()).await;

        // Cache in state store for locality tracking
        self.state.cache_result(task_id, &result).await?;

        Ok(())
    }

    /// Get a cached result
    pub async fn get_result(&self, task_id: TaskId) -> Option<crate::task::ExecutionResult> {
        // Try state store cache first (faster)
        if let Some(result) = self.state.get_result(task_id).await {
            return Some(result);
        }

        // Fallback to scheduler
        self.scheduler.get_result(task_id).await
    }

    /// Get locality statistics
    ///
    /// Returns (total_tasks, tasks_with_deps, avg_affinity)
    pub async fn locality_stats(&self) -> (usize, usize, f64) {
        // v2.1: Track actual locality hit rates
        // For now, return placeholder stats
        (0, 0, 0.0)
    }

    /// Access the underlying scheduler
    pub fn scheduler(&self) -> &Scheduler {
        &self.scheduler
    }

    /// Access the state store
    pub fn state(&self) -> &StateStore {
        &self.state
    }
}

/// Calculate expected network transfer savings from locality
///
/// # Arguments
///
/// * `affinity` - Worker affinity scores
/// * `data_sizes` - Size of each data dependency in bytes
///
/// # Returns
///
/// Returns estimated bytes saved by using locality-aware scheduling
pub fn estimate_transfer_savings(
    affinity: &HashMap<WorkerId, AffinityScore>,
    data_sizes: &[usize],
) -> usize {
    if affinity.is_empty() {
        return 0;
    }

    // Best worker has highest affinity
    let best_affinity = affinity.values().max_by(|a, b| {
        a.partial_cmp(b)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    if let Some(&score) = best_affinity {
        // Estimate: score * total_data_size = bytes already on worker
        let total_size: usize = data_sizes.iter().sum();
        (score * total_size as f64) as usize
    } else {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::task::Backend;
    use std::path::PathBuf;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_locality_scheduler_creation() {
        let temp_dir = std::env::temp_dir().join(format!("locality_test_{}", Uuid::new_v4()));
        let state = StateStore::new(temp_dir.clone()).unwrap();
        let scheduler = LocalityScheduler::new(state);

        // Verify scheduler is initialized
        assert_eq!(scheduler.locality_stats().await.0, 0);

        std::fs::remove_dir_all(&temp_dir).ok();
    }

    #[tokio::test]
    async fn test_submit_task_without_dependencies() {
        let temp_dir = std::env::temp_dir().join(format!("locality_test_{}", Uuid::new_v4()));
        let state = StateStore::new(temp_dir.clone()).unwrap();
        let scheduler = LocalityScheduler::new(state);

        let task = Task::builder()
            .binary("/bin/echo")
            .backend(Backend::Cpu)
            .build()
            .unwrap();

        let task_id = scheduler.submit(task).await.unwrap();
        assert!(!task_id.as_uuid().is_nil());

        std::fs::remove_dir_all(&temp_dir).ok();
    }

    #[tokio::test]
    async fn test_submit_task_with_dependencies() {
        let temp_dir = std::env::temp_dir().join(format!("locality_test_{}", Uuid::new_v4()));
        let state = StateStore::new(temp_dir.clone()).unwrap();

        // Register some data on worker1
        state
            .register_data("tensor_42".to_string(), "worker1".to_string(), 1024)
            .await
            .unwrap();

        let scheduler = LocalityScheduler::new(state);

        let task = Task::builder()
            .binary("/bin/echo")
            .backend(Backend::Cpu)
            .data_dependency("tensor_42")
            .build()
            .unwrap();

        let task_id = scheduler.submit(task).await.unwrap();
        assert!(!task_id.as_uuid().is_nil());

        std::fs::remove_dir_all(&temp_dir).ok();
    }

    #[tokio::test]
    async fn test_calculate_affinity() {
        let temp_dir = std::env::temp_dir().join(format!("locality_test_{}", Uuid::new_v4()));
        let state = StateStore::new(temp_dir.clone()).unwrap();

        // Setup: worker1 has tensor_1 and tensor_2, worker2 has tensor_2 only
        state
            .register_data("tensor_1".to_string(), "worker1".to_string(), 1024)
            .await
            .unwrap();
        state
            .register_data("tensor_2".to_string(), "worker1".to_string(), 1024)
            .await
            .unwrap();
        state
            .register_data("tensor_2".to_string(), "worker2".to_string(), 1024)
            .await
            .unwrap();

        let scheduler = LocalityScheduler::new(state);

        // Calculate affinity for tensor_1 and tensor_2
        let affinity = scheduler
            .calculate_affinity(&["tensor_1".to_string(), "tensor_2".to_string()])
            .await
            .unwrap();

        // worker1 has 2/2 = 100% affinity
        assert_eq!(affinity.get("worker1"), Some(&1.0));
        // worker2 has 1/2 = 50% affinity
        assert_eq!(affinity.get("worker2"), Some(&0.5));

        std::fs::remove_dir_all(&temp_dir).ok();
    }

    #[tokio::test]
    async fn test_affinity_no_data() {
        let temp_dir = std::env::temp_dir().join(format!("locality_test_{}", Uuid::new_v4()));
        let state = StateStore::new(temp_dir.clone()).unwrap();
        let scheduler = LocalityScheduler::new(state);

        // No data registered, query for non-existent data
        let affinity = scheduler
            .calculate_affinity(&["missing_data".to_string()])
            .await
            .unwrap();

        assert!(affinity.is_empty());

        std::fs::remove_dir_all(&temp_dir).ok();
    }

    #[tokio::test]
    async fn test_estimate_transfer_savings() {
        let mut affinity = HashMap::new();
        affinity.insert("worker1".to_string(), 0.8); // 80% of data on worker1
        affinity.insert("worker2".to_string(), 0.3); // 30% of data on worker2

        let data_sizes = vec![1000, 2000, 3000]; // Total: 6000 bytes

        let savings = estimate_transfer_savings(&affinity, &data_sizes);

        // Best worker (worker1) has 80% affinity
        // Savings = 0.8 * 6000 = 4800 bytes
        assert_eq!(savings, 4800);
    }

    #[tokio::test]
    async fn test_next_task() {
        let temp_dir = std::env::temp_dir().join(format!("locality_test_{}", Uuid::new_v4()));
        let state = StateStore::new(temp_dir.clone()).unwrap();
        let scheduler = LocalityScheduler::new(state);

        // Submit a task
        let task = Task::builder()
            .binary("/bin/echo")
            .backend(Backend::Cpu)
            .build()
            .unwrap();

        scheduler.submit(task).await.unwrap();

        // Get next task
        let next = scheduler.next_task().await;
        assert!(next.is_some());

        // Queue should be empty now
        let empty = scheduler.next_task().await;
        assert!(empty.is_none());

        std::fs::remove_dir_all(&temp_dir).ok();
    }
}
