const DEFAULT_CAPACITY = 64;

export class Stack {
  private values: unknown[];

  constructor() {
    this.values = [];
  }

  push(v: unknown): void {
    this.values.push(v);
  }

  pop(): unknown | undefined {
    return this.values.pop();
  }

  peek(): unknown | undefined {
    return this.values[this.values.length - 1];
  }

  len(): number {
    return this.values.length;
  }

  clear(): void {
    this.values.length = 0;
  }

  drainRange(start: number, end: number): unknown[] | null {
    if (start < 0 || end > this.values.length || start > end) {
      return null;
    }

    const values = this.values.slice(start, end);
    this.values.length = start;
    return values;
  }

  drainTop(count: number): unknown[] | null {
    const start = this.values.length - count;
    if (count < 0 || start < 0) {
      return null;
    }

    const values = this.values.slice(start);
    this.values.length = start;
    return values;
  }
}

export class ScopeStack {
  private scopes: unknown[];
  private readonly pathParts = new Map<string, string[]>();

  constructor() {
    this.scopes = new Array<unknown>(0);
  }

  push(scope: unknown): void {
    this.scopes.push(scope);
  }

  pop(): unknown | undefined {
    return this.scopes.pop();
  }

  len(): number {
    return this.scopes.length;
  }

  clear(): void {
    this.scopes.length = 0;
  }

  cleanupToDepth(depth: number): void {
    if (depth < 0) {
      this.clear();
      return;
    }
    if (depth < this.scopes.length) {
      this.scopes.length = depth;
    }
  }

  get(key: string): unknown | undefined {
    let parts = this.pathParts.get(key);
    if (parts === undefined) {
      parts = key.split(".");
      this.pathParts.set(key, parts);
    }

    for (let i = this.scopes.length - 1; i >= 0; i--) {
      let currentValue: unknown = this.scopes[i];
      let matchedCount = 0;

      for (const part of parts) {
        if (
          currentValue &&
          typeof currentValue === "object" &&
          !Array.isArray(currentValue)
        ) {
          const obj = currentValue as { [k: string]: unknown };
          if (part in obj) {
            currentValue = obj[part];
            matchedCount++;
          } else {
            break;
          }
        } else {
          break;
        }
      }

      if (matchedCount === parts.length) {
        return currentValue;
      }

      if (matchedCount > 0) {
        break;
      }
    }

    return undefined;
  }
}

export const defaultStackCapacity = DEFAULT_CAPACITY;
