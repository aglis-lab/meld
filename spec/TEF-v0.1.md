# Template Execution Format (TEF) Specification

## Table Binary Representation

| Represent   | Version | IR Length | Content Length | Checksum  | IR Body                 | Template Content             |
| ----------- | ------- | --------- | -------------- | --------- | ----------------------- | ---------------------------- |
| **Size**    | 2 bytes | 4 bytes   | 4 bytes        | 32 bytes  | as defined by ir length | as defined by content length |
| **Example** | 1       | 213213    | 21323          | xxxx-xxxx | `[Opcodes...]`          | `<h1>Hello...`               |

_(Note: All integers are strictly **little-endian**)_

---

## Render Format

These opcodes control the primary flow of rendering the template, outputting text, and looping or branching.

### END (0x00)

Signals the runtime to halt the execution of the current block or template and return the rendered output.

- **Size:** 1 byte

### TEXT (0x01)

Outputs raw static text directly from the template content block.

- **Soze:** 9 bytes
- **Arg 1 (4 bytes):** OFFSET in the content block
- **Arg 2 (4 bytes):** LENGTH of the text to render

### VAR (0x02)

Pops a value from the evaluation stack and output the resulting value as text

- **Size:** 1 byte

### CONDITION (0x03)

Checks the boolean value on top of the evaluation stack or evaluates a condition

- **Size:** 9 bytes
- **Arg 1 (4 bytes):** IR Body offset to execute to determine the boolean condition
- **Arg 2 (4 bytes):** Jump offset to jump if the condition evaluates to `false`

_(**Notes:** We can execute the condition expression first. with this design we can ignore arg 1 and make the binary representation little smaller)_

### ISLAND (0x04)

Defines an interactive or hydrated component (e.g., astro-like island architecture)

- **Size:** 9 bytes
- **Arg 1 (4 bytes):** Content offset (e.g., regular query URL)
- **Arg 2 (4 bytes):** Length (how long those data or query URL is)

### ITERATE (0x05)

Iterates over an array/slice. Expects the collection and starting index to be pushed to the evaluation stack prior to execution.

- **Size:** 21 bytes
- **Arg 1 (4 bytes):** Content offset pointing the variable name
- **Arg 2 (4 bytes):** Content Length of the variable name
- **Arg 3 (4 bytes):** Content offset pointing the variable index name
- **Arg 4 (4 bytes):** Content Length of the variable index name
- **Arg 5 (4 bytes):** Jump when the iteration is complete

### JUMP

Unconditionally moves the instruction pointer to a specific bytecode offset.

- **Size:** 5 bytes
- **Arg 1 (4 bytes):** IR body offset

### POP_SCOPE (0x07)

Pops the top stack frame from the scope stack, cleaning up localized loop or block variable to prevent scope leaks

- **Size:** 1 byte

---

## Expression IR (Stack Machine)

The expression IR operates on an Evaluation Stack. Operators pop their required arguments, compute the result, and push it back onto the stack.

### CALL (0x10)

Calls a registered helper function or external method.

- **Size:** 10 bytes
- **Arg 1 (4 bytes):** Content offset pointing the variable name
- **Arg 2 (4 bytes):** Length of the variable name
- **Arg 2 (1 byte):** Arg Count determining how many values to pop and pass

### LOOK (0x11)

Looks up a variable in the current Scope Stack, traversing nested paths (e.g., `user.name.first`)

- **Size:** 9 bytes
- **Arg 1 (4 bytes):** Content offset pointing the variable name
- **Arg 2 (4 bytes):** Length of the variable name

### PUSH_CONST (0x12)

Pushes a literal value onto the evaluation stack.

- **Size:** 10 bytes
- **Arg 1 (1 byte):** TYPE (Literal Type ID)
- **Arg 1 (4 bytes):** Content offset pointing to the value
- **Arg 2 (4 bytes):** Length of the value

### Math, Logic, and String Operators (0x13 - 0x1F)

All of these operators consume **0 argument** in bytecode. They exclusively pop from the push to the stack

- **Size:** 1 byte each

| OP Code (Hex) | Name   | Stack Action (Pop -> Push) | Description                   |
| ------------- | ------ | -------------------------- | ----------------------------- |
| **0x13**      | EQ     | Pops 2, pushes 1 (Bool)    | Equals                        |
| **0x14**      | NE     | Pops 2, pushes 1 (Bool)    | Not Equal                     |
| **0x15**      | GT     | Pops 2, pushes 1 (Bool)    | Greater                       |
| **0x16**      | GTE    | Pops 2, pushes 1 (Bool)    | Greater/Equal                 |
| **0x17**      | LT     | Pops 2, pushes 1 (Bool)    | Less                          |
| **0x18**      | LTE    | Pops 2, pushes 1 (Bool)    | Less/Equal                    |
| **0x19**      | NOT    | Pops 2, pushes 1 (Bool)    | Logical Not                   |
| **0x1A**      | AND    | Pops 2, pushes 1 (Bool)    | Logical And                   |
| **0x1B**      | OR     | Pops 2, pushes 1 (Bool)    | Logical Or                    |
| **0x1C**      | EXISTS | Pops 1, pushes 1 (Bool)    | Is Not Null/Undefined         |
| **0x1D**      | EMPTY  | Pops 1, pushes 1 (Bool)    | Is Length 0                   |
| **0x1E**      | LENGTH | Pops 2, pushes 1 (Number)  | Array or string or Map length |
| **0x1F**      | CONCAT | Pops 1, pushes 1 (String)  | Concatenate String            |

---

## Literal Types

Used by the `PUSH_CONST` instruction to dictate how the runtime should cast the raw string data located in the Content block.

| Type Code (Hex) | Name    | Description              |
| --------------- | ------- | ------------------------ |
| **0x20**        | STRING  | Parsed as raw text       |
| **0x21**        | FLOAT   | Parsed as float 64 bit   |
| **0x22**        | INTEGER | Parsed as integer 64 bit |
| **0x23**        | BOOLEAN | Parsed as true/false     |
| **0x24**        | NBULL   | Parsed as raw text       |
