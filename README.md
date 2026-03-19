# Strands

See [docs](./docs) for the design proposal and ongoing team decisions.

## Getting started

### Prerequisites

- Rust stable toolchain with the `wasm32-wasip2` target
- Node.js 22+
- Python 3.11+

### First-time setup

```bash
git clone https://github.com/strands-agents/strands.git
cd strands
npm run dev -- bootstrap
```

`bootstrap` installs toolchains, generates type bindings, builds all layers, and runs all tests. If this command doesn't enable development out of the box, file an issue.

## Architecture

### Build pipeline

Changes flow through a pipeline. Each layer compiles into the next:

```mermaid
graph TD
    WIT["wit/agent.wit"] -->|generate| TS_GEN["strands-ts/generated/"]
    WIT -->|generate| WASM_GEN["strands-wasm/generated/"]

    TS_GEN --> TS["strands-ts (npm build)"]
    TS -->|esbuild bundle| WASM_BUNDLE["strands-wasm (ESM bundle)"]
    WASM_GEN --> WASM_BUNDLE
    WASM_BUNDLE -->|componentize-js| WASM["agent.wasm (WASM component)"]

    WASM -->|"AOT compile (build.rs)"| RS["strands-rs (Cargo)"]
    RS -->|UniFFI cdylib| LIB["libstrands.so/.dylib"]

    LIB -->|"maturin (uniffi)"| PY["strands-py (Python package)"]

    LIB -->|uniffi-bindgen| KT_SRC["strands.kt (generated bindings)"]
    KT_SRC -->|Gradle| KT["strands-kt (Kotlin/Java)"]
```

| Directory         | Language   | What it is                                                          |
| ----------------- | ---------- | ------------------------------------------------------------------- |
| `wit/`            | WIT        | Interface contract between the WASM guest and host                  |
| `strands-ts/`     | TypeScript | Agent runtime: event loop, model providers, tools, hooks, streaming |
| `strands-wasm/`   | TypeScript | Bridges the TS SDK to WIT exports, compiles to a WASM component     |
| `strands-rs/`     | Rust       | WASM host: Wasmtime, AOT compilation, WASI HTTP, UniFFI             |
| `strands-derive/` | Rust       | Proc macro: generates UniFFI wrapper types from WIT bindgen output  |
| `strands-py/`     | Python     | Python wrapper: Agent class, @tool decorator, structured output     |
| `strands-kt/`     | Kotlin     | Kotlin/Java wrapper via UniFFI bindings                             |
| `strands-dev/`    | TypeScript | Dev CLI that orchestrates build, test, lint, and CI                 |

### Generated code

`npm run dev -- generate` produces type bindings from `wit/agent.wit` into:

- `strands-ts/generated/`
- `strands-wasm/generated/`

Generated files are checked in and marked with `// @generated`. Do not edit them by hand. CI runs `generate --check` and fails if they are stale.

UniFFI generates Python and Kotlin bindings from the compiled native library at build time. These are not checked in.

### Tests

| Layer          | Framework | Location                                                          |
| -------------- | --------- | ----------------------------------------------------------------- |
| TypeScript SDK | vitest    | `strands-ts/src/**/__tests__/` (unit), `strands-ts/test/` (integ) |
| Rust host      | cargo     | `strands-rs/src/` (doc-tests)                                     |
| Python wrapper | pytest    | `strands-py/tests_integ/`                                         |
| Kotlin wrapper | JUnit     | `strands-kt/lib/src/test/`                                        |

Add tests alongside the code you change. Bug fixes should include a test that reproduces the original issue.

## Making changes

Each layer depends on the layers above it in the pipeline. The `validate` command rebuilds and tests exactly the layers your change affects.

| What you changed                      | Validate command                      |
| ------------------------------------- | ------------------------------------- |
| WIT contract (`wit/agent.wit`)        | `npm run dev -- validate wit`         |
| TS SDK internals                      | `npm run dev -- validate ts`          |
| TS SDK public API                     | `npm run dev -- validate ts-api`      |
| WASM bridge (`strands-wasm/entry.ts`) | `npm run dev -- validate wasm`        |
| Rust host (`strands-rs/`)             | `npm run dev -- validate rs`          |
| Python bindings (Rust code)           | `npm run dev -- validate py-bindings` |
| Pure Python (`strands-py/`)           | `npm run dev -- validate py`          |

**TS internals vs. public API:** The WASM bridge (`strands-wasm/entry.ts`) imports specific types and functions from `strands-ts/`. If your change modifies something the bridge imports, it is a public API change — use `validate ts-api`. If the bridge does not import it, use `validate ts`.

**WIT contract changes** cascade to every layer. After running `validate wit`, fix any compile errors in `strands-wasm/entry.ts`, `strands-rs/`, `strands-derive/`, and the language wrappers. The build will not succeed until every layer matches the new contract.

## Dev CLI

```bash
npm run dev -- <command> [options]
```

Most commands accept layer flags (`--ts`, `--wasm`, `--rs`, `--py`, `--kt`). No flags means all layers.

| Command            | What it does                                                           |
| ------------------ | ---------------------------------------------------------------------- |
| `bootstrap`        | First-time setup: install, generate, build, test                       |
| `setup`            | Install toolchains (`--rust`, `--node`, `--python`)                    |
| `generate`         | Regenerate type bindings from WIT (`--check`)                          |
| `build`            | Compile layers (`--ts`, `--wasm`, `--rs`, `--py`, `--kt`, `--release`) |
| `test`             | Run tests (`--rs`, `--py`, `--ts`, `--kt`, or a specific `[file]`)     |
| `check`            | Lint and type-check (`--rs`, `--ts`, `--py`, `--kt`)                   |
| `fmt`              | Format all code (`--check` to verify without writing)                  |
| `validate <layer>` | Rebuild and test the layers affected by a change                       |
| `ci`               | Full pipeline: generate, format, lint, build, test                     |
| `rebuild`          | Clean rebuild: clean, generate, build                                  |
| `report`           | Status report from `tasks.toml` (`--full` for task-level detail)       |
| `clean`            | Remove all build artifacts                                             |
| `example <name>`   | Run an example (`--rs`, `--py`, `--ts`, `--kt`, `--java`)              |
| `upgrade`          | Bump Rust dependencies (`--incompatible`)                              |

## Code style

| Language   | Formatter     | Linter         |
| ---------- | ------------- | -------------- |
| Rust       | `cargo fmt`   | `cargo clippy` |
| TypeScript | `prettier`    | `tsc --noEmit` |
| Python     | `ruff format` | `ruff check`   |

```bash
npm run dev -- fmt       # format everything
npm run dev -- check     # lint everything
```

Comments are normative statements that describe what code does or why a decision was made. Avoid TODO's without associated issues, notes-to-self, and parenthetical asides.

## Submitting a PR

- Run `npm run dev -- ci` before pushing. This is the same pipeline CI runs.
- Keep PRs focused on a single change.
- Use single-line commit messages that describe what the change does.
