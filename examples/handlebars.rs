use handlebars::{Context, Handlebars, Helper, HelperResult, Output, RenderContext};
use serde_json::{Value, json};
use std::fs;

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

fn main() {
    let mut handlebars = Handlebars::new();

    // Load template from templates/handlebars.html
    let template =
        fs::read_to_string("templates/handlebars.html").expect("Failed to read template file");
    handlebars
        .register_template_string("benchmark", template)
        .expect("Failed to register template");

    // Register custom helpers used in templates/handlebars.html
    handlebars.register_helper("gt", Box::new(helper_gt));
    handlebars.register_helper("gte", Box::new(helper_gte));
    handlebars.register_helper("and", Box::new(helper_and));
    handlebars.register_helper("or", Box::new(helper_or));
    handlebars.register_helper("concat", Box::new(helper_concat));
    handlebars.register_helper("length", Box::new(helper_length));
    handlebars.register_helper("coalesce", Box::new(helper_coalesce));

    // Create sample data
    let data = json!({
        "username": "john_doe",
        "email": "john@example.com",
        "firstName": "John",
        "lastName": "Doe",
        "count": 42,
        "active": true,
        "verified": true,
        "premium": true,
        "vip": false,
        "disabled": false,
        "status": "active",
        "age": 28,
        "balance": 1500.50,
        "description": "Software engineer and tech enthusiast",
        "bio": "Passionate about open source",
        "items": [
            {
                "name": "Laptop",
                "price": 999.99,
                "available": true,
                "stock": 10,
                "tags": ["electronics", "computers"]
            },
            {
                "name": "Mouse",
                "price": 29.99,
                "available": true,
                "stock": 50,
                "tags": ["electronics"]
            },
            {
                "name": "Monitor",
                "price": 299.99,
                "available": false,
                "stock": 0,
                "tags": ["electronics", "displays"]
            }
        ],
        "categories": [
            {
                "title": "Electronics",
                "items": [
                    {
                        "name": "Keyboard",
                        "price": 89.99,
                        "available": true,
                        "stock": 25,
                        "tags": ["electronics"]
                    },
                    {
                        "name": "Headphones",
                        "price": 149.99,
                        "available": true,
                        "stock": 15,
                        "tags": ["electronics", "audio"]
                    }
                ]
            },
            {
                "title": "Accessories",
                "items": [
                    {
                        "name": "USB Cable",
                        "price": 9.99,
                        "available": true,
                        "stock": 100,
                        "tags": ["accessories"]
                    },
                    {
                        "name": "Phone Stand",
                        "price": 19.99,
                        "available": true,
                        "stock": 30,
                        "tags": ["accessories"]
                    }
                ]
            }
        ]
    });

    match handlebars.render("benchmark", &data) {
        Ok(output) => {
            println!("{}", output);
            // Optionally save to file
            std::fs::write("handlebars_output.html", &output).expect("Failed to write output file");
        }
        Err(e) => {
            eprintln!("Template rendering error: {}", e);
        }
    }
}
