use askama::Template;

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

fn main() {
    // Create sample data
    let products = vec![
        Product {
            name: "Laptop".to_string(),
            price: 999.99,
            available: true,
            featured: true,
            stock: 10,
            tags: vec!["electronics".to_string(), "computers".to_string()],
        },
        Product {
            name: "Mouse".to_string(),
            price: 29.99,
            available: true,
            featured: false,
            stock: 50,
            tags: vec!["electronics".to_string()],
        },
        Product {
            name: "Monitor".to_string(),
            price: 299.99,
            available: false,
            featured: false,
            stock: 0,
            tags: vec!["electronics".to_string(), "displays".to_string()],
        },
    ];

    let categories = vec![
        Category {
            title: "Electronics".to_string(),
            items: vec![
                Product {
                    name: "Keyboard".to_string(),
                    price: 89.99,
                    available: true,
                    featured: true,
                    stock: 25,
                    tags: vec!["electronics".to_string()],
                },
                Product {
                    name: "Headphones".to_string(),
                    price: 149.99,
                    available: true,
                    featured: false,
                    stock: 15,
                    tags: vec!["electronics".to_string(), "audio".to_string()],
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
                    tags: vec!["accessories".to_string()],
                },
                Product {
                    name: "Phone Stand".to_string(),
                    price: 19.99,
                    available: true,
                    featured: false,
                    stock: 30,
                    tags: vec!["accessories".to_string()],
                },
            ],
        },
    ];

    let template = BenchmarkTemplate {
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
        age: 28,
        balance: 1500.50,
        description: "Software engineer and tech enthusiast".to_string(),
        bio: Some("Passionate about open source".to_string()),
        items: products,
        categories,
    };

    match template.render() {
        Ok(output) => {
            println!("{}", output);
            // Optionally save to file
            std::fs::write("askama_output.html", &output).expect("Failed to write output file");
        }
        Err(e) => {
            eprintln!("Template rendering error: {}", e);
        }
    }
}
