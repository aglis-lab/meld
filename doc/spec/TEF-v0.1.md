# Template Execution Format (TEF) Specification v0.1

**Scope**: Bytecode Format & Runtime Specification  
**Status**: Stable  
**Version**: 0.1.0  
**Last Updated**: 2026-07-26  
**Runtime Languages**: Rust, Go, TypeScript  
**Produced By**: Meld Compiler  

## Important: Meld vs TEF

This specification defines the **TEF bytecode format** - the binary output that **runtimes** execute.

- **Meld Spec** (see `MELD-v0.1.md`) = What `.meld` template files can contain
- **TEF Spec** (this file) = What runtimes execute (bytecode format)

**Pipeline:**
```
Meld Template File (.meld)
        ↓
    [COMPILER]
        ↓
TEF Bytecode (.bhtml)  ← This spec defines the format
        ↓
    [RUNTIME]  (uses this TEF spec)
        ↓
    Output
```

This document focuses on the **runtime level** - the binary bytecode format that runtimes execute.

---

## Table of Contents

1. [File Format](#file-format)
2. [Opcodes Reference](#opcodes-reference)
3. [Stack Machine](#stack-machine)
4. [Execution Model](#execution-model)
5. [Error Handling](#error-handling)
6. [Roadmap](#roadmap)

---

## File Format

### Binary Layout

```
[Header (14 bytes)] [Bytecode] [Content Block]
```

| Field | Size | Type | Description |
|-------|------|------|-------------|
| Version | 2 bytes | u16 (LE) | Format version (currently 1) |
| Instruction Length | 4 bytes | u32 (LE) | Size of bytecode section |
| Content Length | 4 bytes | u32 (LE) | Size of content block |
| Checksum | 4 bytes | u32 (LE) | CRC32 of bytecode + content |
| **Bytecode** | variable | bytes | Instruction sequence |
| **Content** | variable | UTF-8 | Static strings and literals |

### Byte Order

All multi-byte integers are **little-endian (LE)**.

### Example

```
[Ver: 01 00] [Len: D5 03 00 00] [Clen: 39 05 00 00] [CRC32: xxxxxxxx] [... bytecode ...] [... content ...]
Version 1       Length 981        Content 1337          Checksum       Instruction body    Template content
```

---

## Opcodes Reference

### Render Opcodes (0x00-0x06)

Opcodes that control template rendering flow, text output, and control structures.

#### END (0x00)

Signals runtime to halt execution of current block or template.

**Bytecode**: 1 byte
```
0x00
```

**Behavior**: Stops execution, returns accumulated output

**Example**: End of template
```
[TEXT ...] [END]
```

---

#### TEXT (0x01)

Outputs static text directly from the content block.

**Bytecode**: 9 bytes
```
0x01 [offset_start: 4 bytes LE] [offset_end: 4 bytes LE]
```

**Arguments**:
- `offset_start`: Position in content block where text begins
- `offset_end`: Position in content block where text ends

**Stack Effect**: (no change)

**Example**: Output "Hello "
```
0x01 [0x00 00 00 00] [0x06 00 00 00]  (content: "Hello ")
```

---

#### OUT (0x02)

Pops value from evaluation stack and outputs it as text.

**Bytecode**: 1 byte
```
0x02
```

**Stack Effect**: 
- **Before**: `[... value]`
- **After**: `[...]`

**Behavior**: 
- Pops top value
- Converts to string
- Appends to output
- Handles null/undefined as empty string

**Example**: After LOOKUP_OUT or expression evaluation
```
[LOOKUP "name"] [OUT]
```

---

#### CONDITION (0x03)

Evaluates condition and conditionally jumps.

**Bytecode**: 5 bytes
```
0x03 [jump_offset: 4 bytes LE]
```

**Arguments**:
- `jump_offset`: Bytecode offset to jump if condition is false

**Stack Effect**:
- **Before**: `[... condition_bool]`
- **After**: `[...]` (after consuming condition)

**Behavior**:
- Pops boolean from stack
- If false, jumps to `jump_offset`
- If true, continues to next instruction

**Example**: `{if age > 18} ... {/if}`
```
[PUSH 18] [LOOKUP "age"] [GT] [CONDITION jump_to_end] [...] [END]
```

---

#### ITERATE (0x04)

Begins iteration over array or collection.

**Bytecode**: 21 bytes
```
0x04 
[item_name_start: 4 bytes LE] [item_name_end: 4 bytes LE]
[index_name_start: 4 bytes LE] [index_name_end: 4 bytes LE]
[loop_end_offset: 4 bytes LE]
```

**Arguments**:
- `item_name`: Variable name for current item (from content block)
- `index_name`: Variable name for current index (from content block)
- `loop_end_offset`: Bytecode offset after loop

**Stack Effect**:
- **Before**: `[... array, start_index]`
- **After**: `[...]`

**Behavior**:
- Pops array and start index from stack
- Creates new scope with `item_name` and `index_name`
- Executes loop body
- Jumps to `loop_end_offset` when complete

**Example**: `{each items as item, idx} ... {/each}`
```
[LOOKUP "items"] [PUSH 0] 
[ITERATE item_offset idx_offset loop_end] 
[... loop body ...] 
[JUMP back_to_iterate]
```

---

#### JUMP (0x05)

Unconditionally jumps to offset.

**Bytecode**: 5 bytes
```
0x05 [target_offset: 4 bytes LE]
```

**Arguments**:
- `target_offset`: Bytecode offset to jump to

**Stack Effect**: (no change)

**Behavior**: Sets instruction pointer to `target_offset`

**Example**: Loop back
```
[... loop body ...] 
[JUMP back_to_iterate]
```

---

#### POP_SCOPE (0x06)

Pops scope frame, cleaning up loop/block variables.

**Bytecode**: 1 byte
```
0x06
```

**Stack Effect**: (scope stack, not evaluation stack)

**Behavior**: 
- Removes top scope frame
- Variables in that scope become inaccessible
- Prevents scope leaks

**Example**: After loop
```
[ITERATE ...] [...] [POP_SCOPE]
```

---

### Expression Opcodes (0x10-0x13)

Opcodes for variable lookup and function calls.

#### CALL (0x10)

Calls built-in helper function or registered external function.

**Bytecode**: 10 bytes
```
0x10 [func_name_start: 4 bytes LE] [func_name_end: 4 bytes LE] [arg_count: 1 byte]
```

**Arguments**:
- `func_name`: Function name (from content block)
- `arg_count`: Number of arguments to pop

**Stack Effect**:
- **Before**: `[... arg1, arg2, ..., argN]` (N = arg_count)
- **After**: `[... result]`

**Behavior**:
- Pops `arg_count` values from stack
- Calls function with those arguments
- Pushes result back to stack

**Built-in Functions**:
| Name | Args | Description |
|------|------|-------------|
| `length` | 1 | Array/string length |
| `concat` | 2+ | Concatenate strings |
| `empty` | 1 | Check if empty |
| `not_empty` | 1 | Check if not empty |
| `coalesce` | 2+ | First non-null value |
| `toUpperCase` | 1 | Uppercase string |

**Example**: `{{ length(items) }}`
```
[LOOKUP "items"] [CALL length 1] [OUT]
```

---

#### PUSH_CONST (0x11)

Pushes literal constant onto evaluation stack.

**Bytecode**: 10 bytes
```
0x11 [type: 1 byte] [value_start: 4 bytes LE] [value_end: 4 bytes LE]
```

**Arguments**:
- `type`: Literal type (see Literal Types section)
- `value_start`, `value_end`: Content block offsets

**Stack Effect**:
- **Before**: `[...]`
- **After**: `[... literal_value]`

**Behavior**:
- Reads raw value from content block
- Parses according to type
- Pushes parsed value to stack

**Example**: Push string "hello"
```
0x11 [0x30] [0x00 00 00 00] [0x05 00 00 00]  (type: STRING, offsets point to "hello")
```

---

#### LOOKUP (0x12)

Looks up variable in scope stack without output.

**Bytecode**: 9 bytes
```
0x12 [var_name_start: 4 bytes LE] [var_name_end: 4 bytes LE]
```

**Arguments**:
- `var_name`: Variable name (from content block, supports dot notation: `user.name.first`)

**Stack Effect**:
- **Before**: `[...]`
- **After**: `[... value]`

**Behavior**:
- Searches scope stack for variable
- Traverses properties using dot notation
- Pushes value to stack
- Returns null if not found

**Example**: Lookup for expression (not output)
```
[LOOKUP "user.age"] [PUSH 18] [GT]
```

---

#### LOOKUP_OUT (0x13)

Looks up variable and directly outputs it (optimized LOOKUP + OUT).

**Bytecode**: 9 bytes
```
0x13 [var_name_start: 4 bytes LE] [var_name_end: 4 bytes LE]
```

**Arguments**:
- `var_name`: Variable name (supports dot notation)

**Stack Effect**: (no change to evaluation stack)

**Behavior**:
- Searches scope stack for variable
- Converts to string
- Appends to output
- Skips stack intermediate step

**Example**: `{{ name }}`
```
0x13 [name_offset_start] [name_offset_end]
```

---

### Logic & Comparison Opcodes (0x20-0x2C)

All logic and comparison opcodes operate on the evaluation stack, popping operands and pushing results.

#### EQ (0x20) - Equality

**Bytecode**: 1 byte
```
0x20
```

**Stack Effect**:
- **Before**: `[... left, right]`
- **After**: `[... bool_result]`

**Behavior**: `left == right` (value equality, not reference)

**Example**: `{{ age == 18 }}`
```
[LOOKUP "age"] [PUSH 18] [EQ] [OUT]
```

---

#### NEQ (0x21) - Not Equal

**Bytecode**: 1 byte
```
0x21
```

**Stack Effect**:
- **Before**: `[... left, right]`
- **After**: `[... bool_result]`

**Behavior**: `left != right`

---

#### GT (0x22) - Greater Than

**Bytecode**: 1 byte
```
0x22
```

**Stack Effect**:
- **Before**: `[... left, right]`
- **After**: `[... bool_result]`

**Behavior**: `left > right`

---

#### GTE (0x23) - Greater Than or Equal

**Bytecode**: 1 byte
```
0x23
```

**Stack Effect**:
- **Before**: `[... left, right]`
- **After**: `[... bool_result]`

**Behavior**: `left >= right`

---

#### LT (0x24) - Less Than

**Bytecode**: 1 byte
```
0x24
```

**Stack Effect**:
- **Before**: `[... left, right]`
- **After**: `[... bool_result]`

**Behavior**: `left < right`

---

#### LTE (0x25) - Less Than or Equal

**Bytecode**: 1 byte
```
0x25
```

**Stack Effect**:
- **Before**: `[... left, right]`
- **After**: `[... bool_result]`

**Behavior**: `left <= right`

---

#### NOT (0x26) - Logical NOT

**Bytecode**: 1 byte
```
0x26
```

**Stack Effect**:
- **Before**: `[... value]`
- **After**: `[... bool_result]`

**Behavior**: `!value` (logical negation)

---

#### AND (0x27) - Logical AND

**Bytecode**: 1 byte
```
0x27
```

**Stack Effect**:
- **Before**: `[... left, right]`
- **After**: `[... bool_result]`

**Behavior**: `left && right` (both must be truthy)

---

#### OR (0x28) - Logical OR

**Bytecode**: 1 byte
```
0x28
```

**Stack Effect**:
- **Before**: `[... left, right]`
- **After**: `[... bool_result]`

**Behavior**: `left || right` (either can be truthy)

---

#### EMPTY (0x29) - Is Empty

**Bytecode**: 1 byte
```
0x29
```

**Stack Effect**:
- **Before**: `[... value]`
- **After**: `[... bool_result]`

**Behavior**: Returns true if value has length 0, is null, or is undefined

---

#### NOT_EMPTY (0x2A) - Is Not Empty

**Bytecode**: 1 byte
```
0x2A
```

**Stack Effect**:
- **Before**: `[... value]`
- **After**: `[... bool_result]`

**Behavior**: Inverse of EMPTY

---

#### LENGTH (0x2B) - Get Length

**Bytecode**: 1 byte
```
0x2B
```

**Stack Effect**:
- **Before**: `[... value]`
- **After**: `[... number]`

**Behavior**: Returns length of array, string, or object

---

#### CONCAT (0x2C) - Concatenate

**Bytecode**: 1 byte
```
0x2C
```

**Stack Effect**:
- **Before**: `[... arg1, arg2, ..., argN]` (variable count)
- **After**: `[... string_result]`

**Behavior**: Concatenates 2+ string values

---

### Math Opcodes (0x30-0x34) - Reserved for v0.2

| Code | Name | Purpose |
|------|------|---------|
| 0x30 | ADD | Addition |
| 0x31 | SUB | Subtraction |
| 0x32 | MUL | Multiplication |
| 0x33 | DIV | Division |
| 0x34 | MOD | Modulus |

These are reserved but not yet implemented in v0.1.

---

## Literal Types

Used by `PUSH_CONST` instruction to specify how to parse the raw value from content block.

| Code | Name | Encoding | Example |
|------|------|----------|---------|
| 0x30 | STRING | UTF-8 text | `hello` |
| 0x31 | FLOAT | IEEE 754 f64 string | `3.14` |
| 0x32 | INTEGER | 64-bit signed int | `42` |
| 0x33 | BOOLEAN | `true`/`false` | `true` |
| 0x34 | NULL | `null` | `null` |

---

## Stack Machine

### Evaluation Stack

The runtime maintains an evaluation stack for expression evaluation.

**Operations**:
- `PUSH`: Add value to stack top
- `POP`: Remove value from stack top
- `PEEK`: View top value without removing

### Stack Example

**Expression**: `age > 18 && verified`

```
1. LOOKUP "age"         → [18]
2. PUSH 18              → [18, 18]
3. GT                   → [true]
4. LOOKUP "verified"    → [true, true]
5. AND                  → [true]
6. CONDITION            → [] (condition checked)
```

---

## Scope Stack

The runtime maintains a scope stack for variable resolution.

**Operations**:
- `PUSH_SCOPE`: Add new scope frame (at ITERATE/CONDITION)
- `POP_SCOPE`: Remove top scope frame
- `LOOKUP`: Search from top scope downward

### Scope Resolution

```
[Global Scope]
  └─ [Loop Scope 1]
      └─ [Loop Scope 2]
          └─ [Current]
```

Variables are searched from current scope upward until found.

---

## Execution Model

### Single-Pass Execution

TEF is designed for single-pass, streaming execution:

1. Read opcode
2. Execute operation
3. Update output
4. Repeat until END

### No Optimization Passes

The bytecode assumes:
- No dead code elimination
- No constant folding
- Compiler handles all optimizations

### Deterministic Output

Given the same input data, TEF produces identical output every time:
- No randomness
- No system-dependent behavior
- Byte-for-byte identical across platforms

---

## Error Handling

### Compile-Time Errors

Compiler must ensure valid bytecode:
- All JUMP/CONDITION offsets are valid
- All variables referenced exist
- All ITERATE collections are arrays

### Runtime Errors

Runtimes should handle gracefully:
- Missing variables → empty string (for output) or null (for evaluation)
- Type mismatches → attempt conversion or error
- Array out of bounds → null
- Stack underflow → abort with error

### Error Recovery

Recommended behavior:
- **Output operations**: Continue with empty string
- **Evaluation operations**: Continue with null
- **Critical errors**: Abort and return error message

---

## Runtime Implementation Checklist

- [x] Rust: All v0.1 opcodes
- [x] Go: All v0.1 opcodes
- [x] TypeScript: All v0.1 opcodes
- [ ] PHP: Planned
- [ ] Python: Planned

---

## Roadmap

### v0.2 (Q3 2026)

**Runtime Features** (TEF bytecode support):
- [ ] Math opcodes (ADD, SUB, MUL, DIV, MOD)
- [ ] Enhanced error codes
- [ ] Stack depth limits
- [ ] Optimization hints in bytecode

### v0.3 (Q4 2026)

**Runtime Features**:
- [ ] Scope serialization
- [ ] Streaming opcodes
- [ ] Async execution markers
- [ ] Filter execution opcodes

### v1.0 (2027)

**Runtime Features**:
- [ ] Bytecode versioning scheme
- [ ] Performance profiling opcodes
- [ ] Debugging hooks
- [ ] Bytecode optimization dialect

---

## Compatibility

| Opcode | Rust | Go | TypeScript | Description |
|--------|------|----|----|-----------|
| 0x00-0x06 | ✅ | ✅ | ✅ | Render opcodes |
| 0x10-0x13 | ✅ | ✅ | ✅ | Expression opcodes |
| 0x20-0x2C | ✅ | ✅ | ✅ | Logic & comparison |
| 0x30-0x34 | 🔲 | 🔲 | 🔲 | Math opcodes |

---

## References

- [Meld Template Language Spec](./MELD-v0.1.md)
- [Runtime Performance](../RUNTIME_PERFORMANCE.md)
- [Compiler Source](../../src/)

---

## Changelog

### v0.1.0 (2026-07-26)

- Comprehensive TEF specification
- Opcodes documentation (render, expression, logic)
- Stack machine model
- Literal types
- Scope and evaluation stacks
- Error handling guidelines
- Runtime implementation status
- Roadmap (v0.2, v0.3, v1.0)
