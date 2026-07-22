# Meld

**Meld** is a portable template compiler. Write once, compile once, execute anywhere.

Meld compiles templates into **TEF (Template Execution Format)**—a compact, language-agnostic bytecode that runs natively on Rust, Go, Java, Python, PHP, and beyond. No runtime parsing. No language-specific code generation. Just fast, portable template execution at 50K–80K ops/sec.

See [`RUNTIME_PERFORMANCE.md`](RUNTIME_PERFORMANCE.md) for runtime optimization
rationale, bytecode invariants, and benchmark guidance.

### Why Meld?

- **Fast**: Compiled templates execute at near-native speed (~50K–80K ops/sec)
- **Portable**: Same compiled template runs on Rust, Go, or any TEF-compatible runtime
- **Lightweight**: Minimal runtimes with zero parsing overhead at execution time
- **Composable**: Automatic template composition and pluggable helper functions

## Quick Start

**Compile a template:**

```rust
use meld::compiler::Builder;

let template = r#"<h1>Hello {{ name }}!</h1>"#;
let mut builder = Builder::new();
builder.build(template.as_bytes())?;
let compiled = builder.compile()?;
```

**Execute compiled template:**

```rust
use meld::runtime::Runtime;
let program = Runtime::new(&compiled, config)?;
let payload = serde_json::json!({"name": "World"});
runtime.run(&payload)?;
let output = runtime.output(); // "<h1>Hello World!</h1>"
```

See [examples/](/examples/) for complete usage patterns in Rust and Go.

## Goals

- Fast execution
- Portable execution format (TEF)
- Tiny runtimes
- Language-agnostic
- Framework-agnostic
- First-class HTML support
- Island architecture support
- Zero template parsing at runtime

## Architecture

```text
Template
    │
    ▼
+-------------+
|   Compiler  |
+-------------+
    │
    ▼
 Template Execution Format (TEF)
    │
    ├──────────────┐
    ▼              ▼
Rust Runtime   Go Runtime
    │              │
    └──────┬───────┘
           ▼
        HTML/etc Output
```

## Why TEF?

Traditional template engines either interpret templates at runtime or
generate language-specific source code.

Meld compiles templates into a portable execution format that can be
executed by any compatible runtime regardless of implementation
language.

## Features

- Linear instruction stream
- Stack-based expression VM
- Static HTML emitted by byte offsets
- Compile-time optimization
- Automatic template composition
- Pluggable helper functions
- Server components
- Island hydration support
- Portable runtime specification

## Project Structure

```text
meld/
├── src/
│   ├── compiler/
|   |   ├── adapters/
│   │   │   ├── react
│   │   │   ├── svelte
│   │   │   └── vue
│   ├── runtime/
└── examples/
└── benches/
└── samples/
```

## Vision

Meld aims to provide a common execution format for server-rendered
templates in the same way that WebAssembly provides a common execution
format for native code.

Templates are authored once, compiled once, and executed efficiently
across multiple languages through TEF.

## Benchmarks

Performance comparison of Rust and Go runtimes executing identical compiled templates:

| Implementation   | Throughput (ops/sec) | Latency (µs) |
| ---------------- | -------------------- | ------------ |
| **Go Runtime**   | ~80,000              | 12.5         |
| **Rust Runtime** | ~50,000              | 20.0         |

Benchmarks measure template execution throughput across varying payload sizes (10K–100K iterations). The Go runtime achieves approximately **60% higher throughput** while maintaining consistent performance. Both implementations scale linearly with input size.

**Benchmark methodology:**

- Compiled template execution only (parsing excluded)
- Real-world payloads with object/array nesting
- Measured across 20 sample runs per configuration

## Status

Early development. The TEF specification and runtime are under active
design and may change before the first stable release.
