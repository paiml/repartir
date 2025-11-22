//! Distributed state management for repartir v2.0.
//!
//! The StateStore provides key-value storage and data locality tracking
//! for distributed task execution. It enables:
//! - Configuration storage across workers
//! - Task result caching for data locality
//! - Metadata tracking for scheduler optimization
//!
//! # Example
//!
//! ```rust,no_run
//! use repartir::state::StateStore;
//! use std::path::PathBuf;
//!
//! # async fn example() -> repartir::error::Result<()> {
//! let store = StateStore::new(PathBuf::from("./state"))?;
//!
//! // Store configuration
//! store.put("model_version", b"v2.0").await?;
//!
//! // Retrieve configuration
//! let version = store.get("model_version").await?;
//! assert_eq!(version.unwrap(), b"v2.0");
//! # Ok(())
//! # }
//! ```

use crate::error::{RepartirError, Result};
use crate::task::{ExecutionResult, TaskId};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Worker identifier
pub type WorkerId = String;

/// Metadata about data location
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataLocation {
    /// Data key (tensor ID, checkpoint ID, file path)
    pub key: String,
    /// Workers that have this data cached
    pub workers: Vec<WorkerId>,
    /// Size in bytes
    pub size_bytes: usize,
    /// Last accessed timestamp
    pub last_accessed: std::time::SystemTime,
}

/// In-memory state store for single-node deployment.
///
/// Future versions will add distributed backend using trueno-db
/// when Phase 3 (distribution) is available.
pub struct StateStore {
    /// Base directory for persistent storage
    base_dir: PathBuf,
    /// In-memory key-value store
    kv_store: Arc<RwLock<HashMap<String, Vec<u8>>>>,
    /// Data locality tracking
    data_locations: Arc<RwLock<HashMap<String, DataLocation>>>,
    /// Task result cache
    result_cache: Arc<RwLock<HashMap<Uuid, ExecutionResult>>>,
}

impl StateStore {
    /// Create a new state store
    ///
    /// # Arguments
    ///
    /// * `base_dir` - Directory for persistent state storage
    ///
    /// # Errors
    ///
    /// Returns error if directory cannot be created
    pub fn new(base_dir: PathBuf) -> Result<Self> {
        // Create base directory if it doesn't exist
        std::fs::create_dir_all(&base_dir).map_err(|e| RepartirError::InvalidTask {
            reason: format!("Failed to create state directory: {}", e),
        })?;

        Ok(Self {
            base_dir,
            kv_store: Arc::new(RwLock::new(HashMap::new())),
            data_locations: Arc::new(RwLock::new(HashMap::new())),
            result_cache: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    /// Store a key-value pair
    ///
    /// # Arguments
    ///
    /// * `key` - Unique key
    /// * `value` - Value as bytes
    ///
    /// # Errors
    ///
    /// Returns error if storage fails
    pub async fn put(&self, key: &str, value: &[u8]) -> Result<()> {
        let mut store = self.kv_store.write().await;
        store.insert(key.to_string(), value.to_vec());

        // Persist to disk for durability (in background)
        let file_path = self.base_dir.join(format!("{}.kv", key));
        let value_clone = value.to_vec();
        tokio::task::spawn_blocking(move || std::fs::write(&file_path, value_clone))
            .await
            .map_err(|e| RepartirError::InvalidTask {
                reason: format!("Failed to spawn persist task: {}", e),
            })?
            .map_err(|e| RepartirError::InvalidTask {
                reason: format!("Failed to persist key-value: {}", e),
            })?;

        Ok(())
    }

    /// Retrieve a value by key
    ///
    /// # Arguments
    ///
    /// * `key` - Unique key
    ///
    /// # Returns
    ///
    /// Returns the value if found, None otherwise
    ///
    /// # Errors
    ///
    /// Returns error if read fails
    pub async fn get(&self, key: &str) -> Result<Option<Vec<u8>>> {
        let store = self.kv_store.read().await;

        if let Some(value) = store.get(key) {
            return Ok(Some(value.clone()));
        }

        // Try to load from disk if not in memory
        let file_path = self.base_dir.join(format!("{}.kv", key));
        if file_path.exists() {
            let value = tokio::task::spawn_blocking(move || std::fs::read(&file_path))
                .await
                .map_err(|e| RepartirError::InvalidTask {
                    reason: format!("Failed to spawn read task: {}", e),
                })?
                .map_err(|e| RepartirError::InvalidTask {
                    reason: format!("Failed to read key-value from disk: {}", e),
                })?;

            // Cache in memory
            drop(store);
            let mut store_mut = self.kv_store.write().await;
            store_mut.insert(key.to_string(), value.clone());

            Ok(Some(value))
        } else {
            Ok(None)
        }
    }

    /// Cache a task result for data locality
    ///
    /// # Arguments
    ///
    /// * `task_id` - Task identifier
    /// * `result` - Execution result
    ///
    /// # Errors
    ///
    /// Returns error if caching fails
    pub async fn cache_result(&self, task_id: TaskId, result: &ExecutionResult) -> Result<()> {
        let mut cache = self.result_cache.write().await;
        cache.insert(*task_id.as_uuid(), result.clone());

        // Track data location for this result
        let data_key = format!("result_{}", task_id);
        let location = DataLocation {
            key: data_key.clone(),
            workers: vec!["local".to_string()], // Single-node for now
            size_bytes: result.stdout().len() + result.stderr().len(),
            last_accessed: std::time::SystemTime::now(),
        };

        let mut locations = self.data_locations.write().await;
        locations.insert(data_key, location);

        Ok(())
    }

    /// Get cached task result
    ///
    /// # Arguments
    ///
    /// * `task_id` - Task identifier
    ///
    /// # Returns
    ///
    /// Returns the cached result if available
    pub async fn get_result(&self, task_id: TaskId) -> Option<ExecutionResult> {
        let cache = self.result_cache.read().await;
        cache.get(task_id.as_uuid()).cloned()
    }

    /// Query which workers have cached data
    ///
    /// # Arguments
    ///
    /// * `data_key` - Data identifier
    ///
    /// # Returns
    ///
    /// Returns list of worker IDs that have this data
    pub async fn locate_data(&self, data_key: &str) -> Result<Vec<WorkerId>> {
        let locations = self.data_locations.read().await;

        if let Some(location) = locations.get(data_key) {
            Ok(location.workers.clone())
        } else {
            Ok(Vec::new())
        }
    }

    /// Register data location for a worker
    ///
    /// # Arguments
    ///
    /// * `data_key` - Data identifier
    /// * `worker_id` - Worker that has the data
    /// * `size_bytes` - Size of the data
    ///
    /// # Errors
    ///
    /// Returns error if registration fails
    pub async fn register_data(
        &self,
        data_key: String,
        worker_id: WorkerId,
        size_bytes: usize,
    ) -> Result<()> {
        let mut locations = self.data_locations.write().await;

        locations
            .entry(data_key.clone())
            .and_modify(|loc| {
                if !loc.workers.contains(&worker_id) {
                    loc.workers.push(worker_id.clone());
                }
                loc.last_accessed = std::time::SystemTime::now();
            })
            .or_insert_with(|| DataLocation {
                key: data_key,
                workers: vec![worker_id],
                size_bytes,
                last_accessed: std::time::SystemTime::now(),
            });

        Ok(())
    }

    /// Query locations for multiple data keys (batch operation)
    ///
    /// # Arguments
    ///
    /// * `data_keys` - List of data identifiers
    ///
    /// # Returns
    ///
    /// Returns a map of worker IDs to count of data items they have
    pub async fn locate_data_batch(
        &self,
        data_keys: &[String],
    ) -> Result<HashMap<WorkerId, usize>> {
        let locations = self.data_locations.read().await;
        let mut worker_counts: HashMap<WorkerId, usize> = HashMap::new();

        for key in data_keys {
            if let Some(location) = locations.get(key) {
                for worker in &location.workers {
                    *worker_counts.entry(worker.clone()).or_insert(0) += 1;
                }
            }
        }

        Ok(worker_counts)
    }

    /// List all data keys
    ///
    /// # Returns
    ///
    /// Returns a list of all data keys in the store
    pub async fn list_data(&self) -> Vec<String> {
        let locations = self.data_locations.read().await;
        locations.keys().cloned().collect()
    }

    /// Remove data location entry
    ///
    /// # Arguments
    ///
    /// * `data_key` - Data identifier to remove
    pub async fn remove_data(&self, data_key: &str) -> Result<()> {
        let mut locations = self.data_locations.write().await;
        locations.remove(data_key);
        Ok(())
    }

    /// Get statistics about the state store
    ///
    /// # Returns
    ///
    /// Returns statistics tuple: (kv_entries, data_locations, cached_results)
    pub async fn stats(&self) -> (usize, usize, usize) {
        let kv = self.kv_store.read().await;
        let locations = self.data_locations.read().await;
        let cache = self.result_cache.read().await;

        (kv.len(), locations.len(), cache.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::task::{Backend, Task};
    use std::time::Duration;

    #[tokio::test]
    async fn test_state_store_kv() {
        let temp_dir = std::env::temp_dir().join(format!("state_test_{}", Uuid::new_v4()));
        let store = StateStore::new(temp_dir.clone()).unwrap();

        // Put and get
        store.put("test_key", b"test_value").await.unwrap();
        let value = store.get("test_key").await.unwrap();
        assert_eq!(value.unwrap(), b"test_value");

        // Non-existent key
        let missing = store.get("missing_key").await.unwrap();
        assert!(missing.is_none());

        // Cleanup
        std::fs::remove_dir_all(&temp_dir).ok();
    }

    #[tokio::test]
    async fn test_result_caching() {
        let temp_dir = std::env::temp_dir().join(format!("state_test_{}", Uuid::new_v4()));
        let store = StateStore::new(temp_dir.clone()).unwrap();

        // Create a task and result
        let task = Task::builder()
            .binary("/bin/echo")
            .backend(Backend::Cpu)
            .build()
            .unwrap();

        let result = crate::task::ExecutionResult::new(
            task.id(),
            0,
            b"output".to_vec(),
            Vec::new(),
            Duration::from_secs(1),
        );

        // Cache result
        store.cache_result(task.id(), &result).await.unwrap();

        // Retrieve cached result
        let cached = store.get_result(task.id()).await;
        assert!(cached.is_some());
        assert_eq!(cached.unwrap().stdout(), b"output");

        // Cleanup
        std::fs::remove_dir_all(&temp_dir).ok();
    }

    #[tokio::test]
    async fn test_data_location_tracking() {
        let temp_dir = std::env::temp_dir().join(format!("state_test_{}", Uuid::new_v4()));
        let store = StateStore::new(temp_dir.clone()).unwrap();

        // Register data on worker1
        store
            .register_data("tensor_42".to_string(), "worker1".to_string(), 1024)
            .await
            .unwrap();

        // Locate data
        let workers = store.locate_data("tensor_42").await.unwrap();
        assert_eq!(workers.len(), 1);
        assert_eq!(workers[0], "worker1");

        // Register same data on worker2
        store
            .register_data("tensor_42".to_string(), "worker2".to_string(), 1024)
            .await
            .unwrap();

        // Now two workers have it
        let workers = store.locate_data("tensor_42").await.unwrap();
        assert_eq!(workers.len(), 2);

        // Cleanup
        std::fs::remove_dir_all(&temp_dir).ok();
    }

    #[tokio::test]
    async fn test_batch_location_query() {
        let temp_dir = std::env::temp_dir().join(format!("state_test_{}", Uuid::new_v4()));
        let store = StateStore::new(temp_dir.clone()).unwrap();

        // Setup: worker1 has tensor_1 and tensor_2, worker2 has tensor_2 and tensor_3
        store
            .register_data("tensor_1".to_string(), "worker1".to_string(), 1024)
            .await
            .unwrap();
        store
            .register_data("tensor_2".to_string(), "worker1".to_string(), 1024)
            .await
            .unwrap();
        store
            .register_data("tensor_2".to_string(), "worker2".to_string(), 1024)
            .await
            .unwrap();
        store
            .register_data("tensor_3".to_string(), "worker2".to_string(), 1024)
            .await
            .unwrap();

        // Query for tensor_1, tensor_2, tensor_3
        let counts = store
            .locate_data_batch(&[
                "tensor_1".to_string(),
                "tensor_2".to_string(),
                "tensor_3".to_string(),
            ])
            .await
            .unwrap();

        // worker1 has 2 items, worker2 has 2 items
        assert_eq!(counts.get("worker1"), Some(&2));
        assert_eq!(counts.get("worker2"), Some(&2));

        // Cleanup
        std::fs::remove_dir_all(&temp_dir).ok();
    }

    #[tokio::test]
    async fn test_list_and_remove_data() {
        let temp_dir = std::env::temp_dir().join(format!("state_test_{}", Uuid::new_v4()));
        let store = StateStore::new(temp_dir.clone()).unwrap();

        // Register some data
        store
            .register_data("data1".to_string(), "worker1".to_string(), 100)
            .await
            .unwrap();
        store
            .register_data("data2".to_string(), "worker1".to_string(), 200)
            .await
            .unwrap();

        // List data
        let keys = store.list_data().await;
        assert_eq!(keys.len(), 2);
        assert!(keys.contains(&"data1".to_string()));
        assert!(keys.contains(&"data2".to_string()));

        // Remove data
        store.remove_data("data1").await.unwrap();

        // List again
        let keys = store.list_data().await;
        assert_eq!(keys.len(), 1);
        assert!(keys.contains(&"data2".to_string()));

        // Cleanup
        std::fs::remove_dir_all(&temp_dir).ok();
    }

    #[tokio::test]
    async fn test_stats() {
        let temp_dir = std::env::temp_dir().join(format!("state_test_{}", Uuid::new_v4()));
        let store = StateStore::new(temp_dir.clone()).unwrap();

        // Add some data
        store.put("key1", b"value1").await.unwrap();
        store.put("key2", b"value2").await.unwrap();

        store
            .register_data("data1".to_string(), "worker1".to_string(), 100)
            .await
            .unwrap();

        let task = Task::builder()
            .binary("/bin/echo")
            .backend(Backend::Cpu)
            .build()
            .unwrap();
        let result = crate::task::ExecutionResult::new(
            task.id(),
            0,
            b"output".to_vec(),
            Vec::new(),
            Duration::from_secs(1),
        );
        store.cache_result(task.id(), &result).await.unwrap();

        // Check stats
        let (kv_count, location_count, result_count) = store.stats().await;
        assert_eq!(kv_count, 2);
        assert_eq!(location_count, 2); // data1 + result location
        assert_eq!(result_count, 1);

        // Cleanup
        std::fs::remove_dir_all(&temp_dir).ok();
    }
}
