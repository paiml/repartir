//! Tensor operations using trueno SIMD library (v2.0).
//!
//! Provides high-performance tensor computation for ML workloads using
//! the trueno library's SIMD and GPU acceleration capabilities.
//!
//! # Example
//!
//! ```rust,no_run
//! # #[cfg(feature = "tensor")]
//! # {
//! use repartir::tensor::TensorExecutor;
//!
//! # async fn example() -> repartir::error::Result<()> {
//! let executor = TensorExecutor::new()?;
//!
//! // Matrix multiplication
//! let a = vec![1.0, 2.0, 3.0, 4.0]; // 2x2 matrix
//! let b = vec![5.0, 6.0, 7.0, 8.0]; // 2x2 matrix
//! let result = executor.matmul(&a, &b, 2, 2, 2).await?;
//!
//! // Element-wise operations
//! let scaled = executor.scale(&a, 2.0).await?;
//! # Ok(())
//! # }
//! # }
//! ```

#![cfg(feature = "tensor")]

use crate::error::{RepartirError, Result};
use std::sync::Arc;
use tokio::sync::RwLock;

#[cfg(feature = "tensor")]
use trueno::{Matrix, Vector};

/// Tensor executor for SIMD-accelerated operations.
///
/// Uses trueno library for high-performance numerical computation
/// with automatic SIMD vectorization and optional GPU acceleration.
pub struct TensorExecutor {
    /// Execution backend preference
    backend: TensorBackend,
    /// Operation cache for repeated computations
    cache: Arc<RwLock<OperationCache>>,
}

/// Backend for tensor operations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TensorBackend {
    /// CPU with SIMD acceleration
    Cpu,
    /// GPU acceleration (requires gpu feature)
    #[cfg(feature = "gpu")]
    Gpu,
}

/// Cache for tensor operation results
struct OperationCache {
    /// Cached matrix multiplications: (key, result)
    matmul_cache: std::collections::HashMap<String, Vec<f64>>,
    /// Maximum cache entries
    max_entries: usize,
}

impl OperationCache {
    fn new(max_entries: usize) -> Self {
        Self {
            matmul_cache: std::collections::HashMap::new(),
            max_entries,
        }
    }

    fn cache_key(op: &str, dims: &[usize]) -> String {
        format!("{}:{}", op, dims.iter().map(ToString::to_string).collect::<Vec<_>>().join("x"))
    }
}

impl TensorExecutor {
    /// Create a new tensor executor with default CPU backend.
    ///
    /// # Errors
    ///
    /// Returns error if initialization fails
    pub fn new() -> Result<Self> {
        Self::with_backend(TensorBackend::Cpu)
    }

    /// Create a tensor executor with specified backend.
    ///
    /// # Arguments
    ///
    /// * `backend` - Execution backend (CPU SIMD or GPU)
    ///
    /// # Errors
    ///
    /// Returns error if backend initialization fails
    pub fn with_backend(backend: TensorBackend) -> Result<Self> {
        Ok(Self {
            backend,
            cache: Arc::new(RwLock::new(OperationCache::new(100))),
        })
    }

    /// Get the current backend
    #[must_use]
    pub const fn backend(&self) -> TensorBackend {
        self.backend
    }

    /// Matrix multiplication: C = A * B
    ///
    /// # Arguments
    ///
    /// * `a` - Left matrix (m x k)
    /// * `b` - Right matrix (k x n)
    /// * `m` - Rows in A
    /// * `k` - Cols in A / Rows in B
    /// * `n` - Cols in B
    ///
    /// # Returns
    ///
    /// Result matrix C (m x n) as flattened vector
    ///
    /// # Errors
    ///
    /// Returns error if dimensions are incompatible or computation fails
    pub async fn matmul(
        &self,
        a: &[f64],
        b: &[f64],
        m: usize,
        k: usize,
        n: usize,
    ) -> Result<Vec<f64>> {
        // Validate dimensions
        if a.len() != m * k {
            return Err(RepartirError::InvalidTask {
                reason: format!("Matrix A size mismatch: expected {}, got {}", m * k, a.len()),
            });
        }
        if b.len() != k * n {
            return Err(RepartirError::InvalidTask {
                reason: format!("Matrix B size mismatch: expected {}, got {}", k * n, b.len()),
            });
        }

        // Check cache
        let cache_key = OperationCache::cache_key("matmul", &[m, k, n]);
        {
            let cache = self.cache.read().await;
            if let Some(result) = cache.matmul_cache.get(&cache_key) {
                return Ok(result.clone());
            }
        }

        // Perform computation using trueno
        let result = self.matmul_impl(a, b, m, k, n).await?;

        // Cache result
        let mut cache = self.cache.write().await;
        if cache.matmul_cache.len() < cache.max_entries {
            cache.matmul_cache.insert(cache_key, result.clone());
        }

        Ok(result)
    }

    /// Internal matrix multiplication implementation
    async fn matmul_impl(
        &self,
        a: &[f64],
        b: &[f64],
        m: usize,
        k: usize,
        n: usize,
    ) -> Result<Vec<f64>> {
        // Convert f64 to f32 for trueno (which uses f32)
        let a_f32: Vec<f32> = a.iter().map(|&x| x as f32).collect();
        let b_f32: Vec<f32> = b.iter().map(|&x| x as f32).collect();

        // Use tokio spawn_blocking for CPU-intensive computation
        tokio::task::spawn_blocking(move || {
            // Create trueno matrices
            let a_matrix = Matrix::from_vec(m, k, a_f32).map_err(|e| RepartirError::InvalidTask {
                reason: format!("Failed to create matrix A: {}", e),
            })?;
            let b_matrix = Matrix::from_vec(k, n, b_f32).map_err(|e| RepartirError::InvalidTask {
                reason: format!("Failed to create matrix B: {}", e),
            })?;

            // Perform matrix multiplication using trueno
            let c_matrix = a_matrix.matmul(&b_matrix).map_err(|e| RepartirError::InvalidTask {
                reason: format!("Matrix multiplication failed: {}", e),
            })?;

            // Convert back to f64 vector
            let result: Vec<f64> = c_matrix.as_slice().iter().map(|&x| x as f64).collect();
            Ok(result)
        })
        .await
        .map_err(|e| RepartirError::InvalidTask {
            reason: format!("Matmul task panicked: {}", e),
        })?
    }

    /// Element-wise scalar multiplication: B = a * A
    ///
    /// # Arguments
    ///
    /// * `data` - Input vector
    /// * `scalar` - Scalar multiplier
    ///
    /// # Returns
    ///
    /// Result vector with each element multiplied by scalar
    ///
    /// # Errors
    ///
    /// Returns error if computation fails
    pub async fn scale(&self, data: &[f64], scalar: f64) -> Result<Vec<f64>> {
        let data_f32: Vec<f32> = data.iter().map(|&x| x as f32).collect();
        let scalar_f32 = scalar as f32;

        tokio::task::spawn_blocking(move || {
            let vector = Vector::from_vec(data_f32);

            // Create vector of scalars and multiply element-wise
            let scalar_vec = Vector::from_vec(vec![scalar_f32; vector.len()]);
            let result = vector.mul(&scalar_vec).map_err(|e| RepartirError::InvalidTask {
                reason: format!("Scale failed: {}", e),
            })?;

            // Convert back to f64
            let result_f64: Vec<f64> = result.as_slice().iter().map(|&x| x as f64).collect();
            Ok(result_f64)
        })
        .await
        .map_err(|e| RepartirError::InvalidTask {
            reason: format!("Scale task panicked: {}", e),
        })?
    }

    /// Element-wise addition: C = A + B
    ///
    /// # Arguments
    ///
    /// * `a` - First vector
    /// * `b` - Second vector
    ///
    /// # Returns
    ///
    /// Result vector with element-wise sum
    ///
    /// # Errors
    ///
    /// Returns error if vectors have different lengths or computation fails
    pub async fn add(&self, a: &[f64], b: &[f64]) -> Result<Vec<f64>> {
        if a.len() != b.len() {
            return Err(RepartirError::InvalidTask {
                reason: format!("Vector size mismatch: {} != {}", a.len(), b.len()),
            });
        }

        let a_f32: Vec<f32> = a.iter().map(|&x| x as f32).collect();
        let b_f32: Vec<f32> = b.iter().map(|&x| x as f32).collect();

        tokio::task::spawn_blocking(move || {
            let a_vector = Vector::from_vec(a_f32);
            let b_vector = Vector::from_vec(b_f32);

            // Element-wise addition
            let result = a_vector.add(&b_vector).map_err(|e| RepartirError::InvalidTask {
                reason: format!("Addition failed: {}", e),
            })?;

            // Convert back to f64
            let result_f64: Vec<f64> = result.as_slice().iter().map(|&x| x as f64).collect();
            Ok(result_f64)
        })
        .await
        .map_err(|e| RepartirError::InvalidTask {
            reason: format!("Add task panicked: {}", e),
        })?
    }

    /// Element-wise ReLU activation: f(x) = max(0, x)
    ///
    /// # Arguments
    ///
    /// * `data` - Input vector
    ///
    /// # Returns
    ///
    /// Result vector with ReLU applied
    ///
    /// # Errors
    ///
    /// Returns error if computation fails
    pub async fn relu(&self, data: &[f64]) -> Result<Vec<f64>> {
        let data_f32: Vec<f32> = data.iter().map(|&x| x as f32).collect();

        tokio::task::spawn_blocking(move || {
            // Apply ReLU: max(0, x) - simple element-wise operation
            let result: Vec<f32> = data_f32.iter().map(|&x| if x > 0.0 { x } else { 0.0 }).collect();

            // Convert back to f64
            let result_f64: Vec<f64> = result.iter().map(|&x| x as f64).collect();
            Ok(result_f64)
        })
        .await
        .map_err(|e| RepartirError::InvalidTask {
            reason: format!("ReLU task panicked: {}", e),
        })?
    }

    /// Sum all elements in a vector (reduction)
    ///
    /// # Arguments
    ///
    /// * `data` - Input vector
    ///
    /// # Returns
    ///
    /// Sum of all elements
    ///
    /// # Errors
    ///
    /// Returns error if computation fails
    pub async fn sum(&self, data: &[f64]) -> Result<f64> {
        let data_f32: Vec<f32> = data.iter().map(|&x| x as f32).collect();

        tokio::task::spawn_blocking(move || {
            let vector = Vector::from_vec(data_f32);

            // Use trueno's SIMD sum
            let sum_f32 = vector.sum().map_err(|e| RepartirError::InvalidTask {
                reason: format!("Sum failed: {}", e),
            })?;

            Ok(sum_f32 as f64)
        })
        .await
        .map_err(|e| RepartirError::InvalidTask {
            reason: format!("Sum task panicked: {}", e),
        })?
    }

    /// Clear operation cache
    pub async fn clear_cache(&self) {
        let mut cache = self.cache.write().await;
        cache.matmul_cache.clear();
    }

    /// Get cache statistics
    ///
    /// # Returns
    ///
    /// Number of cached operations
    pub async fn cache_size(&self) -> usize {
        let cache = self.cache.read().await;
        cache.matmul_cache.len()
    }
}

impl Default for TensorExecutor {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| {
            // Safe fallback - should never fail with default config
            Self {
                backend: TensorBackend::Cpu,
                cache: Arc::new(RwLock::new(OperationCache::new(100))),
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_tensor_executor_creation() {
        let executor = TensorExecutor::new().unwrap();
        assert_eq!(executor.backend(), TensorBackend::Cpu);
    }

    #[tokio::test]
    async fn test_matmul_2x2() {
        let executor = TensorExecutor::new().unwrap();

        // [1 2]   [5 6]   [19 22]
        // [3 4] * [7 8] = [43 50]
        let a = vec![1.0, 2.0, 3.0, 4.0];
        let b = vec![5.0, 6.0, 7.0, 8.0];

        let result = executor.matmul(&a, &b, 2, 2, 2).await.unwrap();

        assert_eq!(result.len(), 4);
        assert!((result[0] - 19.0).abs() < 1e-10);
        assert!((result[1] - 22.0).abs() < 1e-10);
        assert!((result[2] - 43.0).abs() < 1e-10);
        assert!((result[3] - 50.0).abs() < 1e-10);
    }

    #[tokio::test]
    async fn test_matmul_dimension_mismatch() {
        let executor = TensorExecutor::new().unwrap();

        let a = vec![1.0, 2.0, 3.0]; // Wrong size
        let b = vec![5.0, 6.0, 7.0, 8.0];

        let result = executor.matmul(&a, &b, 2, 2, 2).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_scale() {
        let executor = TensorExecutor::new().unwrap();

        let data = vec![1.0, 2.0, 3.0, 4.0];
        let result = executor.scale(&data, 2.5).await.unwrap();

        assert_eq!(result.len(), 4);
        assert!((result[0] - 2.5).abs() < 1e-10);
        assert!((result[1] - 5.0).abs() < 1e-10);
        assert!((result[2] - 7.5).abs() < 1e-10);
        assert!((result[3] - 10.0).abs() < 1e-10);
    }

    #[tokio::test]
    async fn test_add() {
        let executor = TensorExecutor::new().unwrap();

        let a = vec![1.0, 2.0, 3.0];
        let b = vec![4.0, 5.0, 6.0];

        let result = executor.add(&a, &b).await.unwrap();

        assert_eq!(result.len(), 3);
        assert!((result[0] - 5.0).abs() < 1e-10);
        assert!((result[1] - 7.0).abs() < 1e-10);
        assert!((result[2] - 9.0).abs() < 1e-10);
    }

    #[tokio::test]
    async fn test_add_size_mismatch() {
        let executor = TensorExecutor::new().unwrap();

        let a = vec![1.0, 2.0];
        let b = vec![4.0, 5.0, 6.0];

        let result = executor.add(&a, &b).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_relu() {
        let executor = TensorExecutor::new().unwrap();

        let data = vec![-2.0, -1.0, 0.0, 1.0, 2.0];
        let result = executor.relu(&data).await.unwrap();

        assert_eq!(result.len(), 5);
        assert!((result[0] - 0.0).abs() < 1e-10); // -2 -> 0
        assert!((result[1] - 0.0).abs() < 1e-10); // -1 -> 0
        assert!((result[2] - 0.0).abs() < 1e-10); //  0 -> 0
        assert!((result[3] - 1.0).abs() < 1e-10); //  1 -> 1
        assert!((result[4] - 2.0).abs() < 1e-10); //  2 -> 2
    }

    #[tokio::test]
    async fn test_sum() {
        let executor = TensorExecutor::new().unwrap();

        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let result = executor.sum(&data).await.unwrap();

        assert!((result - 15.0).abs() < 1e-10);
    }

    #[tokio::test]
    async fn test_cache() {
        let executor = TensorExecutor::new().unwrap();

        assert_eq!(executor.cache_size().await, 0);

        // First matmul - should cache
        let a = vec![1.0, 2.0, 3.0, 4.0];
        let b = vec![5.0, 6.0, 7.0, 8.0];
        let _result1 = executor.matmul(&a, &b, 2, 2, 2).await.unwrap();

        assert_eq!(executor.cache_size().await, 1);

        // Second matmul with same dimensions - should hit cache
        let _result2 = executor.matmul(&a, &b, 2, 2, 2).await.unwrap();

        assert_eq!(executor.cache_size().await, 1);

        // Clear cache
        executor.clear_cache().await;
        assert_eq!(executor.cache_size().await, 0);
    }

    #[tokio::test]
    async fn test_backend_selection() {
        let executor = TensorExecutor::with_backend(TensorBackend::Cpu).unwrap();
        assert_eq!(executor.backend(), TensorBackend::Cpu);

        #[cfg(feature = "gpu")]
        {
            let gpu_executor = TensorExecutor::with_backend(TensorBackend::Gpu).unwrap();
            assert_eq!(gpu_executor.backend(), TensorBackend::Gpu);
        }
    }

    #[tokio::test]
    async fn test_default() {
        let executor = TensorExecutor::default();
        assert_eq!(executor.backend(), TensorBackend::Cpu);
    }
}
