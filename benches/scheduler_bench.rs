#![allow(clippy::unwrap_used, clippy::expect_used)]
//! Scheduler benchmarks (Iron Lotus Framework: Performance validation)

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use repartir::scheduler::Scheduler;
use repartir::task::{Backend, Task};

fn bench_scheduler_submit(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    c.bench_function("scheduler_submit", |b| {
        b.iter(|| {
            rt.block_on(async {
                let scheduler = Scheduler::new();
                let task = Task::builder()
                    .binary("/bin/echo")
                    .arg("test")
                    .backend(Backend::Cpu)
                    .build()
                    .unwrap();

                scheduler.submit(black_box(task)).await.unwrap();
            });
        });
    });
}

criterion_group!(benches, bench_scheduler_submit);
criterion_main!(benches);
