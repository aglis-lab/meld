use std::{env::args, fs};

fn main() {
    let mut path = "samples/index.bhtml".to_string();
    if args().count() == 2 {
        path = args().nth(1).unwrap();
    }

    let content = fs::read(path).unwrap();
    let program = meld::evaluator::Program::new(&content).unwrap();
    let mut eval = meld::evaluator::Evaluator::new(
        &program,
        meld::evaluator::EvaluatorConfig {
            ignore_missing_variables: true,
        },
    );

    let payload_str = r#"{
        "username": "alice",
        "email": "alice@example.com",
        "age": 30,
        "nested": {
            "field1": "nested_value",
            "field2": 42,
            "field3": true
        },
        "items": [
            {"name": "Widget A", "price": 9.99},
            {"name": "Widget B", "price": 14.99},
            {"name": "Widget C", "price": 19.99}
        ],
        "description": "A great collection",
        "firstName": "John",
        "lastName": "Doe",
        "categories": [
            {
                "title": "Widgets",
                "items": ["Widget A", "Widget B", "Widget C"]
            },
            {
                "title": "Gadgets",
                "items": ["Gadget X", "Gadget Y"]
            }
        ]
    }"#;
    
    let payload = serde_json::from_str(payload_str).unwrap();

    if let Err(err) = eval.run(&payload) {
        eprintln!("Error: {}", err);
    } else {
        println!("Output: {}", eval.output());
    }
}
