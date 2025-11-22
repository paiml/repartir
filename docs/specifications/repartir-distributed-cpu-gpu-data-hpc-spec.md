# Repartir: Distributed Computing Primitives for Rust

**Version:** 1.0.0-draft (Iron Lotus Enhanced)
**Status:** DRAFT - Iteration 2 (Sovereign AI Edition)
**Last Updated:** 2025-11-22
**Quality Framework:** Iron Lotus + Certeza

## Executive Summary

Repartir is a **Sovereign AI-grade** pure Rust distributed computing library that enables seamless execution of Rust binaries across heterogeneous compute resources including CPUs, GPUs, and remote machines. Unlike existing solutions that rely on C dependencies or heavyweight frameworks, repartir provides zero-C, minimal-dependency primitives for distributed work orchestration, leveraging the PAIML stack for maximum performance and integration.

Built on the **Iron Lotus Framework**—a Toyota Way-derived engineering discipline—repartir embodies the principles of Genchi Genbutsu (transparency to the AST), Jidoka (automated quality gates), and Kaizen (continuous improvement through technical debt elimination). This is not merely a distributed computing framework; it is a **clean-slate reimagining** of distributed systems for an era where digital sovereignty, memory safety, and auditability are non-negotiable.

**Key Differentiators:**
- **100% Rust, Absolute Zero C/C++ Dependencies**: True sovereignty through complete auditability
- **Iron Lotus Quality Protocol**: Toyota Way principles operationalized as code review gates
- **Certeza-Enforced Testing**: 95% coverage, 80% mutation score, formal verification
- **Unified Execution Model**: CPU, GPU, and remote resources with zero FFI boundaries
- **Tight PAIML Stack Integration**: trueno (SIMD), aprender (ML), paiml-mcp-agent-toolkit
- **Inspired by Proven Patterns**: ZeroMQ messaging, RabbitMQ broker architecture, Hugging Face distributed ML
- **Work-Stealing Scheduler**: Efficient load balancing with fault tolerance
- **Supply Chain Sovereignty**: Deterministic builds, binary signing, dependency pinning

## 1. Architecture Overview

### 1.1 Core Components

```
┌─────────────────────────────────────────────────────────────┐
│                    Repartir Core Library                    │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  ┌──────────────┐  ┌──────────────┐  ┌─────────────────┐  │
│  │   Scheduler  │  │   Executor   │  │  Task Registry  │  │
│  │  (Work-Steal)│  │   Manager    │  │  (Binary Mgmt)  │  │
│  └──────────────┘  └──────────────┘  └─────────────────┘  │
│                                                             │
│  ┌──────────────┐  ┌──────────────┐  ┌─────────────────┐  │
│  │   Message    │  │  Fault Tol.  │  │  Resource Pool  │  │
│  │   Transport  │  │  & Recovery  │  │   (CPU/GPU/Net) │  │
│  └──────────────┘  └──────────────┘  └─────────────────┘  │
│                                                             │
├─────────────────────────────────────────────────────────────┤
│            Execution Backends (Pluggable)                   │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  ┌──────────────┐  ┌──────────────┐  ┌─────────────────┐  │
│  │ CPU Backend  │  │ GPU Backend  │  │ Remote Backend  │  │
│  │  (Native +   │  │ (wgpu/vulkan │  │  (SSH/Custom    │  │
│  │   trueno)    │  │   compute)   │  │   Protocol)     │  │
│  └──────────────┘  └──────────────┘  └─────────────────┘  │
│                                                             │
└─────────────────────────────────────────────────────────────┘
         │                  │                   │
         ▼                  ▼                   ▼
    ┌────────┐        ┌─────────┐        ┌──────────┐
    │ trueno │        │  wgpu   │        │  tokio   │
    │ aprender        │ (Rust)  │        │ (async)  │
    └────────┘        └─────────┘        └──────────┘
```

### 1.2 Design Principles (Iron Lotus Foundation)

#### Core Technical Principles

1. **Pure Rust Everything (Sovereignty)**: Zero C FFI boundaries, leveraging Rust's safety and performance [1]
   - **NSA/CISA Mandate**: Memory-safe languages reduce critical infrastructure vulnerabilities [8]
   - **Auditability**: Every line of execution visible from source to AST to binary
   - **No Binary Blobs**: Zero tolerance for opaque pre-compiled libraries

2. **Message-Oriented (ZeroMQ Patterns)**: REQ/REP, PUB/SUB, PUSH/PULL [2]
   - Pure Rust transport layer (tokio + bincode)
   - Optional TLS via rustls (zero-OpenSSL)
   - Publish-subscribe patterns for control plane [2]

3. **Work Stealing (Efficient Load Balancing)**: Across heterogeneous resources [3]
   - Blumofe & Leiserson's provably optimal algorithm [3]
   - Adaptive to CPU/GPU/remote performance characteristics
   - NUMA-aware task placement

4. **Binary Execution (Controlled Environment)**: Run compiled Rust binaries, not scripts
   - Binary signing with ed25519 (via `ring`)
   - Whitelisting by content hash (SHA-256)
   - renacer-based process isolation

5. **Fault Tolerance (Chandra-Toueg Model)**: Checkpoint/restart, task migration, failure detection [4]
   - Unreliable failure detectors for reliable distributed systems
   - Checkpoint to trueno-db (v2) or local storage
   - Task migration with state serialization

6. **Resource Awareness (Hardware Topology)**: CPU topology, GPU capabilities, network latency
   - Rust-native hwloc bindings for NUMA
   - wgpu adapter enumeration for GPU discovery
   - Cost model: execution time + data transfer

#### Toyota Way Principles (Operational Discipline)

7. **Genchi Genbutsu (現地現物 - "Go and See")**: Radical transparency
   - AST-level code inspection via pmat
   - Zero black boxes: trace from API → scheduler → GPU shader
   - Dependency auditing: every crate vetted

8. **Jidoka (自働化 - "Automation with Human Touch")**: Quality gates as Andon cords
   - CI pipeline as automated immune system
   - Build fails on: safety violations, lint warnings, test failures, coverage drops
   - Human review only after machines pass (Phase 2)

9. **Kaizen (改善 - "Continuous Improvement")**: Technical Debt Grading
   - TDG (Technical Debt Grade) must never decrease
   - "Ratchet Effect": each PR improves or maintains quality
   - Five Whys root cause analysis for production incidents

10. **Muda (無駄 - "Waste Elimination")**: Seven wastes applied to distributed computing
    - **Overproduction**: No YAGNI features, zero SATD (TODO without ticket)
    - **Waiting**: Incremental compilation (sccache), fast CI
    - **Transportation**: Zero-copy data flow, single language (no Python ↔ C++ context switch)
    - **Overprocessing**: Complexity limits (≤10 cyclomatic complexity)
    - **Inventory**: No stale PRs, merge small batches frequently
    - **Motion**: Integrated tooling (LSP, rust-analyzer)
    - **Defects**: EXTREME TDD with mutation testing (≥80% score)

### 1.3 Stack Integration

**Primary Dependencies (PAIML Stack):**
- `trueno`: SIMD-accelerated tensor operations, numerical computing foundation
- `aprender`: ML workloads, training/inference distribution
- `trueno-db`: Optional data layer (v2+), distributed state management
- `renacer`: Process lifecycle management, restart/recovery
- `paiml-mcp-agent-toolkit`: Agent-based orchestration, autonomous scheduling
- `bashrs`: Shell integration for remote execution

**External Dependencies (Minimal):**
- `tokio`: Async runtime (industry standard)
- `wgpu`: GPU compute (pure Rust, via gfx-rs)
- `serde`: Serialization (ubiquitous)
- `bincode`: Binary protocol efficiency

## 2. CPU Execution Backend

### 2.1 Local CPU Execution

**Thread Pool Architecture:**
- Tokio-based async executor with work-stealing scheduler [3]
- Thread count: auto-detect from `num_cpus` or user-configurable
- NUMA-aware allocation via `hwloc` bindings (Rust-only)
- Priority queues for task scheduling

**Binary Invocation:**
```rust
pub struct CpuExecutor {
    pool: WorkStealingPool,
    trueno_context: TruenoContext,
}

impl CpuExecutor {
    pub async fn execute_binary(
        &self,
        binary_path: PathBuf,
        args: Vec<String>,
        env: HashMap<String, String>,
    ) -> Result<ExecutionResult> {
        // Fork-exec model with bashrs integration
        // Capture stdout/stderr via pipes
        // Return exit code + outputs
    }
}
```

**Integration with Trueno:**
- Share SIMD context across tasks
- Memory-mapped tensor sharing between processes
- Zero-copy data passing via shared memory

### 2.2 CPU Task Model

**Task Types:**
1. **Binary Task**: Execute a Rust binary with args/env
2. **Function Task**: Serialize closure (requires stable ABI, v2 feature)
3. **Pipeline Task**: Chain multiple binaries with stdout → stdin

**Example:**
```rust
let task = Task::Binary {
    path: "./target/release/worker",
    args: vec!["--input", "data.bin"],
    affinity: CpuAffinity::Core(4),
};

let result = executor.submit(task).await?;
```

## 3. GPU Execution Backend

### 3.1 GPU Compute with wgpu

**Rationale for wgpu:**
- Pure Rust implementation (zero C dependencies)
- Cross-platform: Vulkan, Metal, DX12, WebGPU
- Compute shader support (WGSL, SPIR-V)
- Active ecosystem (gfx-rs team)

**GPU Task Abstraction:**
```rust
pub struct GpuExecutor {
    device: wgpu::Device,
    queue: wgpu::Queue,
    trueno_gpu_context: Option<TruenoGpuContext>,
}

impl GpuExecutor {
    pub async fn execute_shader(
        &self,
        shader_binary: &[u8],  // SPIR-V compiled from Rust-GPU
        input_buffers: Vec<GpuBuffer>,
        output_buffers: Vec<GpuBuffer>,
    ) -> Result<()> {
        // Submit compute shader
        // Synchronize with CPU via fences
    }
}
```

### 3.2 Rust-GPU Integration

**Compile Rust to GPU:**
- Use `rust-gpu` to compile Rust code to SPIR-V [5]
- Distribute SPIR-V binaries to remote GPUs
- Unified language: write CPU and GPU code in Rust

**Workflow:**
1. Write GPU kernel in Rust with `#[spirv(compute)]`
2. Compile to SPIR-V via `spirv-builder`
3. Repartir loads SPIR-V and executes on GPU
4. Return results to CPU via buffer readback

**Data Transfer:**
- Minimize CPU ↔ GPU copies
- Use staging buffers for async transfers
- Pin memory for DMA when available (via wgpu abstractions)

### 3.3 GPU Scheduling

**Device Discovery:**
- Enumerate all GPUs via wgpu adapter enumeration
- Track capabilities: compute units, memory, VRAM
- Prioritize discrete GPUs over integrated

**Task Assignment:**
- Bin-packing heuristic: assign tasks to GPU with sufficient VRAM
- FIFO queue per GPU device
- Preemption support (limited by GPU hardware)

## 4. Remote Execution Backend

### 4.1 Message Transport

**Inspired by ZeroMQ Patterns:** [2]

1. **REQ/REP (Request-Reply):** Synchronous RPC for task submission
2. **PUSH/PULL (Pipeline):** Asynchronous work distribution
3. **PUB/SUB (Publish-Subscribe):** Broadcast control messages (shutdown, checkpoint)

**Protocol Design:**
- Binary protocol via `bincode` (efficient, Rust-native)
- Message types: `TaskSubmit`, `TaskResult`, `Heartbeat`, `TaskCancel`
- Compression: optional `lz4` for large payloads

**Transport Layer:**
- TCP with `tokio::net::TcpStream`
- Optional TLS via `rustls` (zero-OpenSSL)
- Custom framing: length prefix (4 bytes) + payload

### 4.2 Broker Architecture (RabbitMQ-Inspired) [6]

**Central Coordinator (Optional):**
```
   Client                Broker               Workers
     │                     │                     │
     ├──── Submit Task ───>│                     │
     │                     ├──── Dispatch ──────>│
     │                     │<──── Heartbeat ─────┤
     │<──── Task Result ───┤                     │
     │                     │<──── Result ────────┤
```

**Peer-to-Peer Mode (ZeroMQ-Style):**
- No central broker, direct client → worker communication
- Service discovery via mDNS or static config
- Lower latency, higher resilience

### 4.3 Remote Binary Deployment

**Binary Distribution:**
1. **Push Model**: Client sends binary to worker via file transfer
2. **Pull Model**: Worker downloads binary from shared storage (S3, trueno-db)
3. **Pre-Deploy**: Assume binaries are pre-installed on workers

**Execution Sandbox:**
- Use `renacer` for process isolation and lifecycle management
- Capture stdout/stderr via pipes
- Set resource limits (CPU time, memory) via `rlimit` (Rust bindings)

**Security:**
- Optional binary signing (ed25519 via `ring`)
- Worker whitelisting (allow only specific binary hashes)
- No arbitrary code execution (only vetted binaries)

## 5. Work-Stealing Scheduler

### 5.1 Algorithm [3]

**Per-Worker Queues:**
- Each worker (CPU thread, GPU, remote machine) has a double-ended queue (deque)
- Local tasks: push to back, pop from back (LIFO, cache-friendly)
- Steal tasks: steal from front of other workers' queues (FIFO)

**Load Balancing:**
- Idle workers randomly probe other workers for tasks
- Steal half of victim's queue (amortize stealing overhead)
- Exponential backoff if no tasks found

**Advantages:**
- Scalable: O(1) task submission and local pop
- Adaptive: automatically balances load across heterogeneous resources
- Cache-friendly: local LIFO reduces cache misses

### 5.2 Heterogeneous Resource Handling

**Task Affinity:**
- Tasks can specify required resources: `CpuOnly`, `GpuRequired`, `RemoteAllowed`
- Scheduler only assigns tasks to compatible workers

**Cost Model:**
- Estimate task execution time based on resource type
- Prefer local CPU < remote CPU < GPU (depending on task)
- Account for data transfer costs (CPU → GPU, local → remote)

**Priority Scheduling:**
- High-priority tasks bypass work-stealing and go to dedicated queue
- Preempt low-priority tasks if necessary (checkpoint state)

## 6. Fault Tolerance and Recovery

### 6.1 Failure Detection [4]

**Heartbeat Protocol:**
- Workers send periodic heartbeats (1-5 second interval)
- Coordinator tracks last heartbeat timestamp
- Timeout threshold: 3x heartbeat interval (adjustable)

**Failure Modes:**
- Worker crash: no heartbeat, no response to probe
- Network partition: heartbeat lost, but worker alive
- Task hang: task exceeds expected execution time

### 6.2 Recovery Strategies

**Task Retry:**
- Retry failed tasks up to N times (configurable, default 3)
- Exponential backoff between retries
- Blacklist workers with repeated failures

**Checkpointing:**
- Tasks can opt-in to checkpointing (via callback)
- Checkpoint state to trueno-db or local disk
- Resume from last checkpoint on failure

**Replication (Future):**
- Run critical tasks on multiple workers (v2 feature)
- First-to-finish wins, cancel others
- Increases reliability at cost of resources

### 6.3 Integration with Renacer

**Process Supervision:**
- `renacer` restarts crashed worker processes
- Exponential backoff for restart attempts
- Health checks: worker must re-register with coordinator

## 7. Distributed ML Patterns (Hugging Face-Inspired)

### 7.1 Model Parallelism

**Pipeline Parallelism:**
- Split model across multiple GPUs (layers → devices)
- Repartir schedules micro-batches across pipeline stages
- Overlap computation and communication

**Tensor Parallelism:**
- Split large tensors across multiple devices (via trueno)
- Repartir coordinates all-reduce operations
- Minimize cross-device communication

### 7.2 Data Parallelism

**Distributed Training:**
- Replicate model on multiple workers
- Each worker processes a shard of data
- Aggregate gradients via ring-allreduce [7]

**Integration with Aprender:**
- Aprender provides gradient computation
- Repartir handles distribution and aggregation
- Trueno provides SIMD-optimized operations

### 7.3 Inference Scaling

**Distributed Inference (Hugging Face Accelerate-Style):**
- Replicate model on multiple workers
- Load balance inference requests via PUSH/PULL pattern
- Batching: accumulate requests, dispatch batch to worker

**Example:**
```rust
let inference_pool = RepartirPool::new()
    .add_gpu_workers(4)
    .add_remote_workers(vec!["gpu1.cluster", "gpu2.cluster"])
    .build()?;

let model_binary = "./target/release/llama_inference";
inference_pool.broadcast_binary(model_binary).await?;

for request in request_stream {
    let task = Task::Binary {
        path: model_binary,
        args: vec!["--prompt", &request.prompt],
        backend: Backend::Gpu,
    };
    let result = inference_pool.submit(task).await?;
}
```

## 8. API Design

### 8.1 High-Level API

```rust
use repartir::{Pool, Task, Backend};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize execution pool
    let pool = Pool::builder()
        .cpu_workers(8)
        .gpu_workers(2)
        .remote_workers(vec!["worker1:9000", "worker2:9000"])
        .build()
        .await?;

    // Submit CPU task
    let cpu_task = Task::binary("./compute")
        .args(vec!["--input", "data.csv"])
        .backend(Backend::Cpu)
        .build();

    let cpu_result = pool.submit(cpu_task).await?;

    // Submit GPU task
    let gpu_task = Task::shader("./shader.spv")
        .input_buffer(&tensor_data)
        .backend(Backend::Gpu)
        .build();

    let gpu_result = pool.submit(gpu_task).await?;

    // Submit remote task
    let remote_task = Task::binary("./bigcompute")
        .backend(Backend::Remote)
        .build();

    let remote_result = pool.submit(remote_task).await?;

    Ok(())
}
```

### 8.2 Low-Level API

```rust
use repartir::{Scheduler, CpuExecutor, GpuExecutor, RemoteExecutor};

// Manual scheduler control
let scheduler = Scheduler::new();

let cpu_exec = CpuExecutor::new(8);
let gpu_exec = GpuExecutor::new().await?;
let remote_exec = RemoteExecutor::connect("worker:9000").await?;

scheduler.register_backend(Backend::Cpu, Box::new(cpu_exec));
scheduler.register_backend(Backend::Gpu, Box::new(gpu_exec));
scheduler.register_backend(Backend::Remote, Box::new(remote_exec));

let task = Task::binary("./worker").build();
let future = scheduler.schedule(task);
let result = future.await?;
```

## 9. Performance Considerations

### 9.1 Benchmarking Strategy

**Microbenchmarks:**
- Task submission latency: < 1μs (local CPU)
- Task stealing overhead: < 100ns per steal attempt
- Message serialization: < 100ns for typical task (bincode)

**Macrobenchmarks:**
- Weak scaling: 1000 workers, 1M tasks, measure throughput
- Strong scaling: fixed workload, vary worker count
- GPU vs CPU: compare task execution time for compute-bound workload

**Comparison Baseline:**
- Ray (Python, heavy dependencies)
- Dask (Python, pandas-focused)
- Apache Arrow Flight (C++, but cross-language)

**Goal:** 10x lower latency than Python-based systems, comparable to C++ frameworks

### 9.2 Optimization Techniques

**Zero-Copy Paths:**
- Use `mmap` for large binary inputs (via `memmap2`)
- Share trueno tensors via shared memory (avoid serialization)
- GPU: use persistent staging buffers

**Batching:**
- Accumulate small tasks into batches to amortize overhead
- GPU: batch multiple compute shader dispatches

**Prefetching:**
- Predict next tasks based on workflow patterns (via paiml-mcp-agent-toolkit)
- Prefetch binaries to remote workers before task assignment

## 10. Testing and Validation

### 10.1 Unit Tests

**Per-Component:**
- CPU executor: spawn 10k tasks, verify all complete
- GPU executor: run matrix multiply shader, verify correctness
- Remote executor: mock network transport, test failure injection
- Scheduler: verify work-stealing correctness with concurrent tasks

### 10.2 Integration Tests

**End-to-End Scenarios:**
- Heterogeneous pool: submit 100 tasks (CPU, GPU, remote), verify all succeed
- Fault injection: kill workers mid-task, verify retry and recovery
- Binary distribution: deploy binary to 10 remote workers, execute concurrently

### 10.3 Property-Based Testing

**Use `proptest`:**
- Property: "All submitted tasks eventually complete or fail"
- Property: "No task is executed more than `max_retries` times"
- Property: "Work-stealing maintains FIFO order for stolen tasks"

### 10.4 Mutation Testing (Certeza Integration)

**Coverage Goals:**
- Line coverage: ≥95%
- Mutation coverage: ≥80%
- Run `certeza` before every commit (per CLAUDE.md)

## 11. Sovereign AI Considerations

### 11.1 The Crisis of Legitimacy in Distributed Systems

**Current State of the Art:**
Most distributed computing frameworks (Ray, Dask, Apache Spark) suffer from the same architectural pathology as modern AI infrastructure: they are Python facades over C/C++ substrates. This creates a "black box" architecture fundamentally incompatible with digital sovereignty.

**Sovereignty Requirements:**
1. **Auditability**: Ability to trace execution from user API → scheduler logic → kernel execution → hardware instruction
2. **Supply Chain Independence**: Deterministic rebuild from source code held in national/organizational repositories
3. **No Foreign Binary Dependencies**: Zero reliance on opaque shared libraries (libtorch.so, CUDA runtime)
4. **Memory Safety Guarantees**: Provable absence of buffer overflows, use-after-free, data races

**Repartir's Sovereign Architecture:**
```
User Code (Rust)
    ↓
Repartir API (Rust) ← Fully auditable, no FFI boundary
    ↓
Scheduler (Rust) ← Work-stealing algorithm visible in source
    ↓
Executor Backend (Rust)
    ├─ CPU: Native Rust + trueno (SIMD, pure Rust)
    ├─ GPU: wgpu (pure Rust) → SPIR-V (compiled from rust-gpu)
    └─ Remote: tokio (pure Rust) + bincode serialization
```

**No Hidden Layers**: Unlike PyTorch (where `torch.matmul()` disappears into 500K lines of C++ in `libtorch`), every operation in repartir is traceable through Rust source code.

### 11.2 Supply Chain Security (Genchi Genbutsu Applied)

**The Dependency Threat:**
Research demonstrates that ML framework supply chains are porous to injection attacks [9]. Malicious actors can compromise PyPI packages, injecting backdoors into training pipelines.

**Repartir's Defenses:**

#### Dependency Pinning (Cargo.lock Enforcement)
```toml
# Cargo.toml: Specify exact dependency versions
[dependencies]
trueno = "=0.5.2"  # Exact version, not "^0.5"
tokio = { version = "=1.35.1", features = ["full"] }
```

- **Cargo.lock Commitment**: Lock file committed to git, reviewed in every PR
- **No Binary Crates**: Reject any crate that downloads pre-compiled binaries during build
- **Audit Trail**: `cargo tree` output logged in CI for every build

#### Binary Signing & Verification
```rust
use ring::signature::{Ed25519KeyPair, UnparsedPublicKey, ED25519};

pub struct SignedBinary {
    binary_hash: [u8; 32],      // SHA-256 of executable
    signature: Vec<u8>,          // ed25519 signature
    signer_pubkey: Vec<u8>,      // Public key of authorized signer
}

impl SignedBinary {
    pub fn verify(&self, binary_content: &[u8]) -> Result<(), SignatureError> {
        // 1. Hash the binary
        let hash = Sha256::digest(binary_content);

        // 2. Verify signature
        let public_key = UnparsedPublicKey::new(&ED25519, &self.signer_pubkey);
        public_key.verify(&hash, &self.signature)?;

        Ok(())
    }
}
```

**Deployment Workflow:**
1. Developer builds binary locally: `cargo build --release`
2. Sign binary with organizational key: `repartir sign ./target/release/worker`
3. Distribute signed binary + signature to worker nodes
4. Worker verifies signature before execution (reject if invalid)

#### License Compliance (cargo-deny)
```toml
# deny.toml
[licenses]
unlicensed = "deny"
copyleft = "deny"  # Reject AGPL/GPL in critical paths
allow = ["MIT", "Apache-2.0", "BSD-3-Clause"]
```

**Automated Scanning**: Run `cargo deny check licenses` in CI to block unapproved licenses.

### 11.3 The "Clean Slate" Justification

**Peisert et al. (2012): "Turtles All the Way Down"** [13]
- **Thesis**: Security cannot be retrofitted; it must be designed from the foundation up
- **Application**: Wrapping PyTorch in Rust FFI bindings does NOT achieve sovereignty
  - The unsafety is merely pushed down one layer (still executing C++ kernels)
  - FFI boundary itself is a source of undefined behavior [20]

**Repartir's Clean Slate:**
- **No Compromise**: Reject "good enough" hybrid solutions (Rust API + C++ compute)
- **First Principles**: Rebuild tensor operations in pure Rust (via trueno)
- **GPU Compute**: Use rust-gpu to compile Rust → SPIR-V, not CUDA C++ [5]

**Cost-Benefit Analysis:**
| Approach | Development Cost | Maintenance Cost | Sovereignty | Security |
|----------|------------------|------------------|-------------|----------|
| Python + C++ (Ray) | Low | Very High (CVEs) | None | Poor |
| Rust FFI to libtorch | Medium | High (FFI bugs) | Low | Moderate |
| Pure Rust (Repartir) | **High** | **Low** | **Full** | **Excellent** |

**Strategic Argument**: The "rewrite cost" is a one-time investment; the maintenance and sovereignty benefits compound over decades.

## 12. The Iron Lotus Code Review Framework

This section operationalizes the Toyota Production System into a rigorous, enforceable code review protocol. Every change to repartir MUST pass through three phases.

### 12.1 Phase 1: Jidoka Gates (Automated Quality Enforcement)

**Philosophy**: "Stop and Fix" — No defect proceeds downstream.

**Implementation**: CI pipeline as Andon Cord (pulls the line on any failure).

#### Automated Pre-Flight Checklist

| Gate | Tool | Standard | Rationale (Toyota Principle) | Blocking |
|------|------|----------|------------------------------|----------|
| **Safety** | `rustc` | Zero Errors | Safety First (Jidoka) | ✅ YES |
| **Lints** | `clippy` | `-D warnings` | Catch common mistakes (Poka-Yoke) | ✅ YES |
| **Format** | `rustfmt` | Check Mode | Standardization (Muda: Motion waste) | ✅ YES |
| **Coverage** | `cargo-llvm-cov` | ≥ 95% | Genchi Genbutsu (verify code exercised) | ✅ YES |
| **Mutation** | `cargo-mutants` | ≥ 80% Score | Defect Prevention (Kaizen) | ✅ YES (Tier 3) |
| **Docs** | `rustdoc` | No broken links, ≥90% coverage | Knowledge Preservation (Standardization) | ✅ YES |
| **Debt** | `pmat tdg` | TDG ≥ Current | Kaizen (No Regression) | ✅ YES |
| **Build** | `bashrs` | Deterministic | Stability (Jidoka) | ✅ YES |
| **Security** | `cargo-audit` | Zero vulnerabilities | National Security (Sovereignty) | ✅ YES |
| **Dependencies** | `cargo-deny` | License + source check | Supply Chain (Sovereignty) | ✅ YES |
| **SATD** | `pmat satd` | Zero un-ticketed TODOs | Inventory Waste (Muda) | ✅ YES |
| **Complexity** | `pmat complexity` | Cyclomatic ≤ 10 | Overprocessing (Muda) | ✅ YES |

#### CI Configuration (.github/workflows/jidoka-gates.yml)

```yaml
name: Jidoka Quality Gates
on: [push, pull_request]

jobs:
  tier1-quick-checks:
    runs-on: ubuntu-latest
    timeout-minutes: 5
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          components: clippy, rustfmt

      - name: Cargo Check (Safety)
        run: cargo check --all-features

      - name: Clippy (Lints)
        run: cargo clippy --all-features -- -D warnings

      - name: Rustfmt (Formatting)
        run: cargo fmt -- --check

      - name: Unit Tests
        run: cargo test --lib

  tier2-commit-gates:
    runs-on: ubuntu-latest
    timeout-minutes: 30
    steps:
      - uses: actions/checkout@v4

      - name: Install Tools
        run: |
          cargo install cargo-llvm-cov
          cargo install --git https://github.com/paiml/paiml-mcp-agent-toolkit pmat

      - name: Full Test Suite
        run: cargo test --all-features

      - name: Coverage Check
        run: |
          cargo llvm-cov --all-features --lcov --output-path lcov.info
          coverage=$(cargo llvm-cov report --summary-only | grep -oP '\d+\.\d+(?=%)')
          if (( $(echo "$coverage < 95.0" | bc -l) )); then
            echo "❌ Coverage $coverage% < 95%"
            exit 1
          fi

      - name: Documentation Check
        run: cargo doc --no-deps --all-features --document-private-items

      - name: PMAT Technical Debt Grade
        run: pmat tdg --fail-on-regression

      - name: PMAT SATD Check
        run: pmat satd --max-satd 0

      - name: Complexity Analysis
        run: pmat complexity --max-cyclomatic 10

  tier3-security-audit:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Cargo Audit (CVE Scan)
        run: |
          cargo install cargo-audit
          cargo audit

      - name: Cargo Deny (License + Source)
        run: |
          cargo install cargo-deny
          cargo deny check licenses
          cargo deny check sources
```

### 12.2 Phase 2: Genchi Genbutsu Review (Human Architectural Audit)

**Philosophy**: "Go and See" — Inspect the actual place where work is done (the AST, not just the diff).

**When**: After Phase 1 passes, before merge approval.

**Who**: Senior engineer with domain expertise (distributed systems, GPU compute, or security).

#### Protocol A: Sovereignty & Supply Chain Audit

**Objective**: Ensure no foreign dependencies or black boxes.

**Checklist**:
- [ ] **No Binary Blobs**: Verify no `.so`, `.dll`, or `.a` files added to repository
- [ ] **Pure Rust Preference**: If adding a crate, verify:
  - [ ] No `build.rs` that downloads binaries
  - [ ] No FFI to C libraries (exception: OS syscalls via `libc`, but prefer `rustix`)
  - [ ] Crate is published on crates.io (not git dependency with ambiguous provenance)
- [ ] **Dependency Pinning**: `Cargo.lock` updated and reviewed
- [ ] **License Verification**: Run `cargo deny check licenses` locally
- [ ] **Transitive Dependencies**: Run `cargo tree` and spot-check for unexpected crates

**Example Review Comment**:
> 🚨 **Sovereignty Violation**: This PR adds `openssl-sys` as a dependency. Per Iron Lotus policy, we use `rustls` for TLS. Please replace with:
> ```toml
> rustls = "0.21"
> tokio-rustls = "0.24"
> ```

#### Protocol B: The `unsafe` Boundary Inspection

**Objective**: Rigorous containment of the `unsafe` keyword (memory safety critical).

**Policy**: `unsafe` is permitted ONLY when:
1. Interacting with hardware (MMIO, inline assembly for SIMD)
2. Performance-critical path with benchmark proof (see Protocol E)
3. Accompanied by formal safety proof or Miri verification

**Checklist**:
- [ ] **Safety Comment Mandatory**: Every `unsafe` block preceded by `// SAFETY: <explanation>`
  ```rust
  // SAFETY: The pointer `ptr` is guaranteed to be:
  // 1. Non-null (checked by previous assertion)
  // 2. Aligned (allocated via Layout::from_size_align_unchecked)
  // 3. Valid for reads (lifetime tied to &self)
  unsafe { ptr.read() }
  ```
- [ ] **Minimal Scope**: `unsafe` block wraps only the unsafe operation, not entire function
- [ ] **Justification**: Performance justification requires benchmark showing ≥20% improvement
- [ ] **Miri Verification**: Run `cargo miri test` on functions containing `unsafe`
- [ ] **Alternative Exploration**: Document why safe alternative (e.g., `Vec` instead of raw pointer) is insufficient

**Review Template**:
```markdown
## Unsafe Block Review: `src/executor/gpu.rs:142`

**Safety Invariant**:
- Pointer `device_mem` is valid because it's allocated via `wgpu::Buffer::map_read()` and unmapped in the `Drop` impl.

**Miri Check**: ✅ PASS (`cargo miri test gpu_buffer_read`)

**Alternatives Considered**:
- Using `wgpu::Buffer::slice().get_mapped_range()` (safe API) would require `&mut self`, which conflicts with our concurrent read pattern.

**Approval**: ✅ Unsafe usage justified and verified.
```

#### Protocol C: Concurrency & Heijunka (平準化 - "Leveling")

**Objective**: Prevent race conditions, deadlocks, and ensure even workload distribution.

**Checklist**:
- [ ] **Lock Ordering**: If multiple `Mutex` held, is order consistent across codebase?
  ```rust
  // GOOD: Consistent order (workers → scheduler)
  let _w = self.workers.lock();
  let _s = self.scheduler.lock();

  // BAD: Inverted order (potential deadlock)
  let _s = self.scheduler.lock();
  let _w = self.workers.lock();
  ```
- [ ] **Send/Sync Verification**: Are `unsafe impl Send/Sync` justified? (Compiler checks this for safe code)
- [ ] **Channel Preference**: Prefer message-passing (channels) over shared state (locks) per Actor model
- [ ] **Work Distribution**: Does scheduler ensure even load? (Heijunka principle: smooth production flow)

#### Protocol D: Kaizen & Zero SATD

**Objective**: Eliminate technical debt "inventory" waste.

**Checklist**:
- [ ] **No Bare TODOs**: Search for `/TODO|FIXME|HACK/` without ticket number
  ```rust
  // ❌ FORBIDDEN
  // TODO: optimize this later

  // ✅ ALLOWED
  // TODO(REPARTIR-123): Implement work-stealing with lock-free deques
  ```
- [ ] **Documentation**: New public API has rustdoc comments with examples?
- [ ] **Simplification**: Can this code be simpler? (Reduce cyclomatic complexity)
  - Use `pmat complexity` to analyze
  - Refactor functions with complexity > 10
- [ ] **TDG Score**: Run `pmat tdg` before/after PR. Score must not decrease.

#### Protocol E: Performance Justification (for `unsafe` or complex optimizations)

**Checklist**:
- [ ] **Baseline Benchmark**: Establish safe baseline performance
  ```bash
  cargo bench --bench scheduler_bench > baseline.txt
  ```
- [ ] **Optimized Benchmark**: Run optimized version (with `unsafe` or complexity)
  ```bash
  cargo bench --bench scheduler_bench > optimized.txt
  ```
- [ ] **Statistical Significance**: Use `criterion` to verify improvement is not noise
  - Require: `p < 0.05` and effect size `> 20%`
- [ ] **Cost-Benefit**: Document maintenance cost (complexity, unsafety) vs. performance gain

**Example**:
> **Optimization**: Replace `Vec::push` with `ptr::write` in hot path (`scheduler.rs:234`)
> **Baseline**: 1.2 μs per task submission
> **Optimized**: 0.9 μs per task submission
> **Improvement**: 25% (p = 0.001, 1000 iterations)
> **Safety**: Verified with Miri, capacity checked before write
> **Approval**: ✅ Justified

### 12.3 Phase 3: Five Whys Root Cause Analysis (Post-Incident)

**Philosophy**: Kaizen requires learning from defects, not just fixing symptoms.

**Trigger**: Any production incident, critical bug, or CI failure that escaped Phase 1/2.

**Protocol**:
1. **Assemble Team**: Developer who wrote code + reviewer + domain expert
2. **Ask "Why?" 5 Times**: Drill down to systemic root cause
3. **Document**: Record in issue tracker (REPARTIR-XXX)
4. **Implement Countermeasure**: Add new quality gate to prevent recurrence

#### Case Study: Panic in Distributed Scheduler

**Incident**: Production scheduler panicked with "index out of bounds" in worker assignment.

**Five Whys Session**:
```
1. Why did it panic?
   → Index `worker_id` was out of bounds for `workers` vector.

2. Why was the index out of bounds?
   → A worker disconnected, shrinking the vector, but a queued task still referenced the old index.

3. Why did the task reference an invalid index?
   → Worker IDs are assigned sequentially (0, 1, 2...). When worker 1 disconnects, IDs become [0, 2], but task queue still had `worker_id: 1`.

4. Why don't we use stable IDs?
   → We used `Vec` index as ID for performance (O(1) access). Didn't anticipate churn.

5. Why (ROOT CAUSE)?
   → We prioritized micro-optimization (Vec index) over correctness (stable IDs). No type-level enforcement that IDs are valid.
```

**Countermeasure (Kaizen)**:
```rust
// BEFORE: Stringly-typed (index as ID, type-unsafe)
type WorkerId = usize;
let workers: Vec<Worker> = vec![...];
let task_assignment: WorkerId = 1;  // ❌ Can become invalid

// AFTER: Type-safe, stable IDs
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
struct WorkerId(Uuid);

let workers: HashMap<WorkerId, Worker> = HashMap::new();
let task_assignment: WorkerId = WorkerId(Uuid::new_v4());  // ✅ Always valid or lookup returns None
```

**Quality Gate Addition**: Add property-based test to CI:
```rust
#[proptest]
fn worker_disconnect_does_not_invalidate_assignments(
    ops: Vec<WorkerOp>  // Enum: Connect | Disconnect | AssignTask
) {
    let mut scheduler = Scheduler::new();
    for op in ops {
        match op {
            WorkerOp::Connect(id) => scheduler.add_worker(id),
            WorkerOp::Disconnect(id) => scheduler.remove_worker(id),
            WorkerOp::AssignTask(id) => {
                // This should never panic, either succeed or return None
                let _ = scheduler.assign_task(id);
            }
        }
    }
    // Invariant: No panics occurred
}
```

**Documentation**: Add to `docs/postmortems/REPARTIR-042-worker-id-panic.md` for team learning.

## 13. Certeza Integration: Three-Tiered Testing

Repartir adopts the **certeza** framework's tiered testing methodology, balancing rigor with developer productivity.

### 13.1 Tier 1: ON-SAVE (Sub-Second Feedback Loop)

**Goal**: Enable "flow state" during development. Instant feedback prevents context switching.

**Scope**:
- Unit tests (lib and module tests)
- `cargo check` (borrow checker)
- `cargo clippy` (subset: only fast lints)
- Focused property tests (small `PROPTEST_CASES`)

**Execution**:
```bash
# Run in IDE on file save (via rust-analyzer)
cargo test --lib
cargo clippy --no-deps
```

**Performance Target**: < 3 seconds total (to maintain flow)

**Trade-off**: Not exhaustive, misses integration issues, but prevents "broken window" syndrome (accumulated small errors).

### 13.2 Tier 2: ON-COMMIT (1-5 Minute Gate)

**Goal**: Pre-commit hook enforcement. Comprehensive but reasonable wait time.

**Scope**:
- Full property-based test suite (`PROPTEST_CASES=1000`)
- Integration tests (`cargo test --all`)
- Coverage analysis (`cargo llvm-cov`, target ≥95%)
- Documentation tests (`cargo test --doc`)
- PMAT gates (TDG, SATD, complexity)
- Security audit (`cargo audit`, `cargo deny`)

**Execution**:
```bash
# Installed as git pre-commit hook
make tier2

# Or manually
cargo test --all-features
cargo llvm-cov --all-features --lcov
pmat tdg --fail-on-regression
pmat satd --max-satd 0
cargo audit
```

**Performance Target**: 1-5 minutes (acceptable for commit ceremony)

**CI**: Runs on every push to feature branch.

### 13.3 Tier 3: ON-MERGE/NIGHTLY (Hours)

**Goal**: Exhaustive verification before merging to `main`. Nightly for continuous regression detection.

**Scope**:
- Mutation testing (`cargo-mutants`, target ≥80% score)
- Formal verification (`kani`, for critical `unsafe` blocks)
- Extended fuzzing (`cargo fuzz`, 10-60 minutes)
- Performance benchmarks (`cargo bench`, with regression detection)
- Chaos engineering (`certeza chaos-test`, resource limits, signal injection)

**Execution**:
```bash
# Manual (developer laptop)
make tier3

# CI: On PR to main
cargo mutants --no-times
cargo fuzz run fuzz_scheduler -- -max_total_time=600
cargo bench --bench all
```

**Performance Target**: 1-6 hours (run overnight or in CI)

**CI**: Required gate for merging to `main`. Nightly cron job for continuous health.

### 13.4 Mutation Testing: The Ultimate Test of Tests

**Why Mutation Testing?**
Traditional coverage metrics (line coverage, branch coverage) measure what code is **executed**, not whether tests actually **validate** behavior. Mutation testing answers: "If I break the code, do the tests catch it?"

**How It Works**:
1. `cargo-mutants` modifies source code (e.g., change `<` to `<=`, `+` to `-`)
2. Runs test suite against each "mutant"
3. **Caught**: Test fails (good! test is effective)
4. **Survived**: Test passes despite broken code (bad! test is weak)

**Mutation Score**: `caught / (caught + survived)` (target: ≥80%)

**Example**:
```rust
// Original code
pub fn schedule_task(&self, priority: u8) -> Result<()> {
    if priority < 10 {  // ← cargo-mutants will mutate this
        self.high_priority_queue.push(task)
    } else {
        self.low_priority_queue.push(task)
    }
}

// Mutant 1: Change `<` to `<=`
if priority <= 10 { ... }  // ← Will tests catch this?

// Mutant 2: Change `<` to `>`
if priority > 10 { ... }  // ← Will tests catch this?
```

**Strong Test** (catches mutants):
```rust
#[test]
fn test_priority_boundary() {
    let scheduler = Scheduler::new();

    scheduler.schedule_task(9);  // Should go to high-priority
    assert_eq!(scheduler.high_priority_queue.len(), 1);

    scheduler.schedule_task(10);  // Should go to low-priority
    assert_eq!(scheduler.low_priority_queue.len(), 1);  // ← Catches `<` → `<=` mutant
}
```

**Weak Test** (mutants survive):
```rust
#[test]
fn test_priority_scheduling() {
    let scheduler = Scheduler::new();
    scheduler.schedule_task(5);
    // ❌ No assertion! Mutants survive because test doesn't verify behavior
}
```

**Configuration** (`.cargo-mutants.toml`):
```toml
# Exclude test utilities from mutation (they're not production code)
exclude_files = ["tests/**", "benches/**"]

# Timeout multiplier (mutants should fail fast)
timeout_multiplier = 2.0

# Minimum test pass rate to avoid flaky tests
minimum_test_pass_rate = 0.95
```

### 13.5 Formal Verification with Kani (Critical Paths Only)

**When to Use Formal Verification:**
- Functions with `unsafe` blocks (memory safety proofs)
- Security-critical logic (authentication, encryption)
- Scheduler invariants (no deadlocks, starvation freedom)

**Example: Work-Stealing Deque Invariant**:
```rust
#[cfg(kani)]
#[kani::proof]
fn verify_deque_capacity_invariant() {
    let mut deque: WorkStealingDeque<Task> = kani::any();

    kani::assume(deque.len <= deque.capacity);  // Precondition

    // Perform operations
    if kani::any() {
        deque.push(kani::any());
    } else {
        let _ = deque.pop();
    }

    // Postcondition: Invariant preserved
    assert!(deque.len <= deque.capacity);
}
```

**Execution**:
```bash
cargo kani --harness verify_deque_capacity_invariant
```

**Trade-off**: Formal verification is expensive (hours for complex code). Reserve for highest-risk 5-10% of codebase.

### 13.6 Chaos Engineering (renacer Integration)

**Purpose**: Test resilience under resource exhaustion and failure injection.

**Scenarios**:
1. **Memory Pressure**: Run scheduler with 64MB limit (should degrade gracefully, not OOM crash)
2. **CPU Throttling**: Limit CPU to 25%, verify work-stealing adapts
3. **Signal Injection**: Send `SIGTERM` to worker mid-task, verify coordinator detects failure

**Configuration**:
```rust
use certeza::chaos::ChaosConfig;

#[test]
#[ignore]  // Run explicitly with `cargo test --ignored chaos_`
fn chaos_test_scheduler_under_memory_pressure() {
    let config = ChaosConfig::new()
        .with_memory_limit(64 * 1024 * 1024)  // 64 MB
        .with_cpu_limit(0.5)  // 50% CPU
        .with_timeout(Duration::from_secs(30))
        .build();

    let result = config.run_chaos(|| {
        let scheduler = Scheduler::new();
        // Simulate high load
        for i in 0..10000 {
            scheduler.submit(Task::new(i));
        }
    });

    // Should complete without crashing, but may slow down
    assert!(result.is_ok());
}
```

**CI Integration**: Run chaos tests in Tier 2 (fast presets) and Tier 3 (aggressive presets).

## 14. Roadmap

### v1.0: Sovereign Foundation (Current Scope)

**Core Distributed Computing:**
- [x] Architecture specification (Iron Lotus Enhanced)
- [ ] CPU executor with work-stealing scheduler (Blumofe-Leiserson algorithm)
- [ ] GPU executor with wgpu + rust-gpu (pure Rust compute pipeline)
- [ ] Remote executor with TCP transport (REQ/REP pattern, rustls TLS)
- [ ] Basic fault tolerance (retry with exponential backoff, heartbeat-based failure detection)
- [ ] High-level API (Pool, Task, Builder pattern)
- [ ] Integration with trueno (SIMD) and aprender (ML workloads)

**Iron Lotus Quality Gates:**
- [ ] Comprehensive test suite (≥95% line coverage, ≥80% mutation score)
- [ ] Certeza integration (Tier 1/2/3 testing pipeline)
- [ ] Property-based testing (proptest: scheduler invariants, work-stealing correctness)
- [ ] CI/CD with Jidoka gates (rustc, clippy, rustfmt, coverage, docs, SATD, complexity)
- [ ] Pre-commit hooks (automated quality enforcement)
- [ ] Miri verification for all `unsafe` blocks
- [ ] Formal verification (Kani) for critical scheduler invariants

**Sovereign AI:**
- [ ] Zero C/C++ dependencies (100% pure Rust stack)
- [ ] Binary signing infrastructure (ed25519 signatures)
- [ ] Supply chain security (cargo-deny, dependency pinning, Cargo.lock enforcement)
- [ ] Deterministic builds via bashrs
- [ ] Security audit tooling (cargo-audit, vulnerability scanning)
- [ ] Transparency tooling (AST inspection via pmat, execution tracing)

### v1.1: Production Hardening

**Advanced Messaging:**
- [ ] PUB/SUB pattern for control plane (configuration updates, shutdown signals)
- [ ] PUSH/PULL pattern for pipelined workflows
- [ ] Broker mode for centralized coordination (optional RabbitMQ-style architecture)
- [ ] Peer-to-peer mode (ZeroMQ-style, no central coordinator)

**Performance & Observability:**
- [ ] Performance benchmarks vs Ray/Dask (latency, throughput, scalability)
- [ ] Criterion-based regression detection (fail CI on >10% slowdown)
- [ ] Tracing infrastructure (opentelemetry-compatible, privacy-preserving)
- [ ] Metrics export (Prometheus-compatible, resource utilization tracking)
- [ ] Documentation and comprehensive examples

**Quality Refinement:**
- [ ] Mutation score ≥85% (stretch goal from 80%)
- [ ] Chaos engineering (certeza chaos-test with aggressive presets)
- [ ] Extended fuzzing (72-hour fuzzing campaigns for scheduler + executor)
- [ ] Documentation coverage ≥95% (stretch goal from 90%)

### v2.0: Data Integration & Advanced ML

**trueno-db Integration:**
- [ ] Distributed state management (checkpoint storage)
- [ ] Data-locality aware scheduling (prefer workers with cached data)
- [ ] Persistent task queues (survive coordinator restart)
- [ ] Distributed KV store for configuration

**Advanced ML Patterns:**
- [ ] Pipeline parallelism (model split across GPUs)
- [ ] Tensor parallelism (large tensors sharded across devices)
- [ ] Ring-allreduce for gradient aggregation (Horovod-style)
- [ ] Distributed inference with batching
- [ ] Hugging Face Accelerate-compatible API

**Stream Processing:**
- [ ] Continuous task streams (not just batch)
- [ ] Backpressure handling
- [ ] Windowing and aggregation primitives

### v3.0: Enterprise & Cloud

**High-Performance Networking:**
- [ ] RDMA support for low-latency (via `rdma-rs`, pure Rust)
- [ ] Zero-copy data transfer for large tensors
- [ ] GPU Direct RDMA (peer-to-peer GPU transfers)

**Multi-Tenancy & Isolation:**
- [ ] Namespace isolation (per-tenant scheduler instances)
- [ ] Resource quotas (CPU, memory, GPU limits per tenant)
- [ ] cgroups integration via Rust bindings (process isolation)
- [ ] Secure multi-tenant binary execution (sandboxing)

**Operational Excellence:**
- [ ] Dynamic scaling (auto-add/remove workers based on load)
- [ ] Kubernetes operator for cloud deployment
- [ ] Helm charts for production deployments
- [ ] Health check endpoints (liveness, readiness probes)
- [ ] Graceful shutdown with task migration
- [ ] Hot-reload for configuration changes

**Compliance & Governance:**
- [ ] Audit logging (tamper-proof logs for regulatory compliance)
- [ ] Role-based access control (RBAC for task submission)
- [ ] Cryptographic task verification (end-to-end provenance)
- [ ] FIPS 140-2 compliance mode (cryptographic algorithms)

## 12. Academic Foundations

This specification is grounded in peer-reviewed research:

**[1] Rust Safety and Performance:**
Jung, R., Jourdan, J., Krebbers, R., & Dreyer, D. (2017). "RustBelt: Securing the Foundations of the Rust Programming Language." *Proceedings of the ACM on Programming Languages*, 2(POPL), 1-34. https://doi.org/10.1145/3158154

**[2] Message-Oriented Middleware:**
Hintjens, P. (2013). "ZeroMQ: Messaging for Many Applications." *O'Reilly Media*. (Industry reference, widely cited in distributed systems research)

Eugster, P. T., Felber, P. A., Guerraoui, R., & Kermarrec, A. M. (2003). "The Many Faces of Publish/Subscribe." *ACM Computing Surveys*, 35(2), 114-131. https://doi.org/10.1145/857076.857078

**[3] Work-Stealing Schedulers:**
Blumofe, R. D., & Leiserson, C. E. (1999). "Scheduling Multithreaded Computations by Work Stealing." *Journal of the ACM*, 46(5), 720-748. https://doi.org/10.1145/324133.324234

Chase, D., & Lev, Y. (2005). "Dynamic Circular Work-Stealing Deque." *Proceedings of the 17th Annual ACM Symposium on Parallelism in Algorithms and Architectures*, 21-28. https://doi.org/10.1145/1073970.1073974

**[4] Fault Tolerance in Distributed Systems:**
Chandra, T. D., & Toueg, S. (1996). "Unreliable Failure Detectors for Reliable Distributed Systems." *Journal of the ACM*, 43(2), 225-267. https://doi.org/10.1145/226643.226647

Alvisi, L., & Marzullo, K. (1998). "Message Logging: Pessimistic, Optimistic, Causal, and Optimal." *IEEE Transactions on Software Engineering*, 24(2), 149-159. https://doi.org/10.1109/32.666828

**[5] GPU Computing and GPGPU:**
Nickolls, J., Buck, I., Garland, M., & Skadron, K. (2008). "Scalable Parallel Programming with CUDA." *Queue*, 6(2), 40-53. https://doi.org/10.1145/1365490.1365500

Stratton, J. A., Rodrigues, C., Sung, I. J., Obeid, N., Chang, L. W., Anssari, N., ... & Hwu, W. M. W. (2012). "Parboil: A Revised Benchmark Suite for Scientific and Commercial Throughput Computing." *Center for Reliable and High-Performance Computing*, 127.

**[6] Distributed Task Scheduling:**
Isard, M., Budiu, M., Yu, Y., Birrell, A., & Fetterly, D. (2007). "Dryad: Distributed Data-Parallel Programs from Sequential Building Blocks." *Proceedings of the 2nd ACM SIGOPS/EuroSys European Conference on Computer Systems*, 59-72. https://doi.org/10.1145/1272996.1273005

**[7] Distributed Machine Learning:**
Sergeev, A., & Del Balso, M. (2018). "Horovod: Fast and Easy Distributed Deep Learning in TensorFlow." *arXiv preprint arXiv:1802.05799*. https://arxiv.org/abs/1802.05799

Dean, J., Corrado, G., Monga, R., Chen, K., Devin, M., Mao, M., ... & Ng, A. Y. (2012). "Large Scale Distributed Deep Networks." *Advances in Neural Information Processing Systems*, 25, 1223-1231.

**[8] Heterogeneous Computing:**
Mittal, S., & Vetter, J. S. (2015). "A Survey of CPU-GPU Heterogeneous Computing Techniques." *ACM Computing Surveys*, 47(4), 1-35. https://doi.org/10.1145/2788396

## 13. Non-Goals (v1)

**Explicitly Out of Scope:**
- Data storage layer (use trueno-db separately)
- Workflow DSL (users write Rust code)
- Python bindings (Rust-native only)
- Dynamic library loading (security risk, complexity)
- Windows support (Linux/macOS first, Windows later)
- WebAssembly workers (interesting, but v2+)

## 14. Open Questions

**To Resolve in Iteration 2:**
1. Should we support function closures (requires stable ABI)?
2. RDMA for low-latency: worth the complexity?
3. Peer-to-peer vs broker: support both or choose one?
4. GPU device sharing: time-slicing vs exclusive access?
5. Container integration: Docker/Podman support?
6. Observability: custom metrics or OpenTelemetry?

## 15. References

See section 12 for full citation list. Additional references:

- **RabbitMQ Architecture:** Videla, A., & Williams, J. W. (2012). *RabbitMQ in Action*. Manning Publications.
- **Hugging Face Transformers:** Wolf, T., et al. (2020). "Transformers: State-of-the-Art Natural Language Processing." *Proceedings of the 2020 Conference on Empirical Methods in Natural Language Processing: System Demonstrations*, 38-45.
- **Apache Ray:** Moritz, P., et al. (2018). "Ray: A Distributed Framework for Emerging AI Applications." *13th USENIX Symposium on Operating Systems Design and Implementation*, 561-577.

---

## Appendix A: Comparison with Existing Systems

| Feature                | Repartir (v1) | Ray       | Dask      | Arrow Flight |
|------------------------|---------------|-----------|-----------|--------------|
| Language               | Rust          | Python    | Python    | C++          |
| C Dependencies         | Zero          | Many      | Some      | Many         |
| GPU Support            | wgpu          | Limited   | No        | No           |
| Binary Execution       | Yes           | No        | No        | No           |
| Work Stealing          | Yes           | No        | Yes       | N/A          |
| Fault Tolerance        | Yes           | Yes       | Limited   | No           |
| Message Overhead       | <1μs          | ~100μs    | ~1ms      | ~10μs        |
| Min Dependency Count   | <15           | 50+       | 30+       | 20+          |

## Appendix B: Example Use Cases

**Scientific Computing:**
- Embarrassingly parallel simulations (Monte Carlo, parameter sweeps)
- Distribute heavy numerical computations across HPC cluster
- GPU-accelerated scientific kernels (FFT, linear algebra via trueno)

**Machine Learning:**
- Distributed training of neural networks (via aprender)
- Hyperparameter search across GPU cluster
- Model inference at scale (Hugging Face-style)

**Data Processing:**
- ETL pipelines with heterogeneous stages (CPU pre-processing, GPU transform)
- Real-time stream processing (combine with trueno-db in v2)

**Agent Orchestration:**
- Distribute autonomous agents across cluster (via paiml-mcp-agent-toolkit)
- Self-organizing task allocation

**Sovereign National Infrastructure:**
- Critical infrastructure resilience (energy grid simulation, defense systems)
- Independent AI capability development (no reliance on foreign cloud providers)
- Regulatory compliance (GDPR, data sovereignty laws)
- Academic research with reproducible, auditable infrastructure

## Conclusion: The Path to Digital Sovereignty

### The Strategic Imperative

The trajectory of artificial intelligence and distributed computing is approaching an inflection point. As AI systems become increasingly central to national infrastructure, economic competitiveness, and security, the inability to independently audit, control, and secure these systems represents an unacceptable strategic vulnerability. The current paradigm—Python facades over C++ substrates, opaque binary blobs, and sprawling supply chains—is fundamentally incompatible with the requirements of digital sovereignty.

**Repartir represents a different path**: a clean-slate, first-principles approach to distributed computing that prioritizes auditability, memory safety, and supply chain independence above all else. By adopting the **Iron Lotus Framework**—the rigorous application of Toyota Production System principles to systems programming—we ensure that this foundation is not only secure by design but also sustainable through continuous improvement.

### The Toyota Way Applied to Systems Programming

The genius of the Toyota Production System lies not in its individual techniques but in its coherent philosophy: **quality is not inspected in; it is built in**. The Iron Lotus Framework operationalizes this insight for software:

- **Jidoka (Automation with Human Touch)**: The CI pipeline acts as an automated immune system, pulling the Andon cord on any defect. But unlike blind automation, it preserves the crucial role of human judgment in architectural review.

- **Genchi Genbutsu (Go and See)**: Transparency is non-negotiable. Every execution path—from user API to GPU shader—must be traceable through auditable Rust source code. No black boxes, no trust-us-it-works claims.

- **Kaizen (Continuous Improvement)**: Technical debt is not inevitable; it is a choice. The ratchet effect ensures that every commit leaves the codebase better than it found it. Mutation testing and formal verification raise the bar continuously.

- **Muda (Waste Elimination)**: Context switching between languages, debugging FFI boundaries, chasing memory corruption—these are wastes. A pure Rust stack eliminates them entirely.

### The Certeza Advantage: Asymptotic Test Effectiveness

Traditional testing approaches suffer from diminishing returns: each additional test provides less confidence than the last. The **certeza** framework breaks this ceiling through a tiered approach that balances rigor with productivity:

- **Tier 1 (Sub-Second)**: Enables flow state during development
- **Tier 2 (Minutes)**: Comprehensive pre-commit gate with 95% coverage
- **Tier 3 (Hours)**: Mutation testing and formal verification for exhaustive validation

By targeting **95% line coverage and 80% mutation score**, we approach asymptotic test effectiveness—the practical maximum confidence achievable without prohibitive cost. This is not theoretical purity; it is empirically validated by certeza's own 97.7% mutation score on its reference implementation.

### Why Pure Rust, Why Now

The decision to mandate **100% Rust, zero C/C++** is not ideological purity—it is strategic necessity:

1. **Memory Safety = National Security**: The NSA/CISA joint guidance is unambiguous: memory-unsafe languages are the primary vector for critical vulnerabilities [8]. A sovereign stack cannot be built on a foundation of undefined behavior.

2. **RustBelt Formal Verification**: Unlike C++, Rust's safety guarantees are mathematically proven [18]. This is not marketing; it is peer-reviewed computer science.

3. **Supply Chain Resilience**: Every C++ dependency is a potential attack vector. The 2024 XZ Utils backdoor demonstrated that even mature, widely-audited projects are vulnerable. Pure Rust eliminates the FFI attack surface entirely.

4. **Performance Parity**: Empirical research shows Rust matches or exceeds C++ in both speed and energy efficiency [24]. The performance argument for C++ is a myth.

### The Cost of Sovereignty

This specification does not minimize the challenge ahead. Rebuilding the distributed computing stack in pure Rust is a multi-year, resource-intensive endeavor. The temptation to wrap existing C++ libraries—PyTorch, MPI, CUDA—will be persistent.

**But the alternative is unacceptable**: a generation of AI infrastructure built on sand, where a single buffer overflow can compromise national systems, where supply chain attacks go undetected, where "sovereignty" is merely a licensing agreement with a foreign corporation.

The **Iron Lotus Framework** provides a path through this challenge:
- Start small: CPU-only executor with work-stealing (v1.0)
- Prove the model: Benchmark against Ray/Dask, achieve competitive performance (v1.1)
- Expand strategically: Add GPU and remote execution incrementally (v1.0-v2.0)
- Sustain through discipline: Kaizen ensures quality never regresses

### Call to Action

This specification is a blueprint, not a product. Realizing the vision of repartir requires:

1. **Institutional Commitment**: Organizations pursuing digital sovereignty must invest in first-principles engineering, not quick wins.

2. **Talent Development**: The Rust ecosystem needs distributed systems experts. Establish "Rust Dojos" to train the next generation.

3. **Open Collaboration**: While the goal is sovereignty, the means is open source. Contribute to trueno, aprender, certeza, and the broader Rust-for-AI ecosystem.

4. **Rejection of Incrementalism**: Wrapping C++ in safe APIs is not sovereignty; it is security theater. The clean-slate approach is the only viable path.

### Final Reflection: Turtles All the Way Down

Peisert et al. titled their clean-slate security paper "Turtles All the Way Down" [13]—a reference to the infinite regress problem in epistemology. In the context of Sovereign AI, the metaphor is apt: if the foundation is insecure, no amount of hardening at higher layers matters.

**Repartir is our turtle**: a solid, auditable, memory-safe foundation for distributed AI. Built on Rust's provable safety guarantees, disciplined by the Toyota Way, and validated by certeza's exhaustive testing, it represents a genuine alternative to the black-box status quo.

The path is difficult. The payoff is sovereignty.

---

**Document Status:** DRAFT - Iteration 2 (Iron Lotus Enhanced)
**Quality Framework:** Iron Lotus + Certeza
**Last Updated:** 2025-11-22
**Next Review:** After v1.0 CPU executor implementation
**Feedback:** Issues and PRs welcome at github.com/paiml/repartir
**Citation:** For academic use, cite as: "Repartir: A Sovereign AI-Grade Distributed Computing Framework in Pure Rust (Iron Lotus Specification v1.0-draft)"

**Acknowledgments:**
- **The Iron Lotus Framework**: Toyota Production System adapted for systems programming
- **Certeza Project**: Asymptotic test effectiveness methodology
- **PAIML Stack**: trueno, aprender, trueno-db, renacer, paiml-mcp-agent-toolkit, bashrs
- **Rust Community**: rust-gpu, wgpu, tokio, and the broader ecosystem
- **Academic Foundations**: The researchers whose work underpins this specification (see Section 12)
