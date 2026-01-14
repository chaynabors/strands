# Strands Specification Preamble

**Version:** 0.1.0
**Status:** Draft (Stable Candidate)
**License:** Apache 2.0 OR MIT

## 1. Executive Summary

Strands is a strict Application Binary Interface (ABI) designed to standardize the execution boundary between intelligent agents and their host runtimes.

It addresses the interoperability problem in the current AI ecosystem where diverse agent frameworks must be manually integrated with various infrastructure providers. Strands replaces fragile, high-latency HTTP/JSON interfaces with a deterministic, high-performance binary contract. This decoupling allows organization-wide infrastructure, including Compute, Auth, Logging, and Tooling, to evolve independently of rapidly changing agent logic.

Strands is not a framework. It is a **compilation target**. It serves as the foundation upon which frameworks, SDKs, and proprietary agent logic are built to ensure portability, security, and long-term ABI stability.

## 2. Strategic Architecture

The specification is engineered to meet the requirements of enterprise-grade control planes. It emphasizes fault isolation, predictable latency, and zero-copy data interchange.

### 2.1 The Fundamental Layer

Strands defines the atomic unit of agent execution. By standardizing the memory layout and state transition mechanics at the C-ABI level, Strands enables a **Unified Runtime Environment**.

- **Infrastructure Agnosticism:** An agent compiled to the Strands ABI functions identically whether deployed on a local edge device, inside a secure enclave (TEE), or within a massive-scale serverless cluster.
- **Logic Isolation:** The strict separation of Host and Guest ensures that logic failures, infinite loops, or memory leaks within an agent cannot compromise the stability of the orchestration platform.

### 2.2 Structural Extensibility (Forward Compatibility)

To prevent specification drift and ensure the standard remains viable for decades, Strands adopts a **Chain-of-Responsibility** extension model similar to Vulkan and OpenXR.

- **Non-Destructive Evolution:** Core structures utilize `s_type` (Structure Type) and `p_next` (Pointer to Next) fields. This allows the ABI to support new capabilities, such as hardware acceleration contexts, specialized security tokens, or real-time sensor buffers, without altering the memory layout of the base standard.
- **Capability Negotiation:** The Host and Agent perform a handshake at initialization. An Agent tailored for future hardware can gracefully degrade when running on legacy Hosts by traversing the `p_next` chain and ignoring unrecognized extensions. This guarantees backward compatibility without version fragmentation.

## 3. Technical Pillars

### 3.1 Strict Binary Layout

The ABI enforces a rigorous memory layout to ensure cross-language and cross-architecture compatibility.

- **Wasm/Native Unification:** Explicit padding macros, specifically `STRANDS_PAD_PTR`, ensure that data structures maintain identical alignment on 32-bit WebAssembly runtimes and 64-bit native Hosts. This eliminates the need for expensive serialization or marshalling steps at the boundary.
- **Cache Efficiency:** Primary event structures are aligned to 128 bytes, or two standard CPU cache lines, to optimize prefetching and throughput in high-density multi-tenant environments.

### 3.2 The Reactive State Machine

Strands rejects the blocking loop model of traditional scripts in favor of a non-blocking, reactive architecture.

- **Inversion of Control:** The Agent does not manage its own lifecycle. The Host weaves the agent into existence for a discrete time slice. The Agent processes inputs, emits a decision, and yields control immediately.
- **Async Determinism:** Long-running operations, such as Network I/O or Vector Search, are offloaded to the Host via the `STRANDS_RESULT_PENDING` signal. This allows a single Host thread to efficiently interleave the execution of thousands of dormant agents to maximize hardware utilization.

### 3.3 Managed Context Lifecycle

The specification treats the Context Window, or Timeline, as a managed system resource rather than an implementation detail.

- **Standardized Truncation:** The ABI defines system-level operations for context garbage collection via `strands.sys.context.truncate`. This creates a uniform interface for managing token budgets across different model providers which prevents context overflow crashes in production.

## 4. Ecosystem Positioning

Strands functions as the interface layer (L2) in the modern AI stack.

- **L4 Application Logic:** Enterprise workflows, vertical-specific agents, autonomous bots.
- **L3 Frameworks:** Tools such as LangChain or AutoGen compile down to Strands modules.
- **L2 Strands ABI:** **The Immutable Contract.**
- **L1 Runtime Infrastructure:** Systems such as Cloudflare Workers, Kubernetes Sidecars, or Desktop Runtimes implement the Strands Host interface.

## 5. Repository Structure

- `spec/`: Normative specification text.
- `include/`: Canonical C headers defining the ABI.
  - `strands.h`: Core symbol definitions, memory alignment macros, and extension mechanisms.
  - `strands_std.h`: Registry of standardized Capability URIs.
  - `strands_std_structs.h`: Binary layouts for high-performance IPC payloads.

## 6. License

This project is dual-licensed under either the **MIT License** or the **Apache License, Version 2.0**, at your option.

- **MIT:** [LICENSE-MIT](LICENSE-MIT)
- **Apache 2.0:** [LICENSE-APACHE](LICENSE-APACHE)

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you shall be dual licensed as above, without any additional terms or conditions.
