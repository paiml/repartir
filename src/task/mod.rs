//! Task definition and execution results.
//!
//! Core abstractions for distributed work in repartir.

use crate::error::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;
use uuid::Uuid;

/// Unique identifier for a task.
///
/// Per Iron Lotus case study (Section 12.3), we use UUIDs instead of
/// indices to prevent invalidation when workers disconnect.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TaskId(Uuid);

impl TaskId {
    /// Creates a new random task ID.
    #[must_use]
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Returns the inner UUID.
    #[must_use]
    pub const fn as_uuid(&self) -> &Uuid {
        &self.0
    }
}

impl Default for TaskId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for TaskId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Execution backend selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Backend {
    /// Execute on local CPU.
    Cpu,
    // NOTE: GPU and Remote backends will be added in v1.1+
    // /// Execute on GPU (requires `gpu` feature).
    // #[cfg(feature = "gpu")]
    // Gpu,
    /// Execute on remote worker (requires `remote` feature).
    #[cfg(feature = "remote")]
    Remote,
}

/// Priority level for task scheduling.
///
/// Higher priority tasks are scheduled before lower priority tasks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Default)]
pub enum Priority {
    /// Low priority (background tasks).
    Low = 0,
    /// Normal priority (default).
    #[default]
    Normal = 1,
    /// High priority (latency-sensitive).
    High = 2,
}

/// A task to be executed.
///
/// Tasks are the fundamental unit of work in repartir. Each task
/// represents a Rust binary to execute with specific arguments and environment.
///
/// # Example
///
/// ```
/// use repartir::task::{Task, Backend};
///
/// let task = Task::builder()
///     .binary("./target/release/worker")
///     .args(vec!["--input".to_string(), "data.bin".to_string()])
///     .backend(Backend::Cpu)
///     .build();
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    /// Unique task identifier.
    id: TaskId,
    /// Path to the binary to execute.
    binary: PathBuf,
    /// Command-line arguments.
    args: Vec<String>,
    /// Environment variables.
    env: HashMap<String, String>,
    /// Execution backend.
    backend: Backend,
    /// Priority level.
    priority: Priority,
    /// Optional timeout.
    timeout: Option<Duration>,
}

impl Task {
    /// Creates a new task builder.
    #[must_use]
    pub fn builder() -> TaskBuilder {
        TaskBuilder::default()
    }

    /// Returns the task ID.
    #[must_use]
    pub const fn id(&self) -> TaskId {
        self.id
    }

    /// Returns the binary path.
    #[must_use]
    pub const fn binary(&self) -> &PathBuf {
        &self.binary
    }

    /// Returns the arguments.
    #[must_use]
    pub fn args(&self) -> &[String] {
        &self.args
    }

    /// Returns the environment variables.
    #[must_use]
    pub const fn env(&self) -> &HashMap<String, String> {
        &self.env
    }

    /// Returns the backend.
    #[must_use]
    pub const fn backend(&self) -> Backend {
        self.backend
    }

    /// Returns the priority.
    #[must_use]
    pub const fn priority(&self) -> Priority {
        self.priority
    }

    /// Returns the timeout.
    #[must_use]
    pub const fn timeout(&self) -> Option<Duration> {
        self.timeout
    }
}

/// Builder for `Task`.
#[derive(Debug)]
pub struct TaskBuilder {
    binary: Option<PathBuf>,
    args: Vec<String>,
    env: HashMap<String, String>,
    backend: Backend,
    priority: Priority,
    timeout: Option<Duration>,
}

impl Default for TaskBuilder {
    fn default() -> Self {
        Self {
            binary: None,
            args: Vec::new(),
            env: HashMap::new(),
            backend: Backend::Cpu,
            priority: Priority::default(),
            timeout: None,
        }
    }
}

impl TaskBuilder {
    /// Sets the binary path.
    #[must_use]
    pub fn binary<P: Into<PathBuf>>(mut self, path: P) -> Self {
        self.binary = Some(path.into());
        self
    }

    /// Sets the arguments.
    #[must_use]
    pub fn args(mut self, args: Vec<String>) -> Self {
        self.args = args;
        self
    }

    /// Adds a single argument.
    #[must_use]
    pub fn arg<S: Into<String>>(mut self, arg: S) -> Self {
        self.args.push(arg.into());
        self
    }

    /// Sets the environment variables.
    #[must_use]
    pub fn env(mut self, env: HashMap<String, String>) -> Self {
        self.env = env;
        self
    }

    /// Adds a single environment variable.
    #[must_use]
    pub fn env_var<K: Into<String>, V: Into<String>>(mut self, key: K, value: V) -> Self {
        self.env.insert(key.into(), value.into());
        self
    }

    /// Sets the backend.
    #[must_use]
    pub const fn backend(mut self, backend: Backend) -> Self {
        self.backend = backend;
        self
    }

    /// Sets the priority.
    #[must_use]
    pub const fn priority(mut self, priority: Priority) -> Self {
        self.priority = priority;
        self
    }

    /// Sets the timeout.
    #[must_use]
    pub const fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    /// Builds the task.
    ///
    /// # Errors
    ///
    /// Returns an error if the binary path is not set.
    pub fn build(self) -> Result<Task> {
        let binary = self
            .binary
            .ok_or_else(|| crate::error::RepartirError::InvalidTask {
                reason: "Binary path not set".to_string(),
            })?;

        Ok(Task {
            id: TaskId::new(),
            binary,
            args: self.args,
            env: self.env,
            backend: self.backend,
            priority: self.priority,
            timeout: self.timeout,
        })
    }
}

/// Result of task execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    /// Task ID that was executed.
    task_id: TaskId,
    /// Exit code from the process.
    exit_code: i32,
    /// Standard output.
    stdout: Vec<u8>,
    /// Standard error.
    stderr: Vec<u8>,
    /// Execution duration.
    duration: Duration,
}

impl ExecutionResult {
    /// Creates a new execution result.
    #[must_use]
    pub const fn new(
        task_id: TaskId,
        exit_code: i32,
        stdout: Vec<u8>,
        stderr: Vec<u8>,
        duration: Duration,
    ) -> Self {
        Self {
            task_id,
            exit_code,
            stdout,
            stderr,
            duration,
        }
    }

    /// Returns the task ID.
    #[must_use]
    pub const fn task_id(&self) -> TaskId {
        self.task_id
    }

    /// Returns the exit code.
    #[must_use]
    pub const fn exit_code(&self) -> i32 {
        self.exit_code
    }

    /// Returns whether the task succeeded (exit code 0).
    #[must_use]
    pub const fn is_success(&self) -> bool {
        self.exit_code == 0
    }

    /// Returns the standard output.
    #[must_use]
    pub fn stdout(&self) -> &[u8] {
        &self.stdout
    }

    /// Returns the standard error.
    #[must_use]
    pub fn stderr(&self) -> &[u8] {
        &self.stderr
    }

    /// Returns the execution duration.
    #[must_use]
    pub const fn duration(&self) -> Duration {
        self.duration
    }

    /// Converts stdout to a UTF-8 string.
    ///
    /// # Errors
    ///
    /// Returns an error if stdout is not valid UTF-8.
    pub fn stdout_str(&self) -> Result<&str> {
        std::str::from_utf8(&self.stdout).map_err(|_| crate::error::RepartirError::InvalidTask {
            reason: "Stdout is not valid UTF-8".to_string(),
        })
    }

    /// Converts stderr to a UTF-8 string.
    ///
    /// # Errors
    ///
    /// Returns an error if stderr is not valid UTF-8.
    pub fn stderr_str(&self) -> Result<&str> {
        std::str::from_utf8(&self.stderr).map_err(|_| crate::error::RepartirError::InvalidTask {
            reason: "Stderr is not valid UTF-8".to_string(),
        })
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn test_task_id_creation() {
        let id1 = TaskId::new();
        let id2 = TaskId::new();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_task_builder() {
        let result = Task::builder()
            .binary("/bin/echo")
            .arg("hello")
            .arg("world")
            .backend(Backend::Cpu)
            .priority(Priority::High)
            .build();

        assert!(result.is_ok());
        let task = result.unwrap();
        assert_eq!(task.args(), &["hello", "world"]);
        assert_eq!(task.backend(), Backend::Cpu);
        assert_eq!(task.priority(), Priority::High);
    }

    #[test]
    fn test_task_builder_missing_binary() {
        let result = Task::builder().arg("test").build();
        assert!(result.is_err());
    }

    #[test]
    fn test_priority_ordering() {
        assert!(Priority::High > Priority::Normal);
        assert!(Priority::Normal > Priority::Low);
    }

    #[test]
    fn test_execution_result() {
        let task_id = TaskId::new();
        let result = ExecutionResult::new(
            task_id,
            0,
            b"output".to_vec(),
            b"".to_vec(),
            Duration::from_secs(1),
        );

        assert!(result.is_success());
        assert_eq!(result.exit_code(), 0);
        assert_eq!(result.stdout_str().unwrap(), "output");
    }
}
