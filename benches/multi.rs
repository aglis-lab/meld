use askama::Template;
use criterion::{BatchSize, Criterion, Throughput, black_box, criterion_group, criterion_main};
use handlebars::{Context, Handlebars, Helper, HelperResult, Output, RenderContext};
use serde_json::Value;
use std::fs;

#[derive(Clone, Debug)]
pub struct Product {
    pub name: String,
    pub price: f64,
    pub available: bool,
    pub featured: bool,
    pub stock: i32,
    pub tags: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct Category {
    pub title: String,
    pub items: Vec<Product>,
}

#[derive(Template)]
#[template(path = "askama.html")]
pub struct BenchmarkTemplate {
    pub username: String,
    pub email: String,
    pub firstName: String,
    pub lastName: String,
    pub count: i32,
    pub active: bool,
    pub verified: bool,
    pub premium: bool,
    pub vip: bool,
    pub disabled: bool,
    pub status: String,
    pub age: i32,
    pub balance: f64,
    pub description: String,
    pub bio: Option<String>,
    pub items: Vec<Product>,
    pub categories: Vec<Category>,
}

const DURATION_SECS: u64 = 20;
const LABEL_SIZE: usize = 1_000;
const SAMPLE_SIZE: usize = 20;
const BENCH_ITERATIONS: [usize; 10] = [
    10_000, 20_000, 30_000, 40_000, 50_000, 60_000, 70_000, 80_000, 90_000, 100_000,
];

fn bench_meld(c: &mut Criterion) {
    let program = create_meld_engine("templates/meld.html");
    let mut eval = meld::runtime::Runtime::new(
        &program,
        meld::runtime::EvaluatorConfig {
            ignore_missing_variables: true,
        },
    );
    let payload = create_payload();

    let mut group = c.benchmark_group("meld");
    group.measurement_time(std::time::Duration::from_secs(DURATION_SECS));
    group.warm_up_time(std::time::Duration::from_secs(3));
    group.sample_size(SAMPLE_SIZE);

    for &val in &BENCH_ITERATIONS {
        let iter_label = val / LABEL_SIZE;
        group.throughput(Throughput::Elements(val as u64));
        group.bench_function(format!("iterations_{}K", iter_label), |b| {
            b.iter(|| {
                for _ in 0..val {
                    let _ = eval.run(black_box(&payload));
                    let _ = eval.output();
                }
            })
        });
    }
    group.finish();
}

fn bench_askama(c: &mut Criterion) {
    let mut group = c.benchmark_group("askama");
    group.measurement_time(std::time::Duration::from_secs(DURATION_SECS));
    group.warm_up_time(std::time::Duration::from_secs(3));
    group.sample_size(SAMPLE_SIZE);

    for &val in &BENCH_ITERATIONS {
        let iter_label = val / LABEL_SIZE;
        group.throughput(Throughput::Elements(val as u64));
        group.bench_function(format!("iterations_{}K", iter_label), |b| {
            b.iter(|| {
                for _ in 0..val {
                    let template = create_askama_template();
                    let _ = template.render().unwrap();
                }
            })
        });
    }
    group.finish();
}

fn bench_handlebars(c: &mut Criterion) {
    let template_str = fs::read_to_string("templates/handlebars.html")
        .expect("Failed to read handlebars template");
    let mut handlebars = handlebars::Handlebars::new();
    let _ = handlebars.register_template_string("template", &template_str);

    // Register custom helpers used in templates/handlebars.html
    handlebars.register_helper("gt", Box::new(helper_gt));
    handlebars.register_helper("gte", Box::new(helper_gte));
    handlebars.register_helper("and", Box::new(helper_and));
    handlebars.register_helper("or", Box::new(helper_or));
    handlebars.register_helper("concat", Box::new(helper_concat));
    handlebars.register_helper("length", Box::new(helper_length));
    handlebars.register_helper("coalesce", Box::new(helper_coalesce));

    let payload = create_payload();

    let mut group = c.benchmark_group("handlebars");
    group.measurement_time(std::time::Duration::from_secs(DURATION_SECS));
    group.warm_up_time(std::time::Duration::from_secs(3));
    group.sample_size(SAMPLE_SIZE);

    for &val in &BENCH_ITERATIONS {
        let iter_label = val / LABEL_SIZE;
        group.throughput(Throughput::Elements(val as u64));
        group.bench_function(format!("iterations_{}K", iter_label), |b| {
            b.iter(|| {
                for _ in 0..val {
                    let _ = handlebars.render("template", black_box(&payload)).unwrap();
                }
            })
        });
    }
    group.finish();
}

fn helper_gt(
    h: &Helper<'_>,
    _: &Handlebars<'_>,
    _: &Context,
    _: &mut RenderContext<'_, '_>,
    out: &mut dyn Output,
) -> HelperResult {
    let left = h.param(0).and_then(|v| v.value().as_f64()).unwrap_or(0.0);
    let right = h.param(1).and_then(|v| v.value().as_f64()).unwrap_or(0.0);

    if left > right {
        out.write("true")?;
    }

    Ok(())
}

fn helper_gte(
    h: &Helper<'_>,
    _: &Handlebars<'_>,
    _: &Context,
    _: &mut RenderContext<'_, '_>,
    out: &mut dyn Output,
) -> HelperResult {
    let left = h.param(0).and_then(|v| v.value().as_f64()).unwrap_or(0.0);
    let right = h.param(1).and_then(|v| v.value().as_f64()).unwrap_or(0.0);

    if left >= right {
        out.write("true")?;
    }

    Ok(())
}

fn helper_and(
    h: &Helper<'_>,
    _: &Handlebars<'_>,
    _: &Context,
    _: &mut RenderContext<'_, '_>,
    out: &mut dyn Output,
) -> HelperResult {
    let left = h
        .param(0)
        .and_then(|v| v.value().as_bool())
        .unwrap_or(false);
    let right = h
        .param(1)
        .and_then(|v| v.value().as_bool())
        .unwrap_or(false);

    if left && right {
        out.write("true")?;
    }

    Ok(())
}

fn helper_or(
    h: &Helper<'_>,
    _: &Handlebars<'_>,
    _: &Context,
    _: &mut RenderContext<'_, '_>,
    out: &mut dyn Output,
) -> HelperResult {
    let left = h
        .param(0)
        .and_then(|v| v.value().as_bool())
        .unwrap_or(false);
    let right = h
        .param(1)
        .and_then(|v| v.value().as_bool())
        .unwrap_or(false);

    if left || right {
        out.write("true")?;
    }

    Ok(())
}

fn helper_concat(
    h: &Helper<'_>,
    _: &Handlebars<'_>,
    _: &Context,
    _: &mut RenderContext<'_, '_>,
    out: &mut dyn Output,
) -> HelperResult {
    let mut combined = String::new();

    for p in h.params() {
        match p.value() {
            Value::String(s) => combined.push_str(s),
            Value::Number(n) => combined.push_str(&n.to_string()),
            Value::Bool(b) => combined.push_str(if *b { "true" } else { "false" }),
            Value::Null => {}
            other => combined.push_str(&other.to_string()),
        }
    }

    out.write(&combined)?;
    Ok(())
}

fn helper_length(
    h: &Helper<'_>,
    _: &Handlebars<'_>,
    _: &Context,
    _: &mut RenderContext<'_, '_>,
    out: &mut dyn Output,
) -> HelperResult {
    let len = h
        .param(0)
        .map(|p| match p.value() {
            Value::Array(arr) => arr.len(),
            Value::String(s) => s.chars().count(),
            Value::Object(map) => map.len(),
            _ => 0,
        })
        .unwrap_or(0);

    out.write(&len.to_string())?;
    Ok(())
}

fn helper_coalesce(
    h: &Helper<'_>,
    _: &Handlebars<'_>,
    _: &Context,
    _: &mut RenderContext<'_, '_>,
    out: &mut dyn Output,
) -> HelperResult {
    let value = h.param(0).map(|p| p.value()).unwrap_or(&Value::Null);
    let fallback = h.param(1).map(|p| p.value()).unwrap_or(&Value::Null);

    let picked = match value {
        Value::Null => fallback,
        Value::String(s) if s.is_empty() => fallback,
        _ => value,
    };

    match picked {
        Value::Null => {}
        Value::String(s) => out.write(s)?,
        other => out.write(&other.to_string())?,
    }

    Ok(())
}

fn create_meld_engine(path: &str) -> meld::runtime::Program {
    let html = fs::read(path).expect("Failed to read template");

    let mut builder = meld::compiler::Builder::new();
    builder.build(&html).expect("Failed to parse template");
    let program = builder.compile().expect("Failed to compile template");

    let compiled_program = meld::runtime::Program::new(&program).expect("Failed to create program");

    compiled_program
}

fn create_askama_template() -> BenchmarkTemplate {
    let products = vec![
        Product {
            name: "Product A".to_string(),
            price: 29.99,
            available: true,
            featured: false,
            stock: 100,
            tags: vec!["tag1".to_string(), "tag2".to_string(), "tag3".to_string()],
        },
        Product {
            name: "Product B".to_string(),
            price: 49.99,
            available: true,
            featured: false,
            stock: 5,
            tags: vec!["tag4".to_string(), "tag5".to_string()],
        },
        Product {
            name: "Product C".to_string(),
            price: 199.99,
            available: false,
            featured: false,
            stock: 0,
            tags: vec!["tag6".to_string()],
        },
    ];

    let categories = vec![
        Category {
            title: "Electronics".to_string(),
            items: vec![
                Product {
                    name: "Laptop".to_string(),
                    price: 999.99,
                    available: true,
                    featured: true,
                    stock: 10,
                    tags: vec!["v1".to_string(), "v2".to_string(), "v3".to_string()],
                },
                Product {
                    name: "Mouse".to_string(),
                    price: 29.99,
                    available: true,
                    featured: false,
                    stock: 50,
                    tags: vec!["wireless".to_string(), "wired".to_string()],
                },
            ],
        },
        Category {
            title: "Accessories".to_string(),
            items: vec![
                Product {
                    name: "USB Cable".to_string(),
                    price: 9.99,
                    available: true,
                    featured: false,
                    stock: 100,
                    tags: vec!["3ft".to_string(), "6ft".to_string(), "10ft".to_string()],
                },
                Product {
                    name: "Screen Protector".to_string(),
                    price: 15.99,
                    available: true,
                    featured: true,
                    stock: 200,
                    tags: vec!["glass".to_string(), "plastic".to_string()],
                },
            ],
        },
    ];

    BenchmarkTemplate {
        username: "john_doe".to_string(),
        email: "john@example.com".to_string(),
        firstName: "John".to_string(),
        lastName: "Doe".to_string(),
        count: 42,
        active: true,
        verified: true,
        premium: true,
        vip: false,
        disabled: false,
        status: "active".to_string(),
        age: 30,
        balance: 1500.0,
        description: "A great user profile".to_string(),
        bio: Some("Software engineer".to_string()),
        items: products,
        categories,
    }
}

fn create_payload() -> serde_json::Value {
    let content = fs::read_to_string("templates/meld.json").expect("Failed to read meld.json");
    let payload: Value = serde_json::from_str(&content).expect("Failed to parse meld.json");
    payload
}

criterion_group!(benches, bench_meld, bench_askama, bench_handlebars);
criterion_main!(benches);
