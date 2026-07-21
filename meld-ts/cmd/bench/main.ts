import { mkdir, readFile, writeFile } from "node:fs/promises";
import { resolve } from "node:path";
import Handlebars from "handlebars";
import { Bench } from "tinybench";
import { Program, Runtime, newRuntimeConfig } from "../../runtime/index.ts";

const SAMPLE_SIZE = 10;
const BENCH_ITERATIONS = Array.from({ length: 10 }, (_, i) => (i + 1) * 1000);

type BenchRow = {
  n: number;
  durationNs: number;
  stdNs: number;
  throughput: number;
};

function csv(rows: BenchRow[]): string {
  const lines = ["n,duration_ns,std_ns,throughput"];
  for (const row of rows) {
    lines.push(
      `${row.n},${row.durationNs.toFixed(2)},${row.stdNs.toFixed(2)},${row.throughput.toFixed(3)}`,
    );
  }
  return `${lines.join("\n")}\n`;
}

async function runBench(
  label: string,
  fn: (iterations: number) => number,
): Promise<BenchRow[]> {
  console.log(`${label} Template Engine Benchmark`);
  console.log("================================");
  console.log(`Sample Size: ${SAMPLE_SIZE}\n`);

  let sink = 0;
  const rows: BenchRow[] = [];
  for (const iterations of BENCH_ITERATIONS) {
    const bench = new Bench({
      iterations: SAMPLE_SIZE,
      warmup: false,
      warmupIterations: 0,
      time: 0,
    });

    bench.add(`iterations_${iterations}`, () => {
      sink = (sink + fn(iterations)) >>> 0;
    });

    await bench.run();
    const task = bench.tasks[0];
    const result = task?.result;

    if (!result || !("latency" in result)) {
      throw new Error(
        `benchmark failed for ${label} at iterations=${iterations}`,
      );
    }

    // tinybench latency/period are in milliseconds; convert to ns for CSV parity.
    const meanBatchMs = result.latency.mean;
    const stdDevBatchMs = result.latency.sd;
    const durationNs = meanBatchMs * 1e6;
    const stdNsPerOp = (stdDevBatchMs * 1e6) / iterations;
    const throughput = iterations / (meanBatchMs / 1e3);

    rows.push({
      n: iterations,
      durationNs,
      stdNs: stdNsPerOp,
      throughput,
    });
  }

  // Keep sink observable so engines cannot treat renders as dead work.
  console.log(`${label} checksum: ${sink}`);

  return rows;
}

function registerHandlebarsHelpers(): void {
  Handlebars.registerHelper("gt", (left: unknown, right: unknown) => {
    const l = typeof left === "number" ? left : Number(left ?? 0);
    const r = typeof right === "number" ? right : Number(right ?? 0);
    return l > r;
  });

  Handlebars.registerHelper("gte", (left: unknown, right: unknown) => {
    const l = typeof left === "number" ? left : Number(left ?? 0);
    const r = typeof right === "number" ? right : Number(right ?? 0);
    return l >= r;
  });

  Handlebars.registerHelper("and", (left: unknown, right: unknown) => {
    return Boolean(left) && Boolean(right);
  });

  Handlebars.registerHelper("or", (left: unknown, right: unknown) => {
    return Boolean(left) || Boolean(right);
  });

  Handlebars.registerHelper("concat", (...args: unknown[]) => {
    const values = args.slice(0, -1);
    return values.map((value) => (value == null ? "" : String(value))).join("");
  });

  Handlebars.registerHelper("length", (value: unknown) => {
    if (Array.isArray(value) || typeof value === "string") {
      return value.length;
    }
    if (value && typeof value === "object") {
      return Object.keys(value).length;
    }
    return 0;
  });

  Handlebars.registerHelper("coalesce", (value: unknown, fallback: unknown) => {
    if (value == null || value === "") {
      return fallback ?? "";
    }
    return value;
  });
}

async function handlebarsBenchmark(root: string): Promise<BenchRow[]> {
  const jsonPath = resolve(root, "templates/meld.json");
  const templatePath = resolve(root, "templates/handlebars.html");

  const payload = JSON.parse(await readFile(jsonPath, "utf8")) as Record<
    string,
    unknown
  >;
  const template = await readFile(templatePath, "utf8");
  registerHandlebarsHelpers();
  const compiled = Handlebars.compile(template);

  return runBench("Handlebars", (iterations) => {
    let localSink = 0;
    for (let i = 0; i < iterations; i++) {
      payload.count = i;
      const out = compiled(payload);
      localSink = (localSink + out.charCodeAt(i % out.length)) >>> 0;
    }
    return localSink;
  });
}

async function meldBenchmark(root: string): Promise<BenchRow[]> {
  const jsonPath = resolve(root, "templates/meld.json");
  const templatePath = resolve(root, "templates/meld.bhtml");

  const bytecode = new Uint8Array(await readFile(templatePath));
  const payload = JSON.parse(await readFile(jsonPath, "utf8")) as Record<
    string,
    unknown
  >;

  const program = new Program(bytecode);
  const runtime = new Runtime(program, newRuntimeConfig());

  return runBench("Meld", (iterations) => {
    let localSink = 0;
    for (let i = 0; i < iterations; i++) {
      payload.count = i;
      runtime.run(payload);
      const out = runtime.output();
      localSink = (localSink + out.charCodeAt(i % out.length)) >>> 0;
    }
    return localSink;
  });
}

async function main(): Promise<void> {
  const root = resolve(import.meta.dirname, "../../..");
  const statsDir = resolve(import.meta.dirname, "../../stats");

  await mkdir(statsDir, { recursive: true });

  const handlebarsRows = await handlebarsBenchmark(root);
  await writeFile(
    resolve(statsDir, "handlebars.csv"),
    csv(handlebarsRows),
    "utf8",
  );

  const meldRows = await meldBenchmark(root);
  await writeFile(resolve(statsDir, "meld.csv"), csv(meldRows), "utf8");

  console.log("Wrote benchmark CSV files:");
  console.log("- stats/handlebars.csv");
  console.log("- stats/meld.csv");
}

main().catch((error: unknown) => {
  console.error(error);
  process.exit(1);
});
