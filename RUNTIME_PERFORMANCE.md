# Runtime Performance Notes

This file documents the runtime optimizations in Meld. Keep it up to date when
changing the interpreter, because many of the optimizations below are
intentional trade-offs that are easy to accidentally undo during cleanup.

## Observed results

The latest local benchmark runs showed approximately:

| Runtime | Before | After |
| --- | ---: | ---: |
| Go | 80K renders | 100K renders |
| Rust | 52K renders | 56K renders |
| TypeScript | 15K renders | 85K renders |

These are throughput observations from the repository benchmark setup, not
portable guarantees. Compare runs on the same machine, with the same template,
payload, build mode, and benchmark settings.

## Why the code looks this way

### Shared runtime strategy

The interpreter executes the same `Program` repeatedly. Bytecode operands,
lookup names, helper names, literal values, and loop metadata are immutable
after program construction. The runtimes therefore cache those values on the
runtime instance and only clear per-render state in `Run`.

Do not clear these caches from the normal render reset path. Doing so moves
parsing, UTF-8 conversion, and literal parsing back into every render.

The runtimes are intentionally not thread-safe. A separate runtime instance
should be used for concurrent rendering.

### TypeScript runtime

Relevant files: `meld-ts/runtime/runtime.ts`, `meld-ts/runtime/program.ts`,
and `meld-ts/runtime/stack.ts`.

- The interpreter dispatches directly from the instruction byte array and keeps
  one reusable instruction decoder instead of constructing `DataView` objects
  for every operand read.
- Program strings and literal values are decoded and parsed once, then reused
  across renders.
- Lookup keys, helper names, and loop metadata are cached by program counter.
- Dotted lookup paths are cached, avoiding repeated `split(".")` calls.
- Helper argument draining truncates the top of the stack instead of using a
  range splice that shifts unrelated values.
- Loop scope objects are reused for the same internal-frame reason as the Go
  runtime.

### Go runtime

Relevant files: `meld-go/runtime/runtime.go` and
`meld-go/runtime/stack.go`.

- The dispatch loop reads the instruction byte directly after a bounds check,
  avoiding a method/error path for every valid opcode.
- Lookup keys, helper names, constants, and iteration metadata are cached by
  program counter. This is especially important inside loops.
- `ScopeStack` caches dotted-path components. Repeated `strings.Split` calls
  were a significant cost for templates with nested lookups.
- `Stack.DrainTop` copies only the argument suffix and truncates the stack. The
  old general range drain also shifted the values below the range; helper
  arguments are always at the top, so that work was unnecessary.
- Loop scope maps are reused per loop instruction. The map is an internal
  frame and must not be exposed to callers; if a future opcode exposes scope
  objects, this optimization must be revisited.

### Rust runtime

Relevant files: `src/runtime/evaluator.rs` and `src/runtime/stack.rs`.

- Lookup keys, helper names, constants, and loop metadata are cached by
  program counter, matching the Go strategy.
- Payload lookups return `Cow::Borrowed` values when the value comes from the
  borrowed root payload. This avoids cloning entire arrays or objects just to
  put them on the evaluation stack.
- Values originating in owned loop scopes remain owned and are cloned when
  required by the evaluation stack. This preserves ownership and lifetime
  safety.
- Dotted lookup paths are split once and reused.
- Helper argument draining uses `Vec::split_off`, which is appropriate because
  helper arguments occupy the top of the evaluation stack.
- Loop metadata is shared through `Rc` so cached names can be used while the
  evaluator mutates its other runtime state. `Rc` is appropriate here because
  a runtime is evaluated synchronously and is not shared across threads; use
  `Arc` only if the runtime ownership model becomes cross-thread.

## Important bytecode invariant

TEF content operands are `(start, end)` byte offsets, not `(start, length)`:

```text
content[start:end]
```

When adding tests or bytecode builders, encode the end offset as
`start + byte_length`. Violating this invariant can produce misleading runtime
errors such as truncated helper names or empty literals.

## Validation

Useful checks after changing a runtime:

```sh
# Go
cd meld-go
go test ./...
go test -run '^$' -bench BenchmarkRuntime -benchtime=100ms ./runtime
go run ./cmd/example

# Rust
cd ..
cargo test
cargo run --example meld
```

The Go repository currently contains older unit-test fixtures that encode some
content operands as lengths rather than end offsets. The production examples
use the TEF `(start, end)` format. Fix or update those fixtures before using
the full Go unit-test result as a regression signal.

When measuring a change, report renders per second or nanoseconds per render,
not only the batch duration. Keep the benchmark iteration count and build mode
consistent; the TypeScript benchmark configuration is in
`meld-ts/cmd/bench/main.ts`.
