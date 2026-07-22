import { createHash } from "crypto";

export interface ProgramHeader {
  version: number;
  instructionLength: number;
  contentLength: number;
}

const HEADER_SIZE = 42;
const CHECKSUM_START = 10;
const CHECKSUM_SIZE = 32;
const CONTENT_DECODER = new TextDecoder();

function sha256(data: Uint8Array): Uint8Array {
  return createHash("sha256").update(data).digest();
}

function hex(data: Uint8Array): string {
  let value = "";
  for (const byte of data) {
    value += byte.toString(16).padStart(2, "0");
  }
  return value;
}

export class Program {
  readonly version: number;
  readonly instructions: Uint8Array;
  readonly content: Uint8Array;
  private readonly instructionView: DataView;
  private readonly contentStrings = new Map<
    number,
    { end: number; value: string }
  >();

  constructor(bytecode: Uint8Array) {
    if (bytecode.length < HEADER_SIZE) {
      throw new Error("bytecode too short: minimum 42 bytes required");
    }

    const view = new DataView(
      bytecode.buffer,
      bytecode.byteOffset,
      bytecode.byteLength,
    );
    this.version = view.getUint16(0, true);
    const instructionLength = view.getUint32(2, true);
    const contentLength = view.getUint32(6, true);

    const contentEnd = HEADER_SIZE + instructionLength + contentLength;
    const expectedLength = contentEnd;
    if (bytecode.length < expectedLength) {
      throw new Error(
        `bytecode too short: expected ${expectedLength} bytes, got ${bytecode.length}`,
      );
    }

    const storedChecksum = bytecode.subarray(
      CHECKSUM_START,
      CHECKSUM_START + CHECKSUM_SIZE,
    );
    const payload = bytecode.subarray(HEADER_SIZE, contentEnd);
    const computed = sha256(payload);

    let checksumMatches = true;
    for (let i = 0; i < CHECKSUM_SIZE; i++) {
      if (computed[i] !== storedChecksum[i]) {
        checksumMatches = false;
        break;
      }
    }

    if (!checksumMatches) {
      const stored = hex(storedChecksum);
      const actual = hex(computed);
      throw new Error(`checksum mismatch: expected ${stored}, got ${actual}`);
    }

    const insStart = HEADER_SIZE;
    const insEnd = HEADER_SIZE + instructionLength;
    const contentStart = insEnd;

    this.instructions = bytecode.slice(insStart, insEnd);
    this.content = bytecode.slice(contentStart, contentEnd);
    this.instructionView = new DataView(
      this.instructions.buffer,
      this.instructions.byteOffset,
      this.instructions.byteLength,
    );
  }

  getOp(pc: number): number {
    if (pc < 0 || pc >= this.instructions.length) {
      throw new Error(`pc out of bounds: ${pc}`);
    }
    return this.instructions[pc];
  }

  getOpRange(pc: number): [number, number] {
    if (pc + 8 > this.instructions.length) {
      throw new Error(`pc out of bounds for range: ${pc}`);
    }

    const start = this.instructionView.getUint32(pc, true);
    const end = this.instructionView.getUint32(pc + 4, true);
    return [start, end];
  }

  getOpU32(pc: number): number {
    if (pc + 4 > this.instructions.length) {
      throw new Error(`pc out of bounds for u32: ${pc}`);
    }

    const instructions = this.instructions;
    return (
      instructions[pc] |
      (instructions[pc + 1] << 8) |
      (instructions[pc + 2] << 16) |
      (instructions[pc + 3] << 24)
    ) >>> 0;
  }

  getContent(start: number, end: number): Uint8Array {
    if (start < 0 || end < start || end > this.content.length) {
      throw new Error(
        `content out of bounds: start=${start}, end=${end}, total=${this.content.length}`,
      );
    }
    return this.content.slice(start, end);
  }

  getContentString(start: number, end: number): string {
    const cached = this.contentStrings.get(start);
    if (cached?.end === end) {
      return cached.value;
    }

    if (start < 0 || end < start || end > this.content.length) {
      throw new Error(
        `content out of bounds: start=${start}, end=${end}, total=${this.content.length}`,
      );
    }

    const value = CONTENT_DECODER.decode(this.content.subarray(start, end));
    this.contentStrings.set(start, { end, value });
    return value;
  }
}
