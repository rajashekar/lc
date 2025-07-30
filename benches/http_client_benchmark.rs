use criterion::{black_box, criterion_group, criterion_main, Criterion};
use lc::http_client::create_optimized_client;
use reqwest::Client;
use std::time::Duration;
use tokio::runtime::Runtime;

fn benchmark_old_client_creation(c: &mut Criterion) {
    c.bench_function("old_client_creation", |b| {
        b.iter(|| {
            black_box(
                Client::builder()
                    .timeout(Duration::from_secs(60))
                    .build()
                    .unwrap()
            )
        })
    });
}

fn benchmark_optimized_client_creation(c: &mut Criterion) {
    c.bench_function("optimized_client_creation", |b| {
        b.iter(|| {
            black_box(create_optimized_client().unwrap())
        })
    });
}

fn benchmark_multiple_requests_old(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    c.bench_function("multiple_requests_old", |b| {
        b.iter(|| {
            rt.block_on(async {
                let mut handles = Vec::new();
                
                for _ in 0..10 {
                    let client = Client::builder()
                        .timeout(Duration::from_secs(60))
                        .build()
                        .unwrap();
                    
                    let handle = tokio::spawn(async move {
                        // Simulate HTTP request overhead without actual network call
                        tokio::time::sleep(Duration::from_millis(1)).await;
                        client
                    });
                    handles.push(handle);
                }
                
                for handle in handles {
                    black_box(handle.await.unwrap());
                }
            })
        })
    });
}

fn benchmark_multiple_requests_optimized(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    c.bench_function("multiple_requests_optimized", |b| {
        b.iter(|| {
            rt.block_on(async {
                let mut handles = Vec::new();
                
                for _ in 0..10 {
                    let client = create_optimized_client().unwrap();
                    
                    let handle = tokio::spawn(async move {
                        // Simulate HTTP request overhead without actual network call
                        tokio::time::sleep(Duration::from_millis(1)).await;
                        client
                    });
                    handles.push(handle);
                }
                
                for handle in handles {
                    black_box(handle.await.unwrap());
                }
            })
        })
    });
}

criterion_group!(
    benches,
    benchmark_old_client_creation,
    benchmark_optimized_client_creation,
    benchmark_multiple_requests_old,
    benchmark_multiple_requests_optimized
);
criterion_main!(benches);