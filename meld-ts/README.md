# Meld TypeScript Runtime

TypeScript implementation of the Meld TEF runtime and benchmark harness.

## Scripts

- `bun run example`: runs the runtime examples, including rendering a compiled `.bhtml` file
- `bun run bench`: runs benchmarks and writes CSV files to `stats/`

## Benchmarks

The benchmark command uses `tinybench` (portable across Node, Deno, and Bun) and generates:

- `stats/handlebars.csv`
- `stats/meld.csv`
