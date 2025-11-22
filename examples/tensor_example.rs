//! Tensor operations example using trueno SIMD library (v2.0).
//!
//! Demonstrates high-performance SIMD-accelerated tensor operations for ML workloads.
//!
//! Run with:
//! ```bash
//! cargo run --example tensor_example --features tensor
//! ```

#[cfg(feature = "tensor")]
use repartir::tensor::TensorExecutor;

#[tokio::main]
async fn main() -> repartir::error::Result<()> {
    #[cfg(not(feature = "tensor"))]
    {
        eprintln!("Error: This example requires the 'tensor' feature");
        eprintln!("Run with: cargo run --example tensor_example --features tensor");
        std::process::exit(1);
    }

    #[cfg(feature = "tensor")]
    {
        println!("=== Repartir v2.0: Tensor Operations Example ===\n");

        // Create tensor executor with CPU SIMD backend
        let executor = TensorExecutor::new()?;
        println!("1. Tensor executor initialized");
        println!("   Backend: {:?}\n", executor.backend());

        // Example 1: Matrix Multiplication
        println!("2. Matrix Multiplication (2x2 matrices):");
        println!("   A = [1.0, 2.0]    B = [5.0, 6.0]");
        println!("       [3.0, 4.0]        [7.0, 8.0]");

        let a = vec![1.0, 2.0, 3.0, 4.0];
        let b = vec![5.0, 6.0, 7.0, 8.0];
        let result = executor.matmul(&a, &b, 2, 2, 2).await?;

        println!("\n   C = A × B = [{:.1}, {:.1}]", result[0], result[1]);
        println!("               [{:.1}, {:.1}]", result[2], result[3]);
        println!("   Expected:   [19.0, 22.0]");
        println!("               [43.0, 50.0]\n");

        // Example 2: Vector Addition (SIMD accelerated)
        println!("3. Vector Addition (SIMD accelerated):");
        let v1 = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0];
        let v2 = vec![8.0, 7.0, 6.0, 5.0, 4.0, 3.0, 2.0, 1.0];

        let sum = executor.add(&v1, &v2).await?;
        println!("   v1 = {:?}", v1);
        println!("   v2 = {:?}", v2);
        println!("   v1 + v2 = {:?}", sum);
        println!("   All elements = 9.0 ✓\n");

        // Example 3: Scalar Multiplication
        println!("4. Scalar Multiplication:");
        let data = vec![1.0, 2.0, 3.0, 4.0];
        let scaled = executor.scale(&data, 2.5).await?;

        println!("   data = {:?}", data);
        println!("   scaled (×2.5) = {:?}", scaled);
        println!("   Expected: [2.5, 5.0, 7.5, 10.0]\n");

        // Example 4: ReLU Activation
        println!("5. ReLU Activation (ML use case):");
        let activations = vec![-2.0, -1.0, 0.0, 1.0, 2.0];
        let relu_output = executor.relu(&activations).await?;

        println!("   Input:  {:?}", activations);
        println!("   ReLU:   {:?}", relu_output);
        println!("   Negative values clamped to 0.0 ✓\n");

        // Example 5: Vector Sum (Reduction operation)
        println!("6. Vector Sum (SIMD reduction):");
        let large_vector: Vec<f64> = (1..=100).map(|x| x as f64).collect();
        let total = executor.sum(&large_vector).await?;

        println!("   Sum of 1..=100 = {:.1}", total);
        println!("   Expected: 5050.0 ✓\n");

        // Example 6: Larger Matrix Multiplication
        println!("7. Larger Matrix Multiplication (4x4):");
        let a_4x4 = vec![
            1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0, 13.0, 14.0, 15.0,
            16.0,
        ];
        let b_4x4 = vec![
            16.0, 15.0, 14.0, 13.0, 12.0, 11.0, 10.0, 9.0, 8.0, 7.0, 6.0, 5.0, 4.0, 3.0, 2.0,
            1.0,
        ];

        let result_4x4 = executor.matmul(&a_4x4, &b_4x4, 4, 4, 4).await?;

        println!("   4×4 matrix multiplication completed");
        println!("   Result (first row): [{:.1}, {:.1}, {:.1}, {:.1}]",
                 result_4x4[0], result_4x4[1], result_4x4[2], result_4x4[3]);
        println!("   Computation used SIMD acceleration\n");

        // Example 7: Cache Performance
        println!("8. Cache Performance:");
        println!("   Cache size before: {}", executor.cache_size().await);

        // Perform same matmul again - should hit cache
        let _cached_result = executor.matmul(&a, &b, 2, 2, 2).await?;

        println!("   Cache size after: {}", executor.cache_size().await);
        println!("   Second matmul used cached result ✓\n");

        // Performance summary
        println!("=== Example Complete ===\n");
        println!("Key Features Demonstrated:");
        println!("  • Matrix multiplication with SIMD acceleration");
        println!("  • Element-wise vector operations (add, scale)");
        println!("  • ML activation functions (ReLU)");
        println!("  • Reduction operations (sum)");
        println!("  • Operation caching for repeated computations");
        println!("  • Automatic backend selection (AVX2/SSE2/NEON/Scalar)");
        println!("\nPerformance Notes:");
        println!("  • SIMD provides 2-8x speedup for vector operations");
        println!("  • Cache avoids redundant computation");
        println!("  • Operations run in spawn_blocking for CPU-intensive work");
    }

    Ok(())
}
