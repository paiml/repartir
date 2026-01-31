# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**repartir** provides Sovereign AI-grade distributed computing primitives for Rust. It enables CPU, GPU, and remote execution with work-stealing schedulers, checkpointing, and TUI visualization.

## Build and Test Commands

```bash
# Build with default features (CPU only)
cargo build

# Build with all features
cargo build --features full

# Test
cargo test                           # Default features
cargo test --features full           # All features

# Run examples
cargo run --example basic_cpu
cargo run --example gpu_compute --features gpu
cargo run --example distributed --features remote

# Lint
cargo clippy -- -D warnings
cargo fmt --check
```

## Feature Flags

| Feature | Purpose |
|---------|---------|
| `cpu` (default) | Local multi-core execution with work-stealing |
| `gpu` | wgpu GPU compute (Vulkan/Metal/DX12/WebGPU) |
| `remote` | TCP-based distributed execution |
| `remote-tls` | TLS-secured remote execution |
| `tensor` | trueno SIMD tensor integration |
| `checkpoint` | trueno-db + Parquet state persistence |
| `tui` | Job flow TUI visualization |
| `serverless` | Serverless functions with pepita |
| `microvm` | MicroVM isolation with pepita |
| `full` | All features enabled |

## Architecture

### Executors

- **CpuExecutor**: Rayon-based parallel execution with work-stealing
- **GpuExecutor**: wgpu compute shaders for GPU workloads
- **RemoteExecutor**: Distributed execution across TCP/TLS connections

### Key Patterns

```rust
use repartir::{Pool, Task, Backend};

// Automatic backend selection
let pool = Pool::new();
let result = pool.execute(Task::new(|| expensive_computation())).await?;

// Explicit GPU execution
let gpu_task = Task::builder()
    .backend(Backend::Gpu)
    .build(|| gpu_kernel());

// Distributed execution
let executor = RemoteExecutor::builder()
    .add_worker("node1:9000")
    .add_worker("node2:9000")
    .build().await?;
```

## Integration with Sovereign AI Stack

repartir coordinates distributed workloads across the stack:

- **trueno**: SIMD tensor operations (via `tensor` feature)
- **trueno-db**: Checkpoint persistence (via `checkpoint` feature)
- **pepita**: Serverless and MicroVM execution
- **batuta**: Orchestration and workflow management

## Dependencies

- `tokio`: Async runtime
- `wgpu`: GPU compute (optional)
- `trueno`: SIMD tensors (optional)
- `trueno-db`: Checkpointing (optional)
- `ratatui`: TUI visualization (optional)

## Stack Documentation Search

Query this component's documentation and the entire Sovereign AI Stack using batuta's RAG Oracle:

```bash
# Index all stack documentation (run once, persists to ~/.cache/batuta/rag/)
batuta oracle --rag-index

# Search across the entire stack
batuta oracle --rag "your question here"

# Examples
batuta oracle --rag "distributed computing work stealing"
batuta oracle --rag "GPU compute wgpu"
batuta oracle --rag "checkpoint persistence"

# Check index status
batuta oracle --rag-stats
```

The RAG index includes CLAUDE.md, README.md, and source files from all stack components plus Python ground truth corpora for cross-language pattern matching.
