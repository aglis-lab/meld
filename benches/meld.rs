use criterion::{Criterion, Throughput, black_box, criterion_group, criterion_main};
use rand::RngExt;
use std::{fs, vec};

const EVALUATOR_COUNT: usize = 50;
const SAMPLE_SIZE: usize = 20;
const BENCH_ITERATIONS: [usize; 10] = [
    10_000_000_000,
    20_000_000_000,
    30_000_000_000,
    40_000_000_000,
    50_000_000_000,
    60_000_000_000,
    70_000_000_000,
    80_000_000_000,
    90_000_000_000,
    100_000_000_000,
];

fn bench_evaluator(c: &mut Criterion) {
    let (program, payload) = create_evaluator("templates/meld.html");

    let mut group = c.benchmark_group("meld");
    group.measurement_time(std::time::Duration::from_secs(30));
    group.warm_up_time(std::time::Duration::from_secs(3));
    group.sample_size(SAMPLE_SIZE);

    for &val in &BENCH_ITERATIONS {
        group.throughput(Throughput::Elements(val as u64));
        group.bench_function(&format!("iterations_{}M", val / 1_000_000), |b| {
            b.iter_custom(|iters| {
                let start = std::time::Instant::now();
                for _ in 0..iters {
                    let mut eval = meld::runtime::Runtime::new(
                        &program,
                        meld::runtime::EvaluatorConfig {
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
    let (program, payload) = create_evaluator("templates/bench_giant.html");

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
                    let mut eval = meld::runtime::Runtime::new(
                        &program,
                        meld::runtime::EvaluatorConfig {
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

fn bench_evaluator_giant_multi_evaluator(c: &mut Criterion) {
    let instances: vec::Vec<meld::runtime::Program> = (0..EVALUATOR_COUNT)
        .map(|_| create_program("templates/bench_giant.html"))
        .collect();

    let mut group = c.benchmark_group("evaluator_giant_multi_evaluator");
    group.measurement_time(std::time::Duration::from_secs(30));
    group.warm_up_time(std::time::Duration::from_secs(3));
    group.sample_size(SAMPLE_SIZE);
    let mut rng = rand::rng();

    for &val in &BENCH_ITERATIONS {
        group.throughput(Throughput::Elements(val as u64));
        group.bench_function(
            &format!("iterations_multi_eval_{}M", val / 1_000_000),
            |b| {
                b.iter_custom(|iters| {
                    let start = std::time::Instant::now();
                    for _ in 0..iters {
                        let i = rng.random_range(0..EVALUATOR_COUNT);
                        let mut eval = meld::runtime::Runtime::new(
                            &instances[i],
                            meld::runtime::EvaluatorConfig {
                                ignore_missing_variables: true,
                            },
                        );
                        let payload = create_payload();
                        let _ = eval.run(black_box(&payload));
                        let _ = eval.output();
                    }
                    start.elapsed()
                })
            },
        );
    }
    group.finish();
}

fn create_program(path: &str) -> meld::runtime::Program {
    // Load and compile the template
    let html = fs::read(path).expect("Failed to read template");

    let mut builder = meld::compiler::Builder::new();
    builder.build(&html).expect("Failed to parse template");
    let program = builder.compile().expect("Failed to compile template");

    let compiled_program = meld::runtime::Program::new(&program).expect("Failed to create program");

    compiled_program
}

fn create_evaluator(path: &str) -> (meld::runtime::Program, serde_json::Value) {
    // Load and compile the template
    let html = fs::read(path).expect("Failed to read template");

    let mut builder = meld::compiler::Builder::new();
    builder.build(&html).expect("Failed to parse template");
    let program = builder.compile().expect("Failed to compile template");

    let compiled_program = meld::runtime::Program::new(&program).expect("Failed to create program");

    // Create a comprehensive payload
    let payload = create_payload();

    (compiled_program, payload)
}

fn create_payload() -> serde_json::Value {
    let content =
        fs::read_to_string("templates/payload.json").expect("Failed to read payload.json");
    let payload = serde_json::from_str(&content).expect("Failed to parse payload.json");
    payload
}

criterion_group!(
    benches,
    bench_evaluator,
    bench_evaluator_giant,
    bench_evaluator_giant_multi_evaluator
);

criterion_main!(benches);
