//! Checkpoint management for persistent task state.
//!
//! This module provides checkpointing capabilities for long-running tasks,
//! enabling restart from the last checkpoint on failure.
//!
//! # Example
//!
//! ```rust,no_run
//! use repartir::checkpoint::{CheckpointManager, TaskState};
//! use std::path::PathBuf;
//!
//! # async fn example() -> repartir::error::Result<()> {
//! let manager = CheckpointManager::new(PathBuf::from("./checkpoints"))?;
//!
//! // Create a checkpoint
//! let task_id = uuid::Uuid::new_v4();
//! let state = TaskState {
//!     task_id,
//!     iteration: 42,
//!     data: vec![1, 2, 3, 4],
//!     timestamp: std::time::SystemTime::now(),
//! };
//!
//! let checkpoint_id = manager.checkpoint(task_id, &state).await?;
//!
//! // Restore from checkpoint
//! let restored = manager.restore(task_id).await?;
//! assert_eq!(restored.unwrap().iteration, 42);
//! # Ok(())
//! # }
//! ```

use crate::error::{RepartirError, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::SystemTime;
use uuid::Uuid;

#[cfg(feature = "checkpoint")]
use trueno_db::storage::StorageEngine;

/// Unique identifier for a checkpoint
pub type CheckpointId = Uuid;

/// Task state that can be checkpointed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskState {
    /// Unique task identifier
    pub task_id: Uuid,
    /// Current iteration number
    pub iteration: u64,
    /// Serialized application state
    pub data: Vec<u8>,
    /// Timestamp when checkpoint was created
    pub timestamp: SystemTime,
}

/// Metadata about a checkpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckpointMetadata {
    /// Checkpoint identifier
    pub checkpoint_id: CheckpointId,
    /// Task identifier
    pub task_id: Uuid,
    /// Iteration number
    pub iteration: u64,
    /// Size of checkpoint data in bytes
    pub size_bytes: usize,
    /// Creation timestamp
    pub created_at: SystemTime,
}

/// Manages persistent checkpoints for tasks
pub struct CheckpointManager {
    checkpoint_dir: PathBuf,
    #[cfg(feature = "checkpoint")]
    _storage: Option<StorageEngine>,
}

impl CheckpointManager {
    /// Create a new checkpoint manager
    ///
    /// # Arguments
    ///
    /// * `checkpoint_dir` - Directory to store checkpoint files
    ///
    /// # Errors
    ///
    /// Returns error if directory cannot be created
    pub fn new(checkpoint_dir: PathBuf) -> Result<Self> {
        // Create checkpoint directory if it doesn't exist
        std::fs::create_dir_all(&checkpoint_dir).map_err(|e| {
            RepartirError::InvalidTask {
                reason: format!("Failed to create checkpoint directory: {}", e),
            }
        })?;

        Ok(Self {
            checkpoint_dir,
            #[cfg(feature = "checkpoint")]
            _storage: None,
        })
    }

    /// Create a checkpoint for a task
    ///
    /// # Arguments
    ///
    /// * `task_id` - Unique task identifier
    /// * `state` - Task state to checkpoint
    ///
    /// # Returns
    ///
    /// Returns the checkpoint ID on success
    ///
    /// # Errors
    ///
    /// Returns error if checkpoint cannot be written
    #[cfg(feature = "checkpoint")]
    pub async fn checkpoint(&self, task_id: Uuid, state: &TaskState) -> Result<CheckpointId> {
        let checkpoint_id = Uuid::new_v4();

        // Serialize state to JSON for now (v2.1 will use Parquet)
        let serialized = serde_json::to_vec(&state).map_err(|e| RepartirError::InvalidTask {
            reason: format!("Failed to serialize checkpoint: {}", e),
        })?;

        // Write to file: checkpoints/<task_id>/<checkpoint_id>.json
        let task_dir = self.checkpoint_dir.join(task_id.to_string());
        std::fs::create_dir_all(&task_dir).map_err(|e| RepartirError::InvalidTask {
            reason: format!("Failed to create task checkpoint directory: {}", e),
        })?;

        let checkpoint_path = task_dir.join(format!("{}.json", checkpoint_id));
        std::fs::write(&checkpoint_path, &serialized).map_err(|e| {
            RepartirError::InvalidTask {
                reason: format!("Failed to write checkpoint: {}", e),
            }
        })?;

        Ok(checkpoint_id)
    }

    /// Checkpoint without trueno-db feature (minimal implementation)
    #[cfg(not(feature = "checkpoint"))]
    pub async fn checkpoint(&self, task_id: Uuid, state: &TaskState) -> Result<CheckpointId> {
        let checkpoint_id = Uuid::new_v4();

        // Serialize state to JSON
        let serialized = serde_json::to_vec(&state).map_err(|e| RepartirError::InvalidTask {
            reason: format!("Failed to serialize checkpoint: {}", e),
        })?;

        // Write to file
        let task_dir = self.checkpoint_dir.join(task_id.to_string());
        std::fs::create_dir_all(&task_dir).map_err(|e| RepartirError::InvalidTask {
            reason: format!("Failed to create task checkpoint directory: {}", e),
        })?;

        let checkpoint_path = task_dir.join(format!("{}.json", checkpoint_id));
        std::fs::write(&checkpoint_path, &serialized).map_err(|e| {
            RepartirError::InvalidTask {
                reason: format!("Failed to write checkpoint: {}", e),
            }
        })?;

        Ok(checkpoint_id)
    }

    /// Restore task from last checkpoint
    ///
    /// # Arguments
    ///
    /// * `task_id` - Unique task identifier
    ///
    /// # Returns
    ///
    /// Returns the most recent task state, or None if no checkpoints exist
    ///
    /// # Errors
    ///
    /// Returns error if checkpoint cannot be read
    pub async fn restore(&self, task_id: Uuid) -> Result<Option<TaskState>> {
        let task_dir = self.checkpoint_dir.join(task_id.to_string());

        // Check if task has any checkpoints
        if !task_dir.exists() {
            return Ok(None);
        }

        // Find most recent checkpoint (lexicographically latest UUID)
        let mut checkpoints: Vec<_> = std::fs::read_dir(&task_dir)
            .map_err(|e| RepartirError::InvalidTask {
                reason: format!("Failed to read checkpoint directory: {}", e),
            })?
            .filter_map(|entry| entry.ok())
            .filter(|entry| {
                entry
                    .path()
                    .extension()
                    .map_or(false, |ext| ext == "json")
            })
            .collect();

        if checkpoints.is_empty() {
            return Ok(None);
        }

        // Sort by filename (checkpoint ID)
        checkpoints.sort_by_key(|e| e.path());

        // Read most recent checkpoint
        let latest = checkpoints
            .last()
            .ok_or_else(|| RepartirError::InvalidTask {
                reason: "No checkpoints found".to_string(),
            })?;

        let data = std::fs::read(latest.path()).map_err(|e| RepartirError::InvalidTask {
            reason: format!("Failed to read checkpoint: {}", e),
        })?;

        let state = serde_json::from_slice(&data).map_err(|e| RepartirError::InvalidTask {
            reason: format!("Failed to deserialize checkpoint: {}", e),
        })?;

        Ok(Some(state))
    }

    /// List all checkpoints for a task
    ///
    /// # Arguments
    ///
    /// * `task_id` - Unique task identifier
    ///
    /// # Returns
    ///
    /// Returns a list of checkpoint metadata, sorted by creation time
    ///
    /// # Errors
    ///
    /// Returns error if checkpoints cannot be listed
    pub async fn list_checkpoints(&self, task_id: Uuid) -> Result<Vec<CheckpointMetadata>> {
        let task_dir = self.checkpoint_dir.join(task_id.to_string());

        if !task_dir.exists() {
            return Ok(Vec::new());
        }

        let mut metadata = Vec::new();

        for entry in std::fs::read_dir(&task_dir).map_err(|e| RepartirError::InvalidTask {
            reason: format!("Failed to read checkpoint directory: {}", e),
        })? {
            let entry = entry.map_err(|e| RepartirError::InvalidTask {
                reason: format!("Failed to read directory entry: {}", e),
            })?;

            let path = entry.path();
            if path.extension().map_or(false, |ext| ext == "json") {
                // Read checkpoint to get metadata
                let data = std::fs::read(&path).map_err(|e| RepartirError::InvalidTask {
                    reason: format!("Failed to read checkpoint: {}", e),
                })?;

                let state: TaskState =
                    serde_json::from_slice(&data).map_err(|e| RepartirError::InvalidTask {
                        reason: format!("Failed to deserialize checkpoint: {}", e),
                    })?;

                let checkpoint_id = path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .and_then(|s| Uuid::parse_str(s).ok())
                    .ok_or_else(|| RepartirError::InvalidTask {
                        reason: "Invalid checkpoint filename".to_string(),
                    })?;

                metadata.push(CheckpointMetadata {
                    checkpoint_id,
                    task_id,
                    iteration: state.iteration,
                    size_bytes: data.len(),
                    created_at: state.timestamp,
                });
            }
        }

        // Sort by creation time
        metadata.sort_by_key(|m| m.created_at);

        Ok(metadata)
    }

    /// Delete old checkpoints based on retention policy
    ///
    /// # Arguments
    ///
    /// * `retention_days` - Keep checkpoints newer than this many days
    ///
    /// # Returns
    ///
    /// Returns the number of checkpoints deleted
    ///
    /// # Errors
    ///
    /// Returns error if checkpoints cannot be deleted
    pub async fn cleanup(&self, retention_days: u32) -> Result<usize> {
        let retention_duration = std::time::Duration::from_secs(retention_days as u64 * 24 * 3600);
        let cutoff_time = SystemTime::now()
            .checked_sub(retention_duration)
            .ok_or_else(|| RepartirError::InvalidTask {
                reason: "Invalid retention duration".to_string(),
            })?;

        let mut deleted_count = 0;

        // Iterate over all task directories
        for entry in std::fs::read_dir(&self.checkpoint_dir).map_err(|e| {
            RepartirError::InvalidTask {
                reason: format!("Failed to read checkpoint directory: {}", e),
            }
        })? {
            let entry = entry.map_err(|e| RepartirError::InvalidTask {
                reason: format!("Failed to read directory entry: {}", e),
            })?;

            if entry.path().is_dir() {
                // This is a task directory
                if let Ok(task_id) = Uuid::parse_str(&entry.file_name().to_string_lossy()) {
                    let checkpoints = self.list_checkpoints(task_id).await?;

                    for checkpoint in checkpoints {
                        if checkpoint.created_at < cutoff_time {
                            // Delete old checkpoint
                            let checkpoint_path = self
                                .checkpoint_dir
                                .join(task_id.to_string())
                                .join(format!("{}.json", checkpoint.checkpoint_id));

                            std::fs::remove_file(&checkpoint_path).map_err(|e| {
                                RepartirError::InvalidTask {
                                    reason: format!("Failed to delete checkpoint: {}", e),
                                }
                            })?;

                            deleted_count += 1;
                        }
                    }
                }
            }
        }

        Ok(deleted_count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[tokio::test]
    async fn test_checkpoint_and_restore() {
        let temp_dir = std::env::temp_dir().join(format!("repartir_test_{}", Uuid::new_v4()));
        let manager = CheckpointManager::new(temp_dir.clone()).unwrap();

        let task_id = Uuid::new_v4();
        let state = TaskState {
            task_id,
            iteration: 42,
            data: vec![1, 2, 3, 4, 5],
            timestamp: SystemTime::now(),
        };

        // Create checkpoint
        let checkpoint_id = manager.checkpoint(task_id, &state).await.unwrap();
        assert!(!checkpoint_id.is_nil());

        // Restore checkpoint
        let restored = manager.restore(task_id).await.unwrap();
        assert!(restored.is_some());

        let restored_state = restored.unwrap();
        assert_eq!(restored_state.task_id, task_id);
        assert_eq!(restored_state.iteration, 42);
        assert_eq!(restored_state.data, vec![1, 2, 3, 4, 5]);

        // Cleanup
        std::fs::remove_dir_all(&temp_dir).ok();
    }

    #[tokio::test]
    async fn test_restore_nonexistent() {
        let temp_dir = std::env::temp_dir().join(format!("repartir_test_{}", Uuid::new_v4()));
        let manager = CheckpointManager::new(temp_dir.clone()).unwrap();

        let task_id = Uuid::new_v4();
        let restored = manager.restore(task_id).await.unwrap();
        assert!(restored.is_none());

        // Cleanup
        std::fs::remove_dir_all(&temp_dir).ok();
    }

    #[tokio::test]
    async fn test_list_checkpoints() {
        let temp_dir = std::env::temp_dir().join(format!("repartir_test_{}", Uuid::new_v4()));
        let manager = CheckpointManager::new(temp_dir.clone()).unwrap();

        let task_id = Uuid::new_v4();

        // Create multiple checkpoints
        for i in 0..3 {
            let state = TaskState {
                task_id,
                iteration: i,
                data: vec![i as u8],
                timestamp: SystemTime::now(),
            };
            manager.checkpoint(task_id, &state).await.unwrap();
            tokio::time::sleep(Duration::from_millis(10)).await;
        }

        // List checkpoints
        let checkpoints = manager.list_checkpoints(task_id).await.unwrap();
        assert_eq!(checkpoints.len(), 3);

        // Verify sorted by creation time
        for (i, checkpoint) in checkpoints.iter().enumerate() {
            assert_eq!(checkpoint.iteration, i as u64);
        }

        // Cleanup
        std::fs::remove_dir_all(&temp_dir).ok();
    }

    #[tokio::test]
    async fn test_cleanup() {
        let temp_dir = std::env::temp_dir().join(format!("repartir_test_{}", Uuid::new_v4()));
        let manager = CheckpointManager::new(temp_dir.clone()).unwrap();

        let task_id = Uuid::new_v4();

        // Create checkpoint with old timestamp
        let old_state = TaskState {
            task_id,
            iteration: 1,
            data: vec![1],
            timestamp: SystemTime::now()
                .checked_sub(Duration::from_secs(10 * 24 * 3600))
                .unwrap(),
        };
        manager.checkpoint(task_id, &old_state).await.unwrap();

        // Create checkpoint with recent timestamp
        let new_state = TaskState {
            task_id,
            iteration: 2,
            data: vec![2],
            timestamp: SystemTime::now(),
        };
        manager.checkpoint(task_id, &new_state).await.unwrap();

        // Cleanup checkpoints older than 7 days
        let deleted = manager.cleanup(7).await.unwrap();
        assert_eq!(deleted, 1);

        // Verify only recent checkpoint remains
        let checkpoints = manager.list_checkpoints(task_id).await.unwrap();
        assert_eq!(checkpoints.len(), 1);
        assert_eq!(checkpoints[0].iteration, 2);

        // Cleanup
        std::fs::remove_dir_all(&temp_dir).ok();
    }
}
