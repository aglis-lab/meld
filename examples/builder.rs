use std::env::args;
use std::fs;

fn main() {
    let input = args()
        .nth(1)
        .unwrap_or_else(|| "samples/index.html".to_string());
    let ext = input
        .rsplit('.')
        .next()
        .unwrap_or_else(|| panic!("Failed to get file extension for {}", input));
    let output = input[..input.len() - ext.len()].to_string() + "bhtml";

    let content = fs::read(&input).unwrap();
    let mut builder = meld::builder::Builder::new();
    builder.build(&content).unwrap();
    let content = builder.compile().unwrap();

    fs::write(output, content).unwrap();
}
