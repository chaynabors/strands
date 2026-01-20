# Understanding the Filament Specification

This guide explains the Filament Specification, building up from core concepts to technical details.

## What is Filament?

Filament is a **specification for building autonomous AI agents** that can run safely and deterministically across different environments. Think of it as a contract between:
- **The Host** (the runtime environment that executes the agent)
- **The Plugin** (the actual AI agent code)

## The Core Philosophy

Filament treats agents as **pure, stateless functions** rather than long-running processes. Instead of an agent constantly running and maintaining state, it exists only during brief execution cycles called "Weaves."

### The Weave Model

A Weave is like a single turn in a conversation:

1. The Host gives the Plugin a **history of events** (the Timeline)
2. The Plugin processes this history and **returns new events** (actions to take)
3. The Host executes those actions and appends results to the Timeline
4. Repeat for the next cycle

This is similar to a stateless request-response model - each execution includes the full conversation history, and the agent responds based on that history without maintaining persistent memory between sessions.

## Key Architectural Decisions

### 1. **Deterministic Execution**
Given the same history and context, the Plugin must produce the same logical output. This enables:
- **Replay** - debugging by re-running the exact same sequence
- **Time-travel debugging** - stepping backward through agent decisions
- **Testing** - verifiable, reproducible behavior

### 2. **Memory Safety**
The spec defines how memory is managed to prevent crashes and security vulnerabilities:

- **The Arena**: Temporary scratch space cleared after each Weave
- **The Heap**: Where the Plugin stores its own state
- **Blob Storage**: For large files that don't fit in memory

### 3. **WebAssembly-First Design**
WebAssembly (Wasm) is the primary target because it provides:
- **Sandboxing** - the agent can't accidentally access the Host's memory
- **Portability** - run the same agent on different platforms
- **Safety** - strict memory bounds checking

However, the spec also supports native code (C/C++, Python, JVM) for enterprise integration.

## The Three Layers

### Layer 1: Binary Interface (The ABI)
This is the low-level protocol - how data structures are laid out in memory, what functions must be exported, etc. It defines the exact format of messages between Host and Plugin.

**Key structures:**
- `FilamentEvent` - represents something that happened (192 bytes, precisely defined)
- `FilamentWeaveInfo` - the context for execution (time limits, resource budgets)
- `FilamentValue` - a flexible data container (can hold numbers, strings, maps, etc.)

### Layer 2: Agent Definition
This is the declarative configuration:
- **Manifest (`filament.toml`)** - describes what Plugins to load, their permissions, configuration
- **Lockfile (`filament.lock`)** - locks exact versions and checksums for reproducibility

It's similar to a `package.json` in Node.js or `Cargo.toml` in Rust.

### Layer 3: Standard Capabilities
Pre-defined schemas for common operations:
- HTTP requests/responses
- Tool invocation (like function calling)
- Key-value storage
- Logging

## How It Works in Practice

Let's trace a simple HTTP request:

1. **Plugin emits an event**: "I want to make an HTTP GET to example.com"
   - Creates a `FilamentEvent` with `type_uri = "filament.std.net.http.request"`
   - Payload contains the URL, method, headers

2. **Host processes the event**:
   - Sees this is an HTTP request
   - Actually makes the network call (the Plugin can't do this itself)
   - Creates a response event with the result

3. **Next Weave cycle**:
   - Host calls the Plugin again
   - Plugin sees the HTTP response in the Timeline
   - Can now use that data to make decisions

## Safety Features

### Resource Limits
The Host enforces strict budgets:
- **Time limits** - Plugins must finish within a deadline
- **Memory limits** - Can't allocate unlimited memory
- **Compute units** - Abstract measure of CPU usage

This prevents infinite loops and resource exhaustion.

### Sandbox Violations
For WebAssembly plugins, the Host validates every memory access:
- All pointers must point to valid memory
- No buffer overflows
- No accessing Host memory

### Deterministic Randomness
Even random numbers are deterministic! The Plugin receives a `random_seed` and must use that for all randomness, ensuring the same sequence of "random" numbers on replay.

## Why This Design?

**Traditional approach**: Agent runs as a long-lived process, maintaining state in memory, making direct system calls.

**Filament approach**: Agent is a pure function from history to actions.

**Benefits:**
- **Easier to test** - just provide a timeline and verify outputs
- **Easier to debug** - replay exact scenarios
- **Safer** - can't accidentally corrupt state or leak memory
- **Scalable** - Host can pool instances, pause/resume agents
- **Auditable** - complete history of what happened and why

## Real-World Use Cases

1. **Robotics**: Hard real-time control loops with safety guarantees
2. **Multi-tenant Cloud**: Run thousands of AI agents safely in shared infrastructure
3. **Enterprise**: Integrate with existing systems (databases, APIs) while maintaining control
4. **Conversational AI**: Stateless agents with persistence and tool access

## The Execution Lifecycle

```
1. Load Plugin → filament_get_info() [negotiate compatibility]
2. Reserve Memory → filament_reserve() [allocate arena]
3. Initialize → filament_create() [configure the agent]
4. Prepare → filament_prepare() [reset for new session]
5. Execute → filament_weave() [process timeline, return events]
6. Commit → Host saves events to timeline
7. Repeat step 5-6 as needed
8. Cleanup → filament_destroy() [teardown]
```

The spec is detailed because it needs to work across different programming languages, operating systems, and use cases - from tiny embedded devices to massive cloud deployments. Every structure size, alignment requirement, and error code is precisely specified to ensure interoperability.

## Getting Started

Filament is a low level interface. Language SDK's do not exist yet. If you're interested in experimenting with the interface then create an issue.

For more details, refer to the full specification document [here](./spec/filament-0.1.0.md).
