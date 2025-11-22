//! Locality-aware scheduling example (v2.0).
//!
//! Demonstrates how data locality tracking improves distributed task scheduling
//! by minimizing network transfers.
//!
//! Run with:
//! ```bash
//! cargo run --example locality_scheduling --features checkpoint
//! ```

use repartir::scheduler::locality::LocalityScheduler;
use repartir::state::StateStore;
use repartir::task::{Backend, Task};
use repartir::Pool;
use std::path::PathBuf;
use std::time::Duration;

#[tokio::main]
async fn main() -> repartir::error::Result<()> {
    println!("=== Repartir v2.0: Locality-Aware Scheduling Example ===\n");

    // Setup state store and locality scheduler
    let state_dir = PathBuf::from("./tmp/locality_state");
    std::fs::create_dir_all(&state_dir)?;
    let state = StateStore::new(state_dir.clone())?;
    let scheduler = LocalityScheduler::new(state);

    println!("1. Setting up distributed data scenario...");
    println!("   Simulating 3 workers with different data distributions\n");

    // Simulate data distribution across workers
    scheduler
        .state()
        .register_data("tensor_batch_1".to_string(), "worker1".to_string(), 10_000_000)
        .await?;
    scheduler
        .state()
        .register_data("tensor_batch_2".to_string(), "worker1".to_string(), 10_000_000)
        .await?;
    scheduler
        .state()
        .register_data("tensor_batch_2".to_string(), "worker2".to_string(), 10_000_000)
        .await?;
    scheduler
        .state()
        .register_data("tensor_batch_3".to_string(), "worker2".to_string(), 10_000_000)
        .await?;
    scheduler
        .state()
        .register_data("tensor_batch_3".to_string(), "worker3".to_string(), 10_000_000)
        .await?;
    scheduler
        .state()
        .register_data("model_weights".to_string(), "worker3".to_string(), 50_000_000)
        .await?;

    println!("   Data distribution:");
    println!("   - worker1: tensor_batch_1 (10MB), tensor_batch_2 (10MB)");
    println!("   - worker2: tensor_batch_2 (10MB), tensor_batch_3 (10MB)");
    println!("   - worker3: tensor_batch_3 (10MB), model_weights (50MB)\n");

    // Task 1: Requires tensor_batch_1 and tensor_batch_2
    println!("2. Submitting Task 1: Process batch_1 and batch_2");
    let task1 = Task::builder()
        .binary("/bin/echo")
        .arg("Processing batch 1 and 2")
        .backend(Backend::Cpu)
        .data_dependency("tensor_batch_1")
        .data_dependency("tensor_batch_2")
        .build()?;

    let affinity1 = scheduler.calculate_affinity(task1.data_dependencies()).await?;
    println!("   Affinity scores:");
    for (worker, score) in &affinity1 {
        println!("     - {}: {:.1}% of data available", worker, score * 100.0);
    }

    let task1_id = scheduler.submit(task1).await?;
    println!("   ✓ Task {} submitted", task1_id);
    println!("   → Best placement: worker1 (100% locality)\n");

    // Task 2: Requires tensor_batch_2 and tensor_batch_3
    println!("3. Submitting Task 2: Process batch_2 and batch_3");
    let task2 = Task::builder()
        .binary("/bin/echo")
        .arg("Processing batch 2 and 3")
        .backend(Backend::Cpu)
        .data_dependency("tensor_batch_2")
        .data_dependency("tensor_batch_3")
        .build()?;

    let affinity2 = scheduler.calculate_affinity(task2.data_dependencies()).await?;
    println!("   Affinity scores:");
    for (worker, score) in &affinity2 {
        println!("     - {}: {:.1}% of data available", worker, score * 100.0);
    }

    let task2_id = scheduler.submit(task2).await?;
    println!("   ✓ Task {} submitted", task2_id);
    println!("   → Best placement: worker2 (100% locality)\n");

    // Task 3: Requires model_weights only
    println!("4. Submitting Task 3: Load model weights");
    let task3 = Task::builder()
        .binary("/bin/echo")
        .arg("Loading model")
        .backend(Backend::Cpu)
        .data_dependency("model_weights")
        .build()?;

    let affinity3 = scheduler.calculate_affinity(task3.data_dependencies()).await?;
    println!("   Affinity scores:");
    for (worker, score) in &affinity3 {
        println!("     - {}: {:.1}% of data available", worker, score * 100.0);
    }

    let task3_id = scheduler.submit(task3).await?;
    println!("   ✓ Task {} submitted", task3_id);
    println!("   → Best placement: worker3 (100% locality)\n");

    // Task 4: No data dependencies (locality-agnostic)
    println!("5. Submitting Task 4: Compute-only task (no data deps)");
    let task4 = Task::builder()
        .binary("/bin/echo")
        .arg("Pure computation")
        .backend(Backend::Cpu)
        .build()?;

    let affinity4 = scheduler.calculate_affinity(task4.data_dependencies()).await?;
    println!("   Affinity scores: None (no data dependencies)");
    assert!(affinity4.is_empty());

    let task4_id = scheduler.submit(task4).await?;
    println!("   ✓ Task {} submitted", task4_id);
    println!("   → Placement: Any worker (round-robin)\n");

    // Demonstrate transfer savings calculation
    println!("6. Network transfer savings analysis:");
    let data_sizes = vec![10_000_000, 10_000_000]; // 20MB total

    use repartir::scheduler::locality::estimate_transfer_savings;
    let savings1 = estimate_transfer_savings(&affinity1, &data_sizes);
    let savings2 = estimate_transfer_savings(&affinity2, &data_sizes);

    println!("   Task 1 savings: {:.1} MB (100% locality)", savings1 as f64 / 1_000_000.0);
    println!("   Task 2 savings: {:.1} MB (100% locality)", savings2 as f64 / 1_000_000.0);
    println!("   Total network traffic saved: {:.1} MB\n", (savings1 + savings2) as f64 / 1_000_000.0);

    // Execute tasks with actual pool
    println!("7. Executing tasks with CPU pool...");
    let pool = Pool::builder().cpu_workers(2).build()?;

    // Get tasks from scheduler and execute
    for i in 1..=4 {
        if let Some(task) = scheduler.next_task().await {
            let result = pool.submit(task).await?;
            if result.is_success() {
                println!("   ✓ Task {} completed: {}", i, result.stdout_str()?.trim());
                scheduler.store_result(result).await?;
            }
        }
    }

    pool.shutdown().await;
    println!();

    // State store statistics
    let (kv_count, location_count, result_count) = scheduler.state().stats().await;
    println!("8. State store statistics:");
    println!("   - Key-value entries: {}", kv_count);
    println!("   - Data locations tracked: {}", location_count);
    println!("   - Cached results: {}\n", result_count);

    println!("=== Example Complete ===\n");
    println!("Key Takeaways:");
    println!("  • Locality-aware scheduling minimizes network transfers");
    println!("  • Tasks are matched to workers with best data affinity");
    println!("  • 100% locality = zero network transfer for that task");
    println!("  • StateStore tracks data locations across workers");
    println!("  • Affinity scores guide optimal task placement");

    // Cleanup
    std::fs::remove_dir_all("./tmp/locality_state").ok();

    Ok(())
}
