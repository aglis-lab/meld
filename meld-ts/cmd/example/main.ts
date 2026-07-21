import { readFile, writeFile } from "node:fs/promises";
import { createHash } from "node:crypto";
import { resolve } from "node:path";
import {
  OpCall,
  OpCondition,
  OpEnd,
  OpIterate,
  OpJump,
  OpLookup,
  OpLookupOut,
  OpOut,
  OpPopScope,
  OpText,
  Program,
  Runtime,
  newRuntimeConfig,
} from "../../runtime/index.ts";

type Bytes = number[];

function encodeU32(val: number): Bytes {
  return [
    val & 0xff,
    (val >> 8) & 0xff,
    (val >> 16) & 0xff,
    (val >> 24) & 0xff,
  ];
}

function createBytecode(instructions: Bytes, content: Bytes): Uint8Array {
  const instBytes = Uint8Array.from(instructions);
  const contentBytes = Uint8Array.from(content);
  const payload = Uint8Array.from([...instBytes, ...contentBytes]);

  const checksum = createHash("sha256").update(payload).digest();

  const header = new Uint8Array(42);
  const view = new DataView(header.buffer);
  view.setUint16(0, 1, true);
  view.setUint32(2, instructions.length, true);
  view.setUint32(6, content.length, true);
  header.set(checksum, 10);

  return Uint8Array.from([...header, ...instBytes, ...contentBytes]);
}

function appendText(instructions: Bytes, content: Bytes, text: string): void {
  const start = content.length;
  const chunk = Buffer.from(text, "utf8");
  const end = start + chunk.length;

  instructions.push(OpText, ...encodeU32(start), ...encodeU32(end));
  content.push(...chunk);
}

function appendLookupOut(
  instructions: Bytes,
  content: Bytes,
  key: string,
): void {
  const start = content.length;
  const chunk = Buffer.from(key, "utf8");
  const end = start + chunk.length;

  instructions.push(OpLookupOut, ...encodeU32(start), ...encodeU32(end));
  content.push(...chunk);
}

function appendLookup(instructions: Bytes, content: Bytes, key: string): void {
  const start = content.length;
  const chunk = Buffer.from(key, "utf8");
  const end = start + chunk.length;

  instructions.push(OpLookup, ...encodeU32(start), ...encodeU32(end));
  content.push(...chunk);
}

function appendCall(
  instructions: Bytes,
  content: Bytes,
  helper: string,
  argCount: number,
): void {
  const start = content.length;
  const chunk = Buffer.from(helper, "utf8");
  const end = start + chunk.length;

  instructions.push(OpCall, ...encodeU32(start), ...encodeU32(end), argCount);
  content.push(...chunk);
}

function runProgram(bytecode: Uint8Array, payload: unknown): string {
  const program = new Program(bytecode);
  const runtime = new Runtime(program, newRuntimeConfig());
  runtime.run(payload);
  return runtime.output();
}

function basicExample(): void {
  const instructions: Bytes = [];
  const content: Bytes = [];

  appendText(instructions, content, "Hello, World!");
  instructions.push(OpEnd);

  const output = runProgram(createBytecode(instructions, content), {});
  console.log("Output:", output);
}

function interpolationExample(): void {
  const payload = { name: "Alice", age: 30 };

  const instructions: Bytes = [];
  const content: Bytes = [];

  appendText(instructions, content, "Name: ");
  appendLookupOut(instructions, content, "name");
  appendText(instructions, content, ", Age: ");
  appendLookupOut(instructions, content, "age");
  instructions.push(OpEnd);

  const output = runProgram(createBytecode(instructions, content), payload);
  console.log("Output:", output);
}

function conditionalExample(): void {
  const payload = { admin: true };

  const instructions: Bytes = [];
  const content: Bytes = [];

  appendLookup(instructions, content, "admin");

  const conditionPos = instructions.length;
  instructions.push(OpCondition, 0, 0, 0, 0);

  appendText(instructions, content, "Admin");

  const jumpPos = instructions.length;
  instructions.push(OpJump, 0, 0, 0, 0);

  const elsePos = instructions.length;
  appendText(instructions, content, "User");

  const endPos = instructions.length;
  instructions.push(OpEnd);

  instructions.splice(conditionPos + 1, 4, ...encodeU32(elsePos));
  instructions.splice(jumpPos + 1, 4, ...encodeU32(endPos));

  const output = runProgram(createBytecode(instructions, content), payload);
  console.log("Output:", output);
}

function loopExample(): void {
  const payload = { items: ["apple", "banana", "cherry"] };

  const instructions: Bytes = [];
  const content: Bytes = [];

  appendLookup(instructions, content, "items");

  const loopStart = instructions.length;

  const itemStart = content.length;
  content.push(...Buffer.from("item", "utf8"));
  const itemEnd = content.length;

  const indexStart = content.length;
  content.push(...Buffer.from("index", "utf8"));
  const indexEnd = content.length;

  const iteratePos = instructions.length;
  instructions.push(
    OpIterate,
    ...encodeU32(itemStart),
    ...encodeU32(itemEnd),
    ...encodeU32(indexStart),
    ...encodeU32(indexEnd),
    0,
    0,
    0,
    0,
  );

  appendText(instructions, content, ", ");
  appendLookupOut(instructions, content, "item");
  instructions.push(OpPopScope);
  instructions.push(OpJump, ...encodeU32(loopStart));

  const doneTarget = instructions.length;
  instructions.push(OpEnd);

  instructions.splice(iteratePos + 17, 4, ...encodeU32(doneTarget));

  const output = runProgram(createBytecode(instructions, content), payload);
  console.log("Output:", output);
}

function helperExample(): void {
  const payload = { items: ["a", "b", "c"] };

  const instructions: Bytes = [];
  const content: Bytes = [];

  appendLookup(instructions, content, "items");
  appendCall(instructions, content, "length", 1);
  instructions.push(OpOut);
  instructions.push(OpEnd);

  const output = runProgram(createBytecode(instructions, content), payload);
  console.log("Items count:", output);
}

async function compiledExample(): Promise<void> {
  const root = resolve(import.meta.dirname, "../../..");
  const inputFile = resolve(root, "templates/comprehensive.bhtml");
  const outputFile = resolve(root, "templates/comprehensive.out.html");
  const jsonFile = resolve(root, "templates/comprehensive.json");

  const bytecode = new Uint8Array(await readFile(inputFile));
  const payload = JSON.parse(await readFile(jsonFile, "utf8"));

  const program = new Program(bytecode);
  const runtime = new Runtime(program, newRuntimeConfig());
  runtime.registerCallable("toUpperCase", (...args) => {
    const first = args[0];
    if (typeof first !== "string") {
      return "";
    }
    return first.toUpperCase();
  });
  runtime.run(payload);

  await writeFile(outputFile, runtime.output(), "utf8");
}

async function main(): Promise<void> {
  console.log("=== Example 1: Basic Template ===");
  basicExample();

  console.log("\n=== Example 2: Variable Interpolation ===");
  interpolationExample();

  console.log("\n=== Example 3: Conditionals ===");
  conditionalExample();

  console.log("\n=== Example 4: Loops ===");
  loopExample();

  console.log("\n=== Example 5: Helper Functions ===");
  helperExample();

  console.log("\n=== Example 6: Read Compiled File ===");
  await compiledExample();
  console.log("Wrote templates/comprehensive.out.html");
}

main().catch((error: unknown) => {
  console.error(error);
  process.exit(1);
});
