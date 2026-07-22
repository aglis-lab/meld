import {
  LiteralBool,
  LiteralFloat,
  LiteralInteger,
  LiteralNull,
  LiteralString,
  OpAnd,
  OpCall,
  OpConcat,
  OpCondition,
  OpEmpty,
  OpEnd,
  OpEq,
  OpGt,
  OpGte,
  OpIterate,
  OpJump,
  OpLength,
  OpLookup,
  OpLookupOut,
  OpLt,
  OpLte,
  OpNeq,
  OpNot,
  OpNotEmpty,
  OpOr,
  OpOut,
  OpPopScope,
  OpPushConst,
  OpText,
} from "./opcodes.ts";
import { Program } from "./program.ts";
import { ScopeStack, Stack } from "./stack.ts";

export interface RuntimeConfig {
  ignoreMissingVariables: boolean;
}

export type Callee = (...args: any[]) => any;

type OutputChunk =
  | { type: 0; start: number; end: number }
  | { type: 1; value: any };

type IterateMetadata = {
  itemName: string;
  indexName: string;
  doneTarget: number;
};

export function newRuntimeConfig(): RuntimeConfig {
  return {
    ignoreMissingVariables: true,
  };
}

export class Runtime {
  private readonly program: Program;
  private readonly config: RuntimeConfig;
  private readonly outputChunks: OutputChunk[];
  private outputBuffer: string | null;
  private readonly scopeStack: ScopeStack;
  private readonly scopeMarks: Map<number, number>;
  private readonly iterateIndices: Map<number, number>;
  private readonly evaluationStack: Stack;
  private readonly calleeFunc: Map<string, Callee>;
  private readonly constantValues: Map<number, any>;
  private readonly iterateScopes: Map<number, { [key: string]: any }>;
  private readonly lookupKeys: Map<number, string>;
  private readonly helperNames: Map<number, string>;
  private readonly iterateMetadata: Map<number, IterateMetadata>;

  constructor(program: Program, config: RuntimeConfig) {
    this.program = program;
    this.config = config;
    this.outputChunks = [];
    this.outputBuffer = null;
    this.scopeStack = new ScopeStack();
    this.scopeMarks = new Map();
    this.iterateIndices = new Map();
    this.evaluationStack = new Stack();
    this.calleeFunc = new Map();
    this.constantValues = new Map();
    this.iterateScopes = new Map();
    this.lookupKeys = new Map();
    this.helperNames = new Map();
    this.iterateMetadata = new Map();
  }

  registerCallable(name: string, callee: Callee): void {
    this.calleeFunc.set(name, callee);
  }

  run(payload: any): void {
    this.clear();

    if (payload && typeof payload === "object" && !Array.isArray(payload)) {
      this.scopeStack.push(payload);
    }

    const instructions = this.program.instructions;
    let pc = 0;
    while (true) {
      const opcode = instructions[pc];
      if (opcode === undefined) {
        throw new Error(`pc out of bounds: ${pc}`);
      }
      let step = 0;

      switch (opcode) {
        case OpEnd:
          return;
        case OpText:
          step = this.text(pc);
          break;
        case OpOut:
          step = this.out();
          break;
        case OpLookup:
          step = this.lookup(pc);
          break;
        case OpLookupOut:
          step = this.lookupOut(pc);
          break;
        case OpCall:
          step = this.call(pc);
          break;
        case OpPushConst:
          step = this.pushConst(pc);
          break;
        case OpEq:
          step = this.compare((left, right) => valuesEqual(left, right));
          break;
        case OpNeq:
          step = this.compare((left, right) => !valuesEqual(left, right));
          break;
        case OpGt:
          step = this.compareOrdered((cmp) => cmp > 0);
          break;
        case OpGte:
          step = this.compareOrdered((cmp) => cmp >= 0);
          break;
        case OpLt:
          step = this.compareOrdered((cmp) => cmp < 0);
          break;
        case OpLte:
          step = this.compareOrdered((cmp) => cmp <= 0);
          break;
        case OpNot:
          step = this.not();
          break;
        case OpAnd:
          step = this.logic((left, right) => left && right);
          break;
        case OpOr:
          step = this.logic((left, right) => left || right);
          break;
        case OpEmpty:
          step = this.empty(true);
          break;
        case OpNotEmpty:
          step = this.empty(false);
          break;
        case OpLength:
          step = this.length();
          break;
        case OpConcat:
          step = this.concat();
          break;
        case OpCondition:
          pc = this.condition(pc);
          continue;
        case OpJump:
          pc = this.jump(pc);
          continue;
        case OpPopScope:
          this.scopeStack.pop();
          step = 1;
          break;
        case OpIterate:
          pc = this.iterate(pc);
          continue;
        default:
          throw new Error(`unknown opcode: ${opcode} at pc ${pc}`);
      }

      pc += step;
    }
  }

  output(): string {
    if (this.outputBuffer !== null) {
      return this.outputBuffer;
    }

    if (this.outputChunks.length === 0) {
      this.outputBuffer = "";
      return this.outputBuffer;
    }

    let out = "";
    for (const chunk of this.outputChunks) {
      if (chunk.type === 0) {
        out += this.program.getContentString(chunk.start, chunk.end);
      } else {
        out += stringifyValue(chunk.value);
      }
    }

    this.outputBuffer = out;
    return this.outputBuffer;
  }

  private clear(): void {
    this.outputChunks.length = 0;
    this.outputBuffer = null;
    this.scopeStack.clear();
    this.evaluationStack.clear();
    this.scopeMarks.clear();
    this.iterateIndices.clear();
  }

  private emitText(start: number, end: number): void {
    this.outputChunks.push({ type: 0, start, end });
    this.outputBuffer = null;
  }

  private emitValue(value: any): void {
    this.outputChunks.push({ type: 1, value });
    this.outputBuffer = null;
  }

  private text(pc: number): number {
    const start = this.program.getOpU32(pc + 1);
    const end = this.program.getOpU32(pc + 5);
    this.emitText(start, end);
    return 9;
  }

  private out(): number {
    const val = this.evaluationStack.pop();
    if (val === undefined) {
      throw new Error("evaluation stack is empty");
    }

    this.emitValue(val);
    return 1;
  }

  private lookup(pc: number): number {
    let key = this.lookupKeys.get(pc);
    if (key === undefined) {
      const start = this.program.getOpU32(pc + 1);
      const end = this.program.getOpU32(pc + 5);
      key = this.program.getContentString(start, end);
      this.lookupKeys.set(pc, key);
    }
    let val = this.scopeStack.get(key);

    if (val === undefined) {
      if (!this.config.ignoreMissingVariables) {
        throw new Error(`can't lookup variable ${key}`);
      }
      val = null;
    }

    this.evaluationStack.push(val);
    return 9;
  }

  private lookupOut(pc: number): number {
    let key = this.lookupKeys.get(pc);
    if (key === undefined) {
      const start = this.program.getOpU32(pc + 1);
      const end = this.program.getOpU32(pc + 5);
      key = this.program.getContentString(start, end);
      this.lookupKeys.set(pc, key);
    }
    const val = this.scopeStack.get(key);

    if (val === undefined) {
      if (!this.config.ignoreMissingVariables) {
        throw new Error(`can't lookup variable ${key}`);
      }
      return 9;
    }

    this.emitValue(val);
    return 9;
  }

  private pushConst(pc: number): number {
    if (this.constantValues.has(pc)) {
      this.evaluationStack.push(this.constantValues.get(pc));
      return 10;
    }

    const literalType = this.program.getOp(pc + 1);
    let val: any;
    const start = this.program.getOpU32(pc + 2);
    const end = this.program.getOpU32(pc + 6);
    const content = this.program.getContentString(start, end);

    switch (literalType) {
      case LiteralString:
        val = content;
        break;
      case LiteralFloat: {
        const parsed = Number.parseFloat(content);
        if (Number.isNaN(parsed)) {
          throw new Error(`invalid float literal: ${content}`);
        }
        val = parsed;
        break;
      }
      case LiteralInteger: {
        const parsed = Number.parseInt(content, 10);
        if (Number.isNaN(parsed)) {
          throw new Error(`invalid integer literal: ${content}`);
        }
        val = parsed;
        break;
      }
      case LiteralBool:
        val = content === "true";
        break;
      case LiteralNull:
        val = null;
        break;
      default:
        throw new Error(`unknown literal type: ${literalType}`);
    }
    this.constantValues.set(pc, val);

    this.evaluationStack.push(val);
    return 10;
  }

  private call(pc: number): number {
    let helperName = this.helperNames.get(pc);
    if (helperName === undefined) {
      const start = this.program.getOpU32(pc + 1);
      const end = this.program.getOpU32(pc + 5);
      helperName = this.program.getContentString(start, end);
      this.helperNames.set(pc, helperName);
    }
    const argCount = this.program.getOp(pc + 9);

    const args = this.evaluationStack.drainTop(argCount);
    if (!args) {
      throw new Error("not enough arguments on evaluation stack");
    }

    let result: any;
    switch (helperName) {
      case "length":
        if (args.length !== 1) {
          throw new Error(`length expects 1 argument, got ${args.length}`);
        }
        result = lengthOf(args[0]);
        break;
      case "empty":
        if (args.length !== 1) {
          throw new Error(`empty expects 1 argument, got ${args.length}`);
        }
        result = isEmpty(args[0]);
        break;
      case "not_empty":
        if (args.length !== 1) {
          throw new Error(`not_empty expects 1 argument, got ${args.length}`);
        }
        result = !isEmpty(args[0]);
        break;
      case "concat": {
        let out = "";
        for (const arg of args) {
          out += stringifyValue(arg);
        }
        result = out;
        break;
      }
      case "coalesce": {
        let selected: any = null;
        for (const arg of args) {
          if (arg !== null) {
            selected = arg;
            break;
          }
        }
        result = selected;
        break;
      }
      default: {
        const callee = this.calleeFunc.get(helperName);
        if (!callee) {
          throw new Error(`unknown helper: ${helperName}`);
        }
        result = callee(...args);
      }
    }

    this.evaluationStack.push(result);
    return 10;
  }

  private iterate(pc: number): number {
    let metadata = this.iterateMetadata.get(pc);
    if (metadata === undefined) {
      const itemStart = this.program.getOpU32(pc + 1);
      const itemEnd = this.program.getOpU32(pc + 5);
      const indexStart = this.program.getOpU32(pc + 9);
      const indexEnd = this.program.getOpU32(pc + 13);
      metadata = {
        itemName: this.program.getContentString(itemStart, itemEnd),
        indexName: this.program.getContentString(indexStart, indexEnd),
        doneTarget: this.program.getOpU32(pc + 17),
      };
      this.iterateMetadata.set(pc, metadata);
    }

    const { itemName, indexName, doneTarget } = metadata;

    let baseDepth = this.scopeMarks.get(pc);
    if (baseDepth === undefined) {
      baseDepth = this.scopeStack.len();
      this.scopeMarks.set(pc, baseDepth);
    }

    this.scopeStack.cleanupToDepth(baseDepth);

    const collection = this.evaluationStack.peek();
    if (collection === undefined) {
      throw new Error("iterate expects a collection on evaluation stack");
    }

    if (collection === null) {
      this.iterateIndices.delete(pc);
      this.scopeMarks.delete(pc);
      this.evaluationStack.pop();
      return doneTarget;
    }

    if (!Array.isArray(collection)) {
      throw new Error("iterate requires array collection");
    }

    const nextIndex = this.iterateIndices.get(pc) ?? 0;
    if (nextIndex >= collection.length) {
      this.iterateIndices.delete(pc);
      this.scopeMarks.delete(pc);
      this.scopeStack.cleanupToDepth(baseDepth);
      this.evaluationStack.pop();
      return doneTarget;
    }

    let scope = this.iterateScopes.get(pc);
    if (scope === undefined) {
      scope = {};
      this.iterateScopes.set(pc, scope);
    }
    scope[itemName] = collection[nextIndex];
    scope[indexName] = nextIndex;

    this.scopeStack.push(scope);
    this.iterateIndices.set(pc, nextIndex + 1);
    return pc + 21;
  }

  private condition(pc: number): number {
    const falseTarget = this.program.getOpU32(pc + 1);
    const cond = this.evaluationStack.pop();
    if (cond === undefined) {
      throw new Error("evaluation stack is empty");
    }

    return cond ? pc + 5 : falseTarget;
  }

  private jump(pc: number): number {
    return this.program.getOpU32(pc + 1);
  }

  private compare(predicate: (left: any, right: any) => boolean): number {
    const right = this.evaluationStack.pop();
    const left = this.evaluationStack.pop();
    if (left === undefined || right === undefined) {
      throw new Error("evaluation stack is empty");
    }

    this.evaluationStack.push(predicate(left, right));
    return 1;
  }

  private compareOrdered(predicate: (cmp: number) => boolean): number {
    const right = this.evaluationStack.pop();
    const left = this.evaluationStack.pop();
    if (left === undefined || right === undefined) {
      throw new Error("evaluation stack is empty");
    }

    const cmp = compareValues(left, right);
    if (cmp === null) {
      throw new Error("values are not comparable");
    }

    this.evaluationStack.push(predicate(cmp));
    return 1;
  }

  private not(): number {
    const val = this.evaluationStack.pop();
    if (val === undefined) {
      throw new Error("evaluation stack is empty");
    }

    this.evaluationStack.push(!val);
    return 1;
  }

  private logic(predicate: (left: boolean, right: boolean) => boolean): number {
    const right = this.evaluationStack.pop();
    const left = this.evaluationStack.pop();
    if (left === undefined || right === undefined) {
      throw new Error("evaluation stack is empty");
    }

    this.evaluationStack.push(predicate(!!left, !!right));
    return 1;
  }

  private empty(expectEmpty: boolean): number {
    const val = this.evaluationStack.pop();
    if (val === undefined) {
      throw new Error("evaluation stack is empty");
    }

    let matched = isEmpty(val);
    if (!expectEmpty) {
      matched = !matched;
    }

    this.evaluationStack.push(matched);
    return 1;
  }

  private length(): number {
    const val = this.evaluationStack.pop();
    if (val === undefined) {
      throw new Error("evaluation stack is empty");
    }

    this.evaluationStack.push(lengthOf(val));
    return 1;
  }

  private concat(): number {
    const right = this.evaluationStack.pop();
    const left = this.evaluationStack.pop();
    if (left === undefined || right === undefined) {
      throw new Error("evaluation stack is empty");
    }

    this.evaluationStack.push(stringifyValue(left) + stringifyValue(right));

    return 1;
  }
}

function stringifyValue(val: any): string {
  return String(val);
}

function isEmpty(v: any): boolean {
  if (v === null) {
    return true;
  }
  if (typeof v === "string") {
    return v.length === 0;
  }
  if (Array.isArray(v)) {
    return v.length === 0;
  }
  if (v && typeof v === "object") {
    return Object.keys(v).length === 0;
  }
  return false;
}

function lengthOf(v: any): number {
  if (typeof v === "string" || Array.isArray(v)) {
    return v.length;
  }
  if (v && typeof v === "object") {
    return Object.keys(v).length;
  }
  return 0;
}

function valuesEqual(left: any, right: any): boolean {
  return left === right;
}

function compareValues(left: any, right: any): number | null {
  if (Number.isNaN(left) || Number.isNaN(right)) {
    return null;
  }

  if (left < right) {
    return -1;
  }
  if (left > right) {
    return 1;
  }
  if (left === right) {
    return 0;
  }

  return String(left).localeCompare(String(right));
}
