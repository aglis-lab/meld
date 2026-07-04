# Template Execution Format (TEF) Specification

## Table Binary Representation

| Represent   | Version | Instruction Length | Content Length | Checksum  | Instruction Body                 | Template Content             |
| ----------- | ------- | ------------------ | -------------- | --------- | -------------------------------- | ---------------------------- |
| **Size**    | 2 bytes | 4 bytes            | 4 bytes        | 32 bytes  | as defined by Instruction length | as defined by content length |
| **Example** | 1       | 213213             | 21323          | xxxx-xxxx | `[Opcodes...]`                   | `<h1>Hello...`               |

_(Note: All integers are strictly **little-endian**)_

## Rust Runtime Status

- Implemented: `END`, `TEXT`, `OUT`, `CONDITION`, `JUMP`, `POP_SCOPE`, `CALL`, `PUSH_CONST`, `LOOKUP`, `LOOKUP_OUT`, `EQ`, `NEQ`, `GT`, `GTE`, `LT`, `LTE`, `NOT`, `AND`, `OR`, `EMPTY`, `NOT_EMPTY`, `LENGTH`, `CONCAT`, `ITERATE`
- Not yet implemented: math opcodes (`ADD`, `SUB`, `MUL`, `DIV`, `MOD`) are still reserved.

---

## Render Format

These opcodes control the primary flow of rendering the template, outputting text, and looping or branching.

### END (0x00)

Signals the runtime to halt the execution of the current block or template and return the rendered output.

- **Size:** 1 byte

### TEXT (0x01)

Outputs raw static text directly from the template content block.

- **Size:** 9 bytes
- **Arg 1 (4 bytes):** OFFSET in the content block
- **Arg 2 (4 bytes):** LENGTH of the text to render

### OUT (0x02)

Pops a value from the evaluation stack and output the resulting value as text

- **Size:** 1 byte

### CONDITION (0x03)

Checks the boolean value on top of the evaluation stack or evaluates a condition

- **Size:** 5 bytes
- **Arg 1 (4 bytes):** Jump offset to jump if the condition evaluates to `false`

_(**Note:** Current runtime evaluates the condition expression before `CONDITION` and then performs a false-branch jump.)_

```html
{if expression} ... {else if expression} ... {else} ... {/if}
```

### ITERATE (0x04)

Iterates over an array/slice. Expects the collection and starting index to be pushed to the evaluation stack prior to execution.

- **Size:** 21 bytes
- **Arg 1 (4 bytes):** Content offset pointing the loop item variable name
- **Arg 2 (4 bytes):** Content Length of the loop item variable name
- **Arg 3 (4 bytes):** Content offset pointing the loop index variable name
- **Arg 4 (4 bytes):** Content Length of the loop index variable name
- **Arg 5 (4 bytes):** Jump when the iteration is complete

_(**Current runtime behavior:** reads collection from top of evaluation stack, pushes `{item_name, index_name}` scope object per iteration, and jumps to `Arg 5` when complete.)_

```html
{each expression as name, index}
<!-- Iterate Block -->
{/each}
```

### JUMP (0x05)

Unconditionally moves the instruction pointer to a specific bytecode offset.

- **Size:** 5 bytes
- **Arg 1 (4 bytes):** IR body offset

### POP_SCOPE (0x06)

Pops the top stack frame from the scope stack, cleaning up localized loop or block variable to prevent scope leaks

- **Size:** 1 byte

---

## Expression Instruction (Stack Machine)

The expression Instruction operates on an Evaluation Stack. Operators pop their required arguments, compute the result, and push it back onto the stack.

### CALL (0x10)

Calls a registered helper function or external method.

- **Size:** 10 bytes
- **Arg 1 (4 bytes):** Content offset pointing the variable name
- **Arg 2 (4 bytes):** Length of the variable name
- **Arg 2 (1 byte):** Arg Count determining how many values to pop and pass

_(**Current runtime helpers:** `length`, `empty`, `not_empty`, `concat`, `coalesce`.)_

```html
{{ toCapitalize(name) }} is {{ age }} years old
```

### PUSH_CONST (0x11)

Pushes a literal value onto the evaluation stack.

- **Size:** 10 bytes
- **Arg 1 (1 byte):** TYPE (Literal Type ID)
- **Arg 1 (4 bytes):** Content offset pointing to the value
- **Arg 2 (4 bytes):** Length of the value

```html
<!-- String -->
"hello world"

<!-- Float -->
3.14

<!-- Integer -->
312
```

### LOOKUP (0x12)

Lookup a variable in the current Scope Stack, traversing nested paths (e.g., `user.name.first`)

- **Size:** 9 bytes
- **Arg 1 (4 bytes):** Content offset pointing the variable name
- **Arg 2 (4 bytes):** Length of the variable name

```html
username
```

### LOOKUP_OUT (0x13)

Lookup a variable in the current Scope Stack, traversing nested paths (e.g., `user.name.first`) and output the value directly into the output.

- **Size:** 9 bytes
- **Arg 1 (4 bytes):** Content offset pointing the variable name
- **Arg 2 (4 bytes):** Length of the variable name

_(**Note:** This is the optimized behaviour of LOOKUP and OUT instruction, because push value into evaluation is quite expensive we merge the instructions instead)_

```html
{{ username }}
```

### Logic and String Operators (0x20 - 0x2C)

All of these operators consume **0 argument** in bytecode. They exclusively pop from the push to the stack

- **Size:** 1 byte each

| OP Code (Hex) | Name      | Stack Action (Pop -> Push) | Description                                    |
| ------------- | --------- | -------------------------- | ---------------------------------------------- |
| **0x20**      | EQ        | Pops 2, pushes 1 (Bool)    | Equals                                         |
| **0x21**      | NE        | Pops 2, pushes 1 (Bool)    | Not Equal                                      |
| **0x22**      | GT        | Pops 2, pushes 1 (Bool)    | Greater                                        |
| **0x23**      | GTE       | Pops 2, pushes 1 (Bool)    | Greater/Equal                                  |
| **0x24**      | LT        | Pops 2, pushes 1 (Bool)    | Less                                           |
| **0x25**      | LTE       | Pops 2, pushes 1 (Bool)    | Less/Equal                                     |
| **0x26**      | NOT       | Pops 1, pushes 1 (Bool)    | Logical Not                                    |
| **0x27**      | AND       | Pops 2, pushes 1 (Bool)    | Logical And                                    |
| **0x28**      | OR        | Pops 2, pushes 1 (Bool)    | Logical Or                                     |
| **0x29**      | EMPTY     | Pops 1, pushes 1 (Bool)    | Is Length 0 or Null/undefined                  |
| **0x2A**      | NOT EMPTY | Pops 1, pushes 1 (Bool)    | Is Length greater than 0 or not null/undefined |
| **0x2B**      | LENGTH    | Pops 1, pushes 1 (Number)  | Array or string or Map length                  |
| **0x2C**      | CONCAT    | Pops 2+, pushes 1 (String) | Concatenate string-like values                 |

### Math Operator

---

## Literal Types

Used by the `PUSH_CONST` instruction to dictate how the runtime should cast the raw string data located in the Content block.

| Type Code (Hex) | Name    | Description              |
| --------------- | ------- | ------------------------ |
| **0x30**        | STRING  | Parsed as raw text       |
| **0x31**        | FLOAT   | Parsed as float 64 bit   |
| **0x32**        | INTEGER | Parsed as integer 64 bit |
| **0x33**        | BOOLEAN | Parsed as true/false     |
| **0x34**        | NULL    | Parsed as null value     |
