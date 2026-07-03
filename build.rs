use std::fs;

use toml::Table;

fn main() {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let cargo_path = format!("{}/Cargo.toml", manifest_dir);
    let cargo_content = fs::read_to_string(&cargo_path).expect("Failed to read Cargo.toml");

    let parsed_content = cargo_content
        .parse::<Table>()
        .expect("Failed to parse Cargo.toml");

    if let Some(package) = parsed_content.get("tef").and_then(|p| p.as_table()) {
        if let Some(version) = package.get("version").and_then(|v| v.as_str()) {
            println!("cargo:rustc-env=TEF_VERSION={}", version);
        }
    }

    println!("cargo:rerun-if-changed={}", cargo_path);
}
