use std::{env::args, fs};

fn main() {
    let mut path = "samples/index.bhtml".to_string();
    if args().count() == 2 {
        path = args().nth(1).unwrap();
    }

    let ext = path
        .rsplit('.')
        .next()
        .unwrap_or_else(|| panic!("Failed to get file extension for {}", path));

    let output_path = path[..path.len() - ext.len()].to_string() + "out.html";

    let (program, payload) = create_evaluator(&path);
    let mut eval = meld::evaluator::Evaluator::new(
        &program,
        meld::evaluator::EvaluatorConfig {
            ignore_missing_variables: true,
        },
    );
    eval.run(&payload).expect("failed to run the evaluator");
    let content = eval.output();

    fs::write(&output_path, content).expect("Failed to write output");
}

fn create_evaluator(html_path: &str) -> (meld::evaluator::Program, serde_json::Value) {
    // Load the compiled binary program
    let program_bytes = fs::read(html_path).expect("Failed to read program file");

    let compiled_program =
        meld::evaluator::Program::new(&program_bytes).expect("Failed to create program");

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
