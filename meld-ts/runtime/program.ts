import { createHash } from "crypto";

export interface ProgramHeader {
  version: number;
  instructionLength: number;
  contentLength: number;
}

const HEADER_SIZE = 42;
const CHECKSUM_START = 10;
const CHECKSUM_SIZE = 32;

function sha256Hex(data: Uint8Array): string {
  return createHash("sha256").update(data).digest("hex");
}

export class Program {
  readonly version: number;
  readonly instructions: Uint8Array;
  readonly content: Uint8Array;

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

    const storedChecksum = bytecode.slice(
      CHECKSUM_START,
      CHECKSUM_START + CHECKSUM_SIZE,
    );
    const payload = bytecode.slice(HEADER_SIZE, contentEnd);
    const computed = sha256Hex(payload);
    const stored = Array.from(storedChecksum)
      .map((b) => b.toString(16).padStart(2, "0"))
      .join("");

    if (computed !== stored) {
      throw new Error(`checksum mismatch: expected ${stored}, got ${computed}`);
    }

    const insStart = HEADER_SIZE;
    const insEnd = HEADER_SIZE + instructionLength;
    const contentStart = insEnd;

    this.instructions = bytecode.slice(insStart, insEnd);
    this.content = bytecode.slice(contentStart, contentEnd);
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

    const view = new DataView(
      this.instructions.buffer,
      this.instructions.byteOffset,
      this.instructions.byteLength,
    );
    const start = view.getUint32(pc, true);
    const end = view.getUint32(pc + 4, true);
    return [start, end];
  }

  getOpU32(pc: number): number {
    if (pc + 4 > this.instructions.length) {
      throw new Error(`pc out of bounds for u32: ${pc}`);
    }

    const view = new DataView(
      this.instructions.buffer,
      this.instructions.byteOffset,
      this.instructions.byteLength,
    );
    return view.getUint32(pc, true);
  }

  getContent(start: number, end: number): Uint8Array {
    if (end > this.content.length) {
      throw new Error(
        `content out of bounds: start=${start}, end=${end}, total=${this.content.length}`,
      );
    }
    return this.content.slice(start, end);
  }

  getContentString(start: number, end: number): string {
    return new TextDecoder().decode(this.getContent(start, end));
  }
}
