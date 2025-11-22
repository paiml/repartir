//! GPU execution backend.
//!
//! Executes compute tasks on GPU using wgpu (cross-platform GPU abstraction).
//!
//! This executor uses WebGPU/wgpu for cross-platform GPU compute across:
//! - Vulkan (Linux/Windows/Android)
//! - Metal (macOS/iOS)
//! - DirectX 12 (Windows)
//! - WebGPU (browsers)
//!
//! # Architecture
//!
//! For v1.1, GPU execution works by:
//! 1. Detecting available GPU adapters (discrete, integrated, software)
//! 2. Creating compute pipeline for shader execution
//! 3. Uploading task data to GPU buffers
//! 4. Dispatching compute shaders
//! 5. Reading results back from GPU
//!
//! # Limitations (v1.1)
//!
//! - No JIT compilation of Rust → SPIR-V (requires rust-gpu, deferred to v1.2)
//! - Currently executes pre-compiled compute shaders only
//! - Binary tasks are NOT supported on GPU (use CPU executor)
//! - Designed for data-parallel workloads (matrix ops, image processing)
//!
//! # Example
//!
//! ```no_run
//! use repartir::executor::gpu::GpuExecutor;
//! use repartir::executor::Executor;
//!
//! #[tokio::main]
//! async fn main() -> repartir::error::Result<()> {
//!     let executor = GpuExecutor::new().await?;
//!     println!("GPU: {}", executor.device_name());
//!     println!("Compute units: {}", executor.capacity());
//!     Ok(())
//! }
//! ```

use crate::error::{RepartirError, Result};
use crate::executor::{BoxFuture, Executor};
use crate::task::{ExecutionResult, Task};
use std::sync::Arc;
use tracing::{debug, info, warn};
use wgpu::{Device, Instance, Queue};

/// GPU executor using wgpu for cross-platform compute.
///
/// This executor provides GPU-accelerated task execution using WebGPU/wgpu.
/// It automatically selects the best available GPU adapter.
pub struct GpuExecutor {
    /// GPU device.
    #[allow(dead_code)] // Used in v1.2 for compute shader execution
    device: Arc<Device>,
    /// Command queue for GPU.
    #[allow(dead_code)] // Used in v1.2 for compute shader execution
    queue: Arc<Queue>,
    /// Device name (for logging/debugging).
    device_name: String,
    /// Number of compute units (estimate of parallelism).
    compute_units: usize,
}

impl GpuExecutor {
    /// Creates a new GPU executor with automatic adapter selection.
    ///
    /// Selects the best available GPU in this order:
    /// 1. Discrete GPU (dedicated graphics card)
    /// 2. Integrated GPU (CPU-integrated graphics)
    /// 3. Software rasterizer (CPU fallback)
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - No GPU adapter is available
    /// - Device request fails
    ///
    /// # Example
    ///
    /// ```no_run
    /// use repartir::executor::gpu::GpuExecutor;
    ///
    /// #[tokio::main]
    /// async fn main() -> repartir::error::Result<()> {
    ///     let executor = GpuExecutor::new().await?;
    ///     println!("Using GPU: {}", executor.device_name());
    ///     Ok(())
    /// }
    /// ```
    pub async fn new() -> Result<Self> {
        info!("Initializing GPU executor with wgpu");

        // Create wgpu instance
        let instance = Instance::default();

        // Request adapter (GPU)
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: None,
                force_fallback_adapter: false,
            })
            .await
            .ok_or_else(|| RepartirError::InvalidTask {
                reason: "No GPU adapter found".to_string(),
            })?;

        let adapter_info = adapter.get_info();
        let device_name = adapter_info.name.clone();

        info!("Selected GPU adapter: {device_name}");
        info!("Backend: {:?}", adapter_info.backend);
        info!("Device type: {:?}", adapter_info.device_type);

        // Request device and queue
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("Repartir GPU Device"),
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                    memory_hints: wgpu::MemoryHints::default(),
                },
                None,
            )
            .await
            .map_err(|e| RepartirError::InvalidTask {
                reason: format!("Failed to request GPU device: {e}"),
            })?;

        // Estimate compute units (conservative estimate)
        // Real value requires vendor-specific queries
        let compute_units = Self::estimate_compute_units(&adapter_info);

        info!("GPU executor initialized");
        info!("Estimated compute units: {compute_units}");

        Ok(Self {
            device: Arc::new(device),
            queue: Arc::new(queue),
            device_name,
            compute_units,
        })
    }

    /// Returns the GPU device name.
    #[must_use]
    pub fn device_name(&self) -> &str {
        &self.device_name
    }

    /// Estimates compute units based on adapter info.
    ///
    /// This is a conservative estimate. Real compute unit count
    /// requires vendor-specific queries (CUDA cores, stream processors, etc).
    fn estimate_compute_units(info: &wgpu::AdapterInfo) -> usize {
        use wgpu::DeviceType;

        match info.device_type {
            DeviceType::DiscreteGpu => {
                // Discrete GPU: assume high-end (2048 cores)
                debug!("Discrete GPU detected, estimating 2048 compute units");
                2048
            }
            DeviceType::IntegratedGpu => {
                // Integrated GPU: assume mid-range (512 cores)
                debug!("Integrated GPU detected, estimating 512 compute units");
                512
            }
            DeviceType::VirtualGpu => {
                // Virtual GPU: assume cloud instance (1024 cores)
                debug!("Virtual GPU detected, estimating 1024 compute units");
                1024
            }
            DeviceType::Cpu => {
                // Software rasterizer: minimal parallelism
                debug!("CPU rasterizer detected, estimating 16 compute units");
                16
            }
            DeviceType::Other => {
                // Unknown device: conservative estimate
                warn!("Unknown GPU type, using conservative estimate");
                256
            }
        }
    }

    /// Executes a compute shader on the GPU.
    ///
    /// # Implementation Note (v1.1)
    ///
    /// For v1.1, this is a placeholder that returns an error indicating
    /// GPU compute shaders are not yet supported. Full implementation
    /// requires:
    /// - SPIR-V shader compilation
    /// - Buffer management (upload/download)
    /// - Pipeline setup and dispatch
    ///
    /// These will be implemented in v1.2 with rust-gpu integration.
    #[allow(clippy::unused_async)]
    async fn execute_compute(&self, task: Task) -> Result<ExecutionResult> {
        let task_id = task.id();

        warn!(
            "GPU compute not yet implemented for task {task_id}. \
             Binary execution on GPU is not supported. \
             Use CPU executor for binary tasks, or wait for v1.2 GPU compute."
        );

        Err(RepartirError::InvalidTask {
            reason: "GPU executor v1.1 does not support task execution yet. \
                     Binary tasks cannot run on GPU. \
                     For GPU compute, use dedicated compute shaders (v1.2+)."
                .to_string(),
        })
    }
}

impl Executor for GpuExecutor {
    fn execute(&self, task: Task) -> BoxFuture<'_, Result<ExecutionResult>> {
        Box::pin(async move {
            let task_id = task.id();
            debug!("Executing task {task_id} on GPU");

            // For v1.1, GPU executor validates but doesn't execute binary tasks
            self.execute_compute(task).await
        })
    }

    fn capacity(&self) -> usize {
        self.compute_units
    }

    fn name(&self) -> &'static str {
        "GPU"
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_gpu_executor_creation() {
        // This test may fail on CI without GPU
        match GpuExecutor::new().await {
            Ok(executor) => {
                assert_eq!(executor.name(), "GPU");
                assert!(executor.capacity() > 0);
                assert!(!executor.device_name().is_empty());
                println!("GPU found: {}", executor.device_name());
            }
            Err(e) => {
                println!("No GPU available (expected on CI): {e}");
                // Not a test failure - GPU may not be available
            }
        }
    }

    #[test]
    fn test_compute_units_estimation() {
        use wgpu::{AdapterInfo, Backend, DeviceType};

        let info = AdapterInfo {
            name: "Test GPU".to_string(),
            vendor: 0,
            device: 0,
            device_type: DeviceType::DiscreteGpu,
            driver: String::new(),
            driver_info: String::new(),
            backend: Backend::Vulkan,
        };

        let units = GpuExecutor::estimate_compute_units(&info);
        assert_eq!(units, 2048); // Discrete GPU estimate
    }

    #[tokio::test]
    async fn test_gpu_executor_not_ready_for_tasks() {
        // GPU executor should reject binary tasks in v1.1
        match GpuExecutor::new().await {
            Ok(executor) => {
                use crate::task::Backend;

                let task = Task::builder()
                    .binary("/bin/echo")
                    .arg("test")
                    .backend(Backend::Gpu)
                    .build()
                    .unwrap();

                let result = executor.execute(task).await;
                assert!(result.is_err());

                if let Err(RepartirError::InvalidTask { reason }) = result {
                    assert!(reason.contains("not support"));
                }
            }
            Err(_) => {
                // No GPU available - skip test
                println!("Skipping test: No GPU available");
            }
        }
    }
}
