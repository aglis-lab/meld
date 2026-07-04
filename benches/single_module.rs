use criterion::{Criterion, Throughput, black_box, criterion_group, criterion_main};
use std::fs;

const SAMPLE_SIZE: usize = 20;
const BENCH_ITERATIONS: [usize; 3] = [10_000_000_000, 20_000_000_000, 40_000_000_000];

fn bench_evaluator(c: &mut Criterion) {
    let (program, payload) = create_evaluator("samples/bench.html");

    let mut group = c.benchmark_group("evaluator");
    group.measurement_time(std::time::Duration::from_secs(30));
    group.warm_up_time(std::time::Duration::from_secs(3));
    group.sample_size(SAMPLE_SIZE);

    for &val in &BENCH_ITERATIONS {
        group.throughput(Throughput::Elements(val as u64));
        group.bench_function(&format!("iterations_{}M", val / 1_000_000), |b| {
            b.iter_custom(|iters| {
                let start = std::time::Instant::now();
                for _ in 0..iters {
                    let mut eval = meld::evaluator::Evaluator::new(
                        &program,
                        meld::evaluator::EvaluatorConfig {
                            ignore_missing_variables: true,
                        },
                    );
                    let _ = eval.run(black_box(&payload));
                    let _ = eval.output();
                }
                start.elapsed()
            })
        });
    }
    group.finish();
}

fn bench_evaluator_giant(c: &mut Criterion) {
    let (program, payload) = create_evaluator("samples/bench_giant.html");

    let mut group = c.benchmark_group("evaluator_giant");
    group.measurement_time(std::time::Duration::from_secs(30));
    group.warm_up_time(std::time::Duration::from_secs(3));
    group.sample_size(SAMPLE_SIZE);

    for &val in &BENCH_ITERATIONS {
        group.throughput(Throughput::Elements(val as u64));
        group.bench_function(&format!("iterations_{}M", val / 1_000_000), |b| {
            b.iter_custom(|iters| {
                let start = std::time::Instant::now();
                for _ in 0..iters {
                    let mut eval = meld::evaluator::Evaluator::new(
                        &program,
                        meld::evaluator::EvaluatorConfig {
                            ignore_missing_variables: true,
                        },
                    );
                    let _ = eval.run(black_box(&payload));
                    let _ = eval.output();
                }
                start.elapsed()
            })
        });
    }
    group.finish();
}

fn create_evaluator(path: &str) -> (meld::evaluator::Program, serde_json::Value) {
    // Load and compile the template
    let html = fs::read(path).expect("Failed to read template");

    let mut builder = meld::builder::Builder::new();
    builder.build(&html).expect("Failed to parse template");
    let program = builder.compile().expect("Failed to compile template");

    let compiled_program =
        meld::evaluator::Program::new(&program).expect("Failed to create program");

    // Create a comprehensive payload
    let payload = serde_json::json!({
        "username": "john_doe",
        "email": "john@example.com",
        "firstName": "John",
        "lastName": "Doe",
        "count": 42,
        "active": true,
        "status": "active",
        "role": "admin",
        "age": 25,
        "balance": 1500,
        "score": 85,
        "attempts": 2,
        "verified": true,
        "premium": true,
        "vip": false,
        "disabled": false,
        "description": "A great user profile",
        "bio": "Software engineer",
        "notes": "",
        "items": [
            {
                "name": "Product A",
                "price": 99.99,
                "available": true,
                "stock": 10,
                "tags": ["tag1", "tag2", "tag3"]
            },
            {
                "name": "Product B",
                "price": 149.99,
                "available": true,
                "stock": 5,
                "tags": ["tag4", "tag5"]
            },
            {
                "name": "Product C",
                "price": 199.99,
                "available": false,
                "stock": 0,
                "tags": ["tag6"]
            }
        ],
        "categories": [
            {
                "title": "Electronics",
                "items": [
                    {
                        "name": "Laptop",
                        "price": 999.99,
                        "featured": true,
                        "variants": ["v1", "v2", "v3"]
                    },
                    {
                        "name": "Mouse",
                        "price": 29.99,
                        "featured": false,
                        "variants": ["wireless", "wired"]
                    }
                ]
            },
            {
                "title": "Accessories",
                "items": [
                    {
                        "name": "USB Cable",
                        "price": 9.99,
                        "featured": false,
                        "variants": ["3ft", "6ft", "10ft"]
                    },
                    {
                        "name": "Screen Protector",
                        "price": 15.99,
                        "featured": true,
                        "variants": ["glass", "plastic"]
                    }
                ]
            }
        ]
    });

    (compiled_program, payload)
}

criterion_group!(benches, bench_evaluator, bench_evaluator_giant);
criterion_main!(benches);
