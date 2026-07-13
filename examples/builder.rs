use std::fs;

use meld::value::Value;

fn main() {
    let input_file = "templates/comprehensive.html";
    let output_file = "templates/comprehensive.bhtml";
    let eval_file = "templates/comprehensive.out.html";
    let payload_file = "templates/comprehensive.json";

    build_template(input_file, output_file);
    evaluate_template(output_file, eval_file, payload_file);
}

fn build_template(input_file: &str, output_file: &str) {
    let content = fs::read(input_file).unwrap();
    let mut builder = meld::compiler::Builder::new();
    builder.build(&content).unwrap();
    let compiled_content = builder.compile().unwrap();
    fs::write(output_file, &compiled_content).unwrap();
}

fn evaluate_template(input_file: &str, output_file: &str, payload_file: &str) {
    let compiled_content = fs::read(input_file).unwrap();
    let program = meld::runtime::Program::new(&compiled_content).unwrap();
    let mut eval = meld::runtime::Runtime::new(
        &program,
        meld::runtime::EvaluatorConfig {
            ignore_missing_variables: true,
        },
    );
    eval.register_callable_fn("toUpperCase", |value| {
        let s = value.get(0).and_then(|v| v.as_str()).unwrap_or("");
        Value::String(s.to_uppercase())
    });

    let payload = fs::read(payload_file).unwrap();
    let payload: serde_json::Value = serde_json::from_slice(&payload).unwrap();

    eval.run(&payload).unwrap();

    fs::write(output_file, eval.output()).unwrap();
}
