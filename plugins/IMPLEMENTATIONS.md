# IMPLEMENTATIONS

This document tracks what we can implement when combining Meld with the TEF spec/runtime.

## Assumptions

- TEF runtime is the source of truth for template execution.
- Backend engines (Go/TS/Rust) execute TEF bytecode.
- TEF does not define, discover, or schedule island architecture.
- JavaScript runtime SSR is out of scope.
- Vite plugin remains responsible for frontend asset graph and hydration entry generation.

## Boundary: Meld/Vite vs TEF

This project should follow the TEF spec boundary strictly:

- Meld + Vite plugin layer:
  - Parses `.meld` and resolves islands.
  - Decides hydration directives and client behavior.
  - Builds route manifest and client entry/chunk mapping.
- TEF compiler/runtime layer:
  - Compiles templates to bytecode and executes bytecode.
  - Produces deterministic template output from route/context.
  - Does not know React/Vue/Svelte, hydration, or island scheduling.

Practical rule:

- By the time TEF runs, island architecture must already be reduced to plain renderable template output + metadata manifests generated in the Vite/plugin pipeline.

## Current Status (Already Working)

- [x] Parse `.meld` pages into static HTML shell plus island metadata.
- [x] Discover `.meld` pages automatically from `src/pages/**/*.meld`.
- [x] Generate hydration modules via Vite virtual modules.
- [x] Serve pages in development with hydration script injection.
- [x] Emit production HTML plus compiled JS chunks.

## TEF Integration Targets

### A. Compiler and Intermediate Format

- [ ] Define Meld -> TEF mapping for:
  - [ ] Frontmatter imports table
  - [ ] Static HTML blocks
  - [ ] Island nodes (tag, import, props, hydrate directive)
- [ ] Produce deterministic TEF bytecode from `.meld` input.
- [ ] Add a normalized manifest format (JSON or binary index) that backend runtimes can consume.
- [ ] Split outputs explicitly:
  - [ ] TEF bytecode output for template execution
  - [ ] Island/client manifest output for hydration pipeline

Feasibility: High
Notes: This is the core of cross-backend portability.

### B. Backend Runtime Contract (Go/TS/Rust)

- [ ] Define runtime API:
  - [ ] `render(route, context) -> html`
  - [ ] `resolve_assets(route) -> scripts[], styles[]` (from manifest lookup, not TEF VM logic)
- [ ] Guarantee identical output behavior across Go/TS/Rust runtimes.
- [ ] Add conformance tests from shared fixtures.

Feasibility: High
Notes: Keep TEF VM contract minimal and language-neutral; island concerns stay outside runtime.

### C. Asset and Hydration Pipeline

- [ ] Keep Vite in charge of component bundling for React/Vue/Svelte islands.
- [ ] Generate per-route hydration entry modules from TEF island metadata.
- [ ] Emit route-level client manifest:
  - [ ] Which chunk hydrates which route
  - [ ] Which frameworks are required by that route
- [ ] Avoid global framework loading by splitting hydration entries per page/route.

Feasibility: High
Notes: This gives strong performance gains without SSR.

### D. Hydration Directive Semantics (No SSR)

- [ ] Implement directives from metadata:
  - [ ] `client:load`
  - [ ] `client:idle`
  - [ ] `client:visible` via `IntersectionObserver`
  - [ ] `client:media` via `matchMedia`
  - [ ] `client:only`
- [ ] Add idempotent hydration guard (do not mount same island twice).
- [ ] Add optional priority scheduling for many islands on one page.

Feasibility: High
Notes: Fully compatible with TEF runtime model.

### E. Dev Experience

- [ ] Hot reload for `.meld` and island components.
- [ ] Source maps from `.meld` locations to generated modules (for debugging).
- [ ] Error diagnostics:
  - [ ] Missing frontmatter import
  - [ ] Unsupported component extension
  - [ ] Invalid directive syntax
  - [ ] Non-serializable props

Feasibility: High
Notes: Most of this is Vite-plugin-side work.

### F. Routing and Backend Portability

- [ ] Export route manifest from compile step:
  - [ ] route path
  - [ ] tef module id
  - [ ] frontend entry chunk
- [ ] Support mounting in multiple servers:
  - [ ] Go HTTP server adapter
  - [ ] Node/Bun adapter
  - [ ] Rust Axum/Actix adapter
- [ ] Ensure identical route normalization rules.

Feasibility: High
Notes: This is where TEF brings major value.

## Not Possible (Given Current Constraint)

- [ ] JS-runtime component SSR during TEF render phase.
- [ ] Any TEF-runtime-level island discovery/scheduling.

Reason:

- TEF runtime executes templates, not React/Vue/Svelte component code.
- Without a JS SSR runtime bridge, islands cannot be pre-rendered from framework components at request/build render time.
- TEF spec is runtime bytecode focused; island architecture is an upstream compile/bundle concern.

## Optional Future Extensions

- [ ] Hybrid pre-render mode (if later you add a JS SSR bridge):
  - [ ] TEF renders template skeleton
  - [ ] JS runtime pre-renders selected islands
  - [ ] TEF injects resulting HTML + hydration markers
- [ ] Partial static generation cache for backend routes.
- [ ] Streaming TEF render with early HTML flush.

Feasibility: Medium
Notes: Requires architecture changes and cross-runtime synchronization.

## Suggested Implementation Order

1. Freeze TEF island metadata schema.
2. Build Meld -> TEF compiler output + fixtures.
3. Build backend runtime contract and conformance tests.
4. Generate per-route hydration entries and route manifest.
5. Complete full directive behavior (`visible`, `media`, `only`).
6. Add adapters (Go/TS/Rust) and benchmark.
7. Add DX diagnostics and source mapping.

## Minimum Viable TEF + Meld Spec

To keep implementation small and backend-friendly, lock these fields first:

- `tef`:
  - `route`: normalized route id/path
  - `bytecodeRef`: location/key for TEF payload
- `clientManifest`:
  - `route`
  - `islands[]`:
    - `id`
    - `component` (logical symbol)
    - `importPath`
    - `framework`
    - `props` (JSON-serializable only)
    - `directive` (load|idle|visible|media|only)
  - `assets`:
    - `entryScript`
    - `css[]`

If this remains stable, each backend only needs TEF bytecode execution plus manifest lookup.

## Open Questions

- Should `props` allow expression references (non-JSON), or strict JSON only?
- Should route IDs be file-based (`pages/index`) or URL-based (`/pages`)?
- Do you want one global hydration runtime file, or per-route tiny runtimes?
- Should framework chunks be shared across routes, or isolated per route for stricter boundaries?
