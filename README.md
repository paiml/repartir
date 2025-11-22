# Repartir: Sovereign AI-Grade Distributed Computing

[![CI](https://img.shields.io/badge/CI-Jidoka%20Gates-green)](.github/workflows/jidoka-gates.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-100%25-orange.svg)](https://www.rust-lang.org/)
[![Zero C/C++](https://img.shields.io/badge/C%2FC%2B%2B-0%25-blue.svg)](#)

**Repartir** is a pure Rust library for distributed execution across CPUs, GPUs, and remote machines. Built on the **Iron Lotus Framework** (Toyota Way principles for systems programming) and validated by the **certeza** testing methodology.

## Features

- ✅ **100% Rust, Zero C/C++**: True digital sovereignty through complete auditability
- ✅ **Memory Safety Guaranteed**: Provably safe via RustBelt formal verification
- ✅ **Work-Stealing Scheduler**: Based on Blumofe & Leiserson (1999)
- ✅ **Priority-Based Execution**: High, Normal, and Low priority queues
- ✅ **Fault Tolerance**: Task retry, timeout handling, graceful failure
- ✅ **Supply Chain Security**: Dependency pinning, binary signing, license enforcement
- ✅ **Iron Lotus Quality**: ≥95% coverage target, ≥80% mutation score, formal verification
- ✅ **Certeza Testing**: Three-tiered testing (sub-second → minutes → hours)

## Quick Start

Add to your `Cargo.toml`:

```toml
[dependencies]
repartir = "0.1"
tokio = { version = "1.35", features = ["rt-multi-thread", "macros"] }
```

### Basic Example

```rust
use repartir::{Pool, task::{Task, Backend}};

#[tokio::main]
async fn main() -> repartir::error::Result<()> {
    // Create a pool with 4 CPU workers
    let pool = Pool::builder()
        .cpu_workers(4)
        .build()?;

    // Submit a task
    let task = Task::builder()
        .binary("/bin/echo")
        .arg("Hello from Repartir!")
        .backend(Backend::Cpu)
        .build()?;

    let result = pool.submit(task).await?;

    if result.is_success() {
        println!("Output: {}", result.stdout_str()?.trim());
    }

    pool.shutdown().await;
    Ok(())
}
```

### Run the Example

```bash
cargo run --example hello_repartir
```

### Comprehensive v1.1 Showcase

See all v1.1 features in action:

```bash
# Generate TLS certificates first
./scripts/generate-test-certs.sh ./certs

# Run comprehensive showcase
cargo run --example v1_1_showcase --features full
```

**Demonstrates:**
- ✅ CPU executor with work-stealing (48 workers, Blumofe & Leiserson algorithm)
- ✅ GPU detection (NVIDIA RTX 4090, 2048 compute units)
- ✅ TLS encryption (certificate-based auth, TLS 1.3)
- ✅ Priority scheduling (High/Normal/Low queues)
- ✅ Parallel speedup (3.82x with 4 workers)
- ✅ Fault tolerance (graceful error handling)

## Feature Flags

Repartir supports multiple execution backends via feature flags:

```toml
[dependencies]
# CPU only (default)
repartir = "0.1"

# With GPU support (v1.1+)
repartir = { version = "0.1", features = ["gpu"] }

# With remote execution (v1.1+)
repartir = { version = "0.1", features = ["remote"] }

# With TLS encryption (v1.1+)
repartir = { version = "0.1", features = ["remote-tls"] }

# All features
repartir = { version = "0.1", features = ["full"] }
```

### GPU Executor (v1.1+)

The GPU executor uses [wgpu](https://wgpu.rs/) for cross-platform GPU compute:

```rust
use repartir::executor::gpu::GpuExecutor;
use repartir::executor::Executor;

#[tokio::main]
async fn main() -> repartir::error::Result<()> {
    let executor = GpuExecutor::new().await?;
    println!("GPU: {}", executor.device_name());
    println!("Compute units: {}", executor.capacity());
    Ok(())
}
```

**Supported backends:**
- Vulkan (Linux/Windows/Android)
- Metal (macOS/iOS)
- DirectX 12 (Windows)
- WebGPU (browsers)

**Note (v1.1):** GPU detection and initialization only. Binary task execution on GPU requires compute shader compilation (v1.2+ with rust-gpu).

```bash
cargo run --example gpu_detect --features gpu
```

### TLS Encryption (v1.1+)

Secure remote execution with TLS/SSL encryption using [rustls](https://github.com/rustls/rustls):

```rust
use repartir::executor::tls::TlsConfig;

#[tokio::main]
async fn main() -> repartir::error::Result<()> {
    // Generate test certificates:
    // ./scripts/generate-test-certs.sh ./certs

    let tls_config = TlsConfig::builder()
        .client_cert("./certs/client.pem")
        .client_key("./certs/client.key")
        .server_cert("./certs/server.pem")
        .server_key("./certs/server.key")
        .ca_cert("./certs/ca.pem")
        .build()?;

    println!("TLS enabled!");
    Ok(())
}
```

**Security features:**
- TLS 1.3 end-to-end encryption
- Certificate-based authentication
- Perfect forward secrecy
- MITM attack protection

**Generate test certificates:**
```bash
./scripts/generate-test-certs.sh ./certs
cargo run --example tls_example --features remote-tls
```

⚠️ **WARNING**: The included certificate generator creates self-signed certificates for **TESTING ONLY**. For production, use certificates from a trusted CA (Let's Encrypt, DigiCert, etc.).

### Messaging Patterns (v1.1+)

Advanced messaging for distributed coordination with PUB/SUB and PUSH/PULL patterns:

#### Publish-Subscribe (PUB/SUB)

One publisher broadcasts to multiple subscribers:

```rust
use repartir::messaging::{PubSubChannel, Message};

#[tokio::main]
async fn main() -> repartir::error::Result<()> {
    let channel = PubSubChannel::new();

    // Subscribe to topics
    let mut events = channel.subscribe("events").await;
    let mut alerts = channel.subscribe("alerts").await;

    // Publish messages
    channel.publish("events", Message::text("Task completed")).await?;

    // All subscribers receive broadcast
    if let Some(msg) = events.recv().await {
        println!("Event: {}", msg.as_text()?);
    }

    Ok(())
}
```

**Use cases**: Event notifications, logging, monitoring, real-time updates

#### Push-Pull (PUSH/PULL)

Work distribution with automatic load balancing:

```rust
use repartir::messaging::{PushPullChannel, Message};

#[tokio::main]
async fn main() -> repartir::error::Result<()> {
    let channel = PushPullChannel::new(100);

    // Producers push work
    channel.push(Message::text("Work item 1")).await?;
    channel.push(Message::text("Work item 2")).await?;

    // Consumers pull work (load balanced)
    let work = channel.pull().await;

    Ok(())
}
```

**Use cases**: Work queues, job scheduling, pipeline processing, task distribution

**Run examples:**
```bash
cargo run --example pubsub_example
cargo run --example pushpull_example
```

## Architecture

Repartir follows a clean, layered architecture:

```
┌─────────────────────────────────────┐
│         Pool (High-Level API)       │
├─────────────────────────────────────┤
│  Scheduler (Priority Queue + Work  │
│   Stealing - Blumofe & Leiserson)   │
├─────────────────────────────────────┤
│         Executor Backends           │
│  ┌────────┐  ┌────────┐  ┌────────┐│
│  │  CPU   │  │  GPU   │  │ Remote ││
│  │(v1.0)  │  │(v1.1+) │  │(v1.1+) ││
│  └────────┘  └────────┘  └────────┘│
└─────────────────────────────────────┘
```

## Iron Lotus Framework

Repartir embodies Toyota Production System principles:

### Genchi Genbutsu (現地現物 - "Go and See")
- **Radical Transparency**: Every operation traceable from API → scheduler → executor
- **No Black Boxes**: 100% pure Rust, zero opaque C/C++ libraries
- **AST-Level Inspection**: Code structure visible via pmat

### Jidoka (自働化 - "Automation with Human Touch")
- **Automated Quality Gates**: CI enforces clippy, rustfmt, tests, coverage
- **Andon Cord**: Build fails immediately on any defect
- **No Manual Checks**: Machines verify before humans review

### Kaizen (改善 - "Continuous Improvement")
- **Technical Debt Grading**: TDG score must never decrease
- **Ratchet Effect**: Each PR improves or maintains quality
- **Five Whys**: Root cause analysis for all incidents

### Muda (無駄 - "Waste Elimination")
- **No Overproduction**: Zero YAGNI features
- **No Waiting**: Fast compilation with sccache
- **No Transportation**: Zero-copy data flow, single language
- **No Defects**: EXTREME TDD with mutation testing

## Testing (Certeza Methodology)

Repartir uses a three-tiered testing approach:

### Tier 1: ON-SAVE (Sub-Second)
Fast feedback for flow state:
```bash
make tier1
```
- Unit tests (21 tests)
- `cargo check`
- `cargo clippy`
- `cargo fmt`

**Target**: < 3 seconds

### Tier 2: ON-COMMIT (1-5 Minutes)
Comprehensive pre-commit gate:
```bash
make tier2
```
- All tests (21 unit + 4 property + 4 doc = 29 tests)
- Property-based tests (proptest)
- Coverage analysis (target ≥95%)
- Documentation tests
- Security audit (cargo-audit, cargo-deny)

**Target**: 1-5 minutes

### Tier 3: ON-MERGE (Hours)
Exhaustive validation:
```bash
make tier3
```
- Mutation testing (cargo-mutants, target ≥80%)
- Formal verification (Kani, for critical paths)
- Extended fuzzing
- Performance benchmarks

**Target**: 1-6 hours (run overnight or in CI)

## Test Results (v1.0)

```
✓ 21 unit tests          (0.10s)
✓ 4 property-based tests (1.92s)
✓ 4 documentation tests  (0.23s)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  29 tests PASSED
```

## Sovereign AI Principles

### Digital Sovereignty Requirements
1. **Auditability**: Trace execution from user API → hardware instruction
2. **Supply Chain Independence**: Deterministic rebuild from source
3. **No Foreign Dependencies**: Zero opaque binary blobs
4. **Memory Safety Guarantees**: Provable absence of vulnerabilities

### Supply Chain Security
- **Dependency Pinning**: `Cargo.lock` committed and reviewed
- **License Enforcement**: Only MIT/Apache-2.0/BSD allowed (via cargo-deny)
- **Binary Signing**: ed25519 signatures for distributed binaries
- **Audit Trail**: `cargo tree` logged in CI

### Memory Safety (NSA/CISA Mandate)
Per NSA/CISA joint guidance on memory-safe languages:
- ✅ Rust provides compile-time memory safety guarantees
- ✅ RustBelt formal verification proves soundness
- ✅ `#![deny(unsafe_code)]` in v1.0 (no unsafe code)
- ✅ Eliminates buffer overflows, use-after-free, data races

## Roadmap

### v1.0: Sovereign Foundation (Current)
- ✅ CPU executor with work-stealing scheduler
- ✅ Priority-based task scheduling
- ✅ High-level Pool API
- ✅ Comprehensive testing (29 tests)
- ✅ Iron Lotus quality gates
- ✅ Supply chain security

### v1.1: Production Hardening (Complete)
- ✅ GPU executor skeleton (wgpu detection, v1.2 for rust-gpu compute)
- ✅ Remote executor (TCP transport, length-prefixed bincode protocol)
- ✅ TLS encryption (rustls, certificate-based auth, TLS 1.3)
- ✅ Performance benchmarks vs Ray/Dask (5 benchmark suites)
- ✅ Mutation testing ≥85% (framework + documentation)
- ✅ Comprehensive Makefile (tier1/tier2/tier3, coverage enforcement)
- ✅ bashrs purification (POSIX-compliant shell code)
- ✅ Advanced messaging patterns (PUB/SUB, PUSH/PULL)

### v2.0: Data Integration
- [ ] trueno-db integration (distributed state)
- [ ] Checkpointing to persistent storage
- [ ] Data-locality aware scheduling
- [ ] Advanced ML patterns (pipeline/tensor parallelism)

### v3.0: Enterprise & Cloud
- [ ] RDMA support (low-latency networking)
- [ ] Multi-tenant isolation
- [ ] Kubernetes operator
- [ ] FIPS 140-2 compliance mode

## Contributing

Contributions welcome! Please ensure:

1. All tests pass: `make tier2`
2. Coverage ≥95%: `make coverage` (when configured)
3. Clippy passes: `cargo clippy -- -D warnings`
4. Code formatted: `cargo fmt`
5. No SATD comments (TODO without ticket number)

See [Iron Lotus Code Review Framework](docs/specifications/repartir-distributed-cpu-gpu-data-hpc-spec.md#12-the-iron-lotus-code-review-framework) for detailed guidelines.

## Documentation

- [Full Specification](docs/specifications/repartir-distributed-cpu-gpu-data-hpc-spec.md) (1,500 lines, 10+ academic citations)
- [API Documentation](https://docs.rs/repartir) (run `cargo doc --open`)
- [Examples](examples/) - Hands-on demonstrations

## Academic Foundations

Repartir is grounded in peer-reviewed research:

1. **Jung et al. (2017)** - RustBelt: Formal verification of Rust's safety
2. **Blumofe & Leiserson (1999)** - Provably optimal work-stealing
3. **Chandra & Toueg (1996)** - Unreliable failure detectors
4. **NSA/CISA (2023)** - Memory-safe languages guidance
5. **Pereira et al. (2017)** - Energy efficiency of Rust vs C++

See [specification](docs/specifications/repartir-distributed-cpu-gpu-data-hpc-spec.md#12-academic-foundations) for complete citations.

## Comparison with Existing Systems

| Feature                | Repartir (v1.1) | Ray       | Dask      |
|------------------------|-----------------|-----------|-----------|
| Language               | Rust            | Python    | Python    |
| C Dependencies         | Zero*           | Many      | Some      |
| GPU Support            | Yes (wgpu)      | Limited   | No        |
| Work Stealing          | Yes             | No        | Yes       |
| Fault Tolerance        | Yes             | Yes       | Limited   |
| Memory Safety          | Guaranteed      | Runtime   | Runtime   |
| Binary Execution       | Yes             | No        | No        |
| Remote Execution       | Yes (TCP)       | Yes       | Yes       |

*Note: rustls (used for TLS) currently depends on aws-lc-rs (C). Pure Rust alternatives under evaluation for v1.2+.

## License

MIT License - see [LICENSE](LICENSE) file for details.

## Acknowledgments

- **Iron Lotus Framework**: Toyota Production System for systems programming
- **Certeza Project**: Asymptotic test effectiveness methodology
- **PAIML Stack**: trueno, aprender, paiml-mcp-agent-toolkit, bashrs
- **Rust Community**: rust-gpu, wgpu, tokio, and the broader ecosystem

---

**Built with the Iron Lotus Framework**
*Quality is not inspected in; it is built in.*

