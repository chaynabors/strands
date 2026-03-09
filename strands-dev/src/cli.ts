#!/usr/bin/env tsx

import { program } from "commander";
import { setup } from "./commands/setup.js";
import { build } from "./commands/build.js";
import { test } from "./commands/test.js";
import { check } from "./commands/check.js";
import { fmt } from "./commands/fmt.js";
import { generate } from "./commands/generate.js";
import { example } from "./commands/example.js";
import { clean } from "./commands/clean.js";
import { upgrade } from "./commands/upgrade.js";
import { ci } from "./commands/ci.js";

process.env.PYTHONPYCACHEPREFIX ??= ".pycache";

program
  .name("strands-dev")
  .description(
    `Strands monorepo development CLI

Build pipeline (each step feeds the next):
  wit/agent.wit -> strands-ts -> strands-wasm -> strands-rs -> strands-py

Most commands accept layer flags (--ts, --rs, --py, --wasm).
No flags = run all layers.

Dependency rules:
  TS SDK changes only need a downstream rebuild if the public API changed.
  WASM depends on TS and is rebuilt automatically.
  Rust host reads the .wasm at build time (AOT-compiled to .cwasm).
  Python extension wraps Rust via PyO3 and maturin.
  Pure Python is editable-installed and never needs a rebuild.
  WIT contract changes require a full generate + build.
  Derive macro changes cascade into the Rust host.`,
  );

program
  .command("setup")
  .description("Install toolchains and dependencies")
  .option("--rust", "Rust stable, wasm32-wasip2, cargo tools")
  .option("--node", "npm install and ComponentizeJS symlink")
  .option("--python", "Create venv, install maturin, ruff, componentize-py")
  .action((opts) => setup(opts));

program
  .command("build")
  .description("Compile one or more layers")
  .option("--ts", "TypeScript SDK")
  .option("--wasm", "WASM component (rebuilds TS first)")
  .option("--rs", "Rust host")
  .option("--py", "Python extension and type stubs")
  .option("--kt", "Kotlin/Java SDK (UniFFI bindings + Gradle)")
  .option("--release", "Release build")
  .action((opts) => build(opts));

program
  .command("test")
  .description("Run tests")
  .option("--rs", "Rust tests")
  .option("--py", "Python tests")
  .option("--ts", "TypeScript tests")
  .option("--kt", "Kotlin/Java tests")
  .argument("[file]", "Specific Python test file")
  .action((file, opts) => test({ ...opts, file }));

program
  .command("check")
  .description("Lint and type-check without building")
  .option("--rs", "Rust clippy (workspace and pyo3)")
  .option("--ts", "TypeScript type-check")
  .option("--py", "Python ruff")
  .option("--kt", "Kotlin/Java compile check")
  .action((opts) => check(opts));

program
  .command("fmt")
  .description("Format all code")
  .option("--check", "Fail if anything would change")
  .action((opts) => fmt(opts));

program
  .command("generate")
  .description("Regenerate type declarations from WIT")
  .option("--check", "Fail if generated files are out of date")
  .action((opts) => generate(opts));

program
  .command("example")
  .description("Run an example by name")
  .argument("<name>", "Example name")
  .option("--rs", "Run a Rust example (default)")
  .option("--py", "Run a Python example")
  .option("--ts", "Run a TypeScript example")
  .option("--kt", "Run the Kotlin example")
  .option("--java", "Run the Java example")
  .action((name, opts) => example(name, opts));

program
  .command("clean")
  .description("Remove all build artifacts")
  .action(() => clean());

program
  .command("upgrade")
  .description("Bump Rust dependencies to latest compatible versions")
  .option("--incompatible", "Include major version bumps")
  .action((opts) => upgrade(opts));

program
  .command("ci")
  .description("Full CI pipeline")
  .action(() => ci());

program.parse();
