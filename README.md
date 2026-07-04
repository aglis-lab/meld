# Meld

**Meld** is a compiler and runtime for server-side templates built
around the **Template Execution Format (TEF)**.

Instead of interpreting template syntax at runtime, Meld compiles
templates into a compact, portable execution format that can be executed
by lightweight runtimes implemented in different programming languages.

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
|   Builder   |
+-------------+
    │
    ▼
 Template Execution Format (TEF)
    │
    ├──────────────┐
    ▼              ▼
Rust Evaluator   Go Evaluator
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
│   ├── builder/
|   |   ├── adapters/
│   │   │   ├── react
│   │   │   ├── svelte
│   │   │   └── vue
│   ├── evaluator/
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

## Status

Early development. The TEF specification and runtime are under active
design and may change before the first stable release.
