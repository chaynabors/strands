# Filament Specification 0.1.0

**Status:** Request for Comments

**Date:** 2026-01-19

**Distribution:** Public

## Table of Contents

**Part I: Concepts**

1.  [**Introduction and Conventions**](#part-i-concepts)
2.  [**Architecture Overview [Informative]**](#2-architecture-overview-informative)
3.  [**Security Model**](#3-security-model)

**Part II: Agent Definition**

1.  [**The Manifest Schema**](#part-ii-agent-definition)
2.  [**The Lockfile Schema**](#2-the-lockfile-schema)
3.  [**Configuration & Resource Resolution**](#3-configuration--resource-resolution)

**Part III: The Binary Interface**

1.  [**System Limits**](#part-iii-the-binary-interface)
2.  [**Execution Model**](#2-execution-model)
3.  [**Constants and Enumerations**](#3-constants-and-enumerations)
4.  [**Binary Layout and Primitives**](#4-binary-layout-and-primitives)
5.  [**Core Data Structures**](#5-core-data-structures)
6.  [**Host Interface Functions**](#6-host-interface-functions)
7.  [**Plugin Interface Functions**](#7-plugin-interface-functions)
8.  [**Host Implementation Requirements**](#8-host-implementation-requirements)

**Part IV: Standard Schemas & Capabilities**

1.  [**The Reactor Model**](#part-iv-standard-schemas--capabilities)
2.  [**Mandatory System Schemas**](#2-mandatory-system-schemas)
3.  [**Optional Capability Schemas**](#3-optional-capability-schemas)

**Appendices**

- [**Appendix A: Glossary [Informative]**](#appendix-a-glossary-informative)
- [**Appendix B: Core Header**](#appendix-b-core-header)
- [**Appendix C: Standard Library Header**](#appendix-c-standard-library-header)
- [**Appendix D: SDK Helpers [Informative]**](#appendix-d-sdk-helpers-informative)
- [**Appendix E: Conformance & Verification [Informative]**](#appendix-e-conformance--verification-informative)

---

# Part I: Concepts

## 1. Introduction and Conventions

### 1.1 Scope

This document defines the **Filament Specification**, a comprehensive standard for composable, deterministic autonomous systems. It unifies the three critical layers required to implement a compliant Host or Plugin:

1.  **The Agent Definition (Layer 2):** A declarative schema defining the topology, configuration, and security boundaries of an Agent.
2.  **The Binary Interface (Layer 1):** The low-level execution contract (ABI) between the Host runtime and the Plugin artifacts, including memory management and the event loop.
3.  **The Standard Capabilities:** A set of common schemas and interactions for I/O, networking, and tool use.

Filament is a **WebAssembly-First** ABI. It defines a **Strict Profile** for safety-critical Wasm execution (Robotics, Multi-tenant Cloud), and a **Relaxed Profile** for legacy Native integration and managed languages (Enterprise Integration).

#### 1.1.1 Maturity Status

Filament v0.1.0 is a draft. It it not ready for production. The Host Compliance Kit is not written, validation layers are not written, SDK's are not written, and types in this document are not yet frozen.

### 1.2 Normative Status

Unless explicitly marked as **[Informative]**, all sections in this document are **Normative**.

### 1.3 Keywords

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHALL**, **SHALL NOT**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY**, and **OPTIONAL** in this document are to be interpreted as described in [RFC 2119](https://www.rfc-editor.org/rfc/rfc2119).

### 1.4 Loader Capabilities

To support a polyglot ecosystem, Filament defines **Loader Capabilities**. A Host MAY support one or more of the following artifact types. The Host MUST declare supported loaders in `FilamentHostInfo`.

| Capability URI           | Description                    | Profile Alignment                            |
| :----------------------- | :----------------------------- | :------------------------------------------- |
| `filament.loader.wasm`   | WebAssembly Modules (`.wasm`)  | **Tier 1 (Strict)**. Hard Real-Time capable. |
| `filament.loader.native` | Shared Objects (`.so`, `.dll`) | **Tier 2 (Relaxed)**. Trusted execution.     |
| `filament.loader.python` | Python Scripts/Archives        | **Tier 2 (Relaxed)**. GC overhead.           |
| `filament.loader.jvm`    | Java Archives (`.jar`)         | **Tier 2 (Relaxed)**. GC overhead.           |

**Managed Language Requirement:** Loaders for managed languages (Python, JVM, JS) **SHOULD** expose the Plugin Arena via zero-copy buffer views (e.g., Python Buffer Protocol, Java `ByteBuffer`) rather than performing eager object deserialization. This ensures performance consistency with the C/Wasm ABI.

#### 1.4.1 Deferred Capabilities

The following capabilities are reserved for future specification and are NOT included in v0.1.0:

- `filament.ext.worker`
- `filament.ext.gpu`

---

## 2. Architecture Overview [Informative]

This section describes the conceptual model of the Filament Specification.

Filament is designed to decouple the logical reasoning of an autonomous agent from its runtime environment. The **Plugin** is a pure, stateless function that transforms a history of events into a new set of actions. The **Host** is the stateful runtime responsible for storage, I/O, network communication, and scheduling.

### 2.1 The Reactive State Machine

Filament inverts the traditional long-running process model. The Agent executes as a transient transition function called a **Weave**.

Let $E$ be the set of valid Events, $C$ the execution context, $S$ the set of valid Internal States (Memory/Heap), and $F$ the set of valid Failure states. The Weave is generally defined as:

$$\text{Weave}: E^* \times C \times S \to (E^* \times S) \cup F$$

where $E^*$ is the set of all finite event sequences. The specific behavior depends on the negotiated Capability Profile:

1.  **Stateless Mode (Pure):** The Agent does not persist memory between cycles. The input state is always the empty set $\emptyset$, and the output state is discarded. The function reduces to:

$$\text{Weave}\_{pure}: E^* \times C \to E^* \cup F$$

2.  **Stateful Mode:** The Agent persists its linear memory or heap. The output state $S'$ from cycle $N$ becomes the input state $S$ for cycle $N+1$.

$$\text{Weave}\_{stateful}: E^* \times C \times S \to (E^* \times S') \cup F$$

Regardless of the mode, the function enforces the following properties:

1.  **Logical Determinism:** For a fixed binary image and hardware architecture, the function is deterministic. Given an identical history $H$, context $c$, and input state $s$, the execution yields an identical result.
2.  **Artifact Integrity:** The definition of the transition function is bound to the specific compiled binary artifact and instruction set architecture.
3.  **Monotonicity:** The operation preserves history. If the result is a successful timeline $T \in E^*$, the input history $H$ is a prefix of $T$.
4.  **Bounded Termination:** The function is total. The Host enforces termination through strict resource budgeting to ensure the function maps to a valid output in finite time.
5.  **Stochastic Isolation:** The function execution is closed over its inputs. Pseudo-random operations derive strictly from the cryptographic seed within $C$. This ensures the tuple $(H, C, S)$ contains the complete state necessary for bitwise replay.

### 2.2 Memory Ownership Models

Filament employs distinct memory ownership strategies to balance safety and performance.

1.  **Guest-Owned, Host-Managed:** This is the standard model. The Plugin reserves a block of its own memory called the **Arena**. When the Plugin requests data, the Host **copies** the data into this Arena.
2.  **Host-Owned, Guest-Borrowed (Zero-Copy):** This model supports high-throughput data. Usage is restricted to Trusted environments via the `FILAMENT_READ_UNSAFE_ZERO_COPY` flag. The Host provides a pointer to its own internal memory. The Plugin **borrows** this pointer for the duration of the Weave.
3.  **Shared Memory:** In environments that support Shared Linear Memory (e.g., Wasm `SharedArrayBuffer` or POSIX `shm_open`), the Host MAY map a shared memory region directly into the Plugin's address space. In this model, the Arena backing store resides in shared memory, allowing the Host and multiple Plugin instances to access data without copying, subject to the safety rules defined in Section 8.4.
4.  **Blob Storage:** This model supports large artifacts that exceed memory limits. The Plugin receives a numeric **Handle** or `Blob ID`. Data is streamed in or out in small chunks.

### 2.3 Memory Lifecycle & Persistence

To enable efficient SDK implementation, Filament defines two distinct memory regions:

1.  **The Arena (Transient I/O):**
    - Allocated by `filament_reserve`.
    - Used for input Events and output commands.
    - **Lifecycle:** Reset/Wiped by the Host at the start of every Weave cycle. Data stored here does **not** persist.
2.  **The Heap (Plugin State):**
    - Managed by the Plugin (e.g., Wasm Linear Memory, C++ Static/Heap, Python Heap).
    - **Lifecycle:**
      - **Stateless Mode:** Wiped/Reloaded every Weave.
      - **Stateful Mode:** Persists across Weaves. The Host maintains the process/instance. Plugins MAY store state variables (`self.counter`) here.

**Snapshot Requirement:** To support `filament_snapshot`, Plugins **MUST** be able to serialize their Heap state into the Arena or Blobs on demand.

### 2.4 Blob Lifecycle

Blobs created via `filament_blob_create` are **Ephemeral** by default.

- **Creation:** The Host allocates temporary storage for the blob during the Weave.
- **Commit/GC:** Upon completion of the Weave, the Host checks if the `blob_id` is explicitly referenced in the **Returned Events** (via `payload_fmt=BLOB` or standard schemas) or the **State Snapshot**.
  - **Referenced:** The Blob is persisted and Sealed (Immutable).
  - **Unreferenced:** The Blob is deleted immediately.
- **Explicit Reference:** Hosts are **NOT** required to parse opaque payloads (strings/JSON) to find Blob IDs. Plugins **MUST** expose Blob IDs in the ABI headers or standard schemas to guarantee persistence.

---

## 3. Security Model

This section defines the safety contract between the Host and the Plugin.

### 3.1 Artifact Contexts

Filament defines distinct security contexts based on the Plugin artifact type.

1.  **Sandboxed Execution:** When hosting WebAssembly or other sandboxed bytecode, the Host **MUST** enforce memory isolation. All `FilamentAddress` values are treated as offsets relative to the Plugin's Address Space base. The Host **MUST** bounds-check every memory access.
2.  **Trusted Execution:** Native Shared Objects or Host-Process Interpreters (Python/JVM) execute within the Host's address space (or a trusted child process). The ABI cannot enforce memory safety at the hardware level for these plugins. The Host **MUST** assume that any `FilamentAddress` provided by a Trusted Plugin is a valid virtual pointer.

### 3.2 Information Leaks

Senders **MUST** initialize all padding bytes (fields starting with an underscore) to `0`. The Host **MUST** guarantee zero-initialization of padding to prevent information leaks from the Host memory space into the Plugin.

### 3.3 Address Validation

All `FilamentAddress` values passed from Plugin to Host must be validated.

- **Sandboxed:** Values are offsets. The Host **MUST** validate every read and write operation against the Plugin's memory bounds. Access violations **MUST** result in an immediate Trap.
- **Trusted:** Values are virtual pointers. The Host cannot validate these cheaply, but **SHOULD** restrict Trusted execution to signed artifacts.

**Relocation Requirement:** To support Snapshots and Time-Travel Debugging, Trusted/Native Plugins **MUST** utilize relative offsets from the Arena Base for any data structures intended to be serialized or inspected by the Host. Storing absolute virtual addresses in persistent state is **Undefined Behavior**.

---

# Part II: Agent Definition

This part defines **Layer 2** of the Filament architecture: the definition, configuration, and resolution of an Agent.

## 1. The Manifest Schema

The Manifest (`filament.toml`) is the authoritative source for the Agent's topology. It **MUST** be encoded using **TOML v1.0**.

**Note for Embedded Hosts:** Hosts operating in filesystem-less environments (e.g., RTOS) **MAY** resolve the Manifest and Config via static compilation or baked-in binary tables, provided the Resolution Order (Section 3.1) is respected.

### 1.1 Root Table: `[agent]`

| Key       | Type   | Required | Description                                |
| :-------- | :----- | :------- | :----------------------------------------- |
| `name`    | String | **Yes**  | The package name such as `aws/claims-bot`. |
| `version` | String | **Yes**  | Semantic Version such as `1.0.0`.          |
| `edition` | String | **Yes**  | Filament Spec Edition such as `2026`.      |
| `id`      | String | No       | Canonical Identity URN.                    |

### 1.2 Table: `[host]`

Declarative resource requirements for the runtime environment.

| Key              | Type    | Default | Description                           |
| :--------------- | :------ | :------ | :------------------------------------ |
| `min_arena`      | String  | `64MB`  | Minimum `FILAMENT_MIN_ARENA_BYTES`.   |
| `compute_budget` | Integer | `0`     | Default Compute Unit limit per Weave. |

### 1.3 Table: `[resources]`

Defines static assets available to the Agent.

| Key    | Type   | Required | Description                               |
| :----- | :----- | :------- | :---------------------------------------- |
| `path` | String | No       | Local filesystem path (Relative).         |
| `uri`  | String | No       | Remote URI (`s3://`, `https://`).         |
| `type` | String | **Yes**  | `text` or `blob`.                         |
| `mime` | String | No       | MIME type hint such as `application/pdf`. |

- **Constraint:** Exactly one of `path` or `uri` **MUST** be present for each resource entry.
- **Expansion:** Remote URIs **MUST** support Environment Variable interpolation such as `s3://${BUCKET}/file.dat`.

### 1.4 Array of Tables: `[[plugin]]`

Defines the execution pipeline. The Host **MUST** initialize and execute plugins in the strict order defined by this array.

| Key           | Type   | Required | Description                                             |
| :------------ | :----- | :------- | :------------------------------------------------------ |
| `alias`       | String | **Yes**  | Unique identifier for this instance such as `reasoner`. |
| `source`      | String | No       | Registry reference `namespace/pkg`.                     |
| `path`        | String | No       | Local path to binary such as `./bin/plugin.wasm`.       |
| `permissions` | List   | No       | List of Capability URIs.                                |
| `mounts`      | Table  | No       | Map of `Config Key` -> `Resource Key`.                  |

- **Resolution:** Exactly one of `source` or `path` **MUST** be present.

### 1.5 Table: `[config]`

The static configuration map. Keys **MUST** be namespaced by the Plugin Alias.

## 2. The Lockfile Schema

The Lockfile (`filament.lock`) is the machine-generated, cryptographically verified resolution of the Manifest. The Host **MUST** prioritize the Lockfile over the Manifest for execution.

### 2.1 Header: `[meta]`

Contains the hash of the Manifest file to detect drift.

### 2.2 Array of Tables: `[[package]]`

| Key        | Type   | Description                                  |
| :--------- | :----- | :------------------------------------------- |
| `name`     | String | The package name.                            |
| `version`  | String | The exact resolved version.                  |
| `source`   | String | The fully qualified URI.                     |
| `checksum` | String | **SHA-256 (Hex string)** of the binary blob. |

### 2.3 Array of Tables: `[[resource]]`

Resources must also be locked to ensure reproducibility.

| Key        | Type   | Description                                    |
| :--------- | :----- | :--------------------------------------------- |
| `name`     | String | The resource key from the manifest.            |
| `checksum` | String | **SHA-256 (Hex string)** of the asset content. |

**Host Requirement:** The Host **MUST** verify the checksum of both Plugins and Resources before execution.

## 3. Configuration & Resource Resolution

This section normatively defines how the Host maps Layer 2 definitions to Layer 1 ABI structures.

### 3.1 Injection Precedence

When initializing a Plugin via `filament_create`, the Host **MUST** populate the `FilamentConfig` structure by resolving values in the following order of precedence (Highest to Lowest):

1.  **Environment Variables:** Dynamic overrides found in the Host Environment matching `FILAMENT__<ALIAS>__<KEY>`.
2.  **Resources (Mounts):** Files or Blobs mounted via the `mounts` table in the Manifest.
3.  **Manifest Config:** Static values defined in `[config]`.

### 3.2 Stack Size

If the Manifest (or the Host configuration) specifies a stack size for a Plugin, the Host **MUST** prioritize that value over the `min_stack_bytes` hint found in the binary's `FilamentPluginInfo`.

### 3.3 Type Mapping

The Host **MUST** map TOML types to ABI `FilamentValueType` as follows:

| TOML Type | ABI Type              | Notes                                              |
| :-------- | :-------------------- | :------------------------------------------------- |
| String    | `FILAMENT_VAL_STRING` | UTF-8 view.                                        |
| Integer   | `FILAMENT_VAL_I64`    | Signed 64-bit integer.                             |
| Float     | `FILAMENT_VAL_F64`    | Double precision.                                  |
| Boolean   | `FILAMENT_VAL_BOOL`   | 1 or 0.                                            |
| Array     | `FILAMENT_VAL_LIST`   | Recursive conversion.                              |
| Table     | `FILAMENT_VAL_MAP`    | Recursive conversion.                              |
| DateTime  | **Error**             | Not supported. Use ISO-8601 Strings.               |
| Resource  | `FILAMENT_VAL_BLOB`   | If `type="blob"`, the Host injects a `blob_id`.    |
| Resource  | `FILAMENT_VAL_STRING` | If `type="text"`, the Host injects buffer content. |

---

# Part III: The Binary Interface

This part defines **Layer 1**: the execution contract between Host and Plugin.

## 1. System Limits

To ensure portability and prevent resource exhaustion, Filament defines specific limits. Compliant Hosts **MUST** provide at least the minimum capacities defined below.

| Limit Constant              | Minimum Value | Unit   | Description                                       |
| :-------------------------- | :------------ | :----- | :------------------------------------------------ |
| `FILAMENT_MIN_ARENA_BYTES`  | 64            | MiB    | Minimum size the Host must support for the Arena. |
| `FILAMENT_MIN_RECURSION`    | 64            | Levels | Max depth for nested `FilamentValue` structures.  |
| `FILAMENT_MIN_GRAPH_NODES`  | 4096          | Items  | Max total nodes in one Event payload.             |
| `FILAMENT_MAX_URI_LEN`      | 2048          | Bytes  | Max length for `type_uri` and Capability URIs.    |
| `FILAMENT_MIN_VALID_OFFSET` | 4096          | Bytes  | Reserve page to trap NULL dereferences.           |

## 2. Execution Model

### 2.1 Version Negotiation

The Filament Version is a packed 32-bit integer. The layout is calculated as:

- **Major:** $V \gg 22$
- **Minor:** $(V \gg 12) \land \text{0x3FF}$
- **Patch:** $V \land \text{0xFFF}$

The Plugin **MUST** export `filament_get_info`. The Host **MUST** verify compatibility according to [Semantic Versioning 2.0.0](https://semver.org/).

### 2.2 The Weave Cycle

The Host **MUST** provide a strict time budget in `FilamentWeaveInfo` and expose resource limits via `resource_max`.

- **Deadlock Protection:** If a Sandboxed Plugin exceeds `time_limit_ns`, the Host **SHOULD** terminate the instance.
- **Output Limits:** The Plugin **MUST NOT** emit an event with a payload larger than `max_event_bytes`.

#### 2.2.1 Real-Time Deadline Semantics

The meaning of `time_limit_ns` depends on the Execution Profile (Section 2.12).

1.  **Strict Timing (Hard Real-Time):** The Host **MAY** enforce `time_limit_ns` preemptively (e.g., using Instruction Counting or Gas Metering). If the limit is reached, the Host suspends execution immediately.
2.  **Relaxed Timing (Soft Real-Time):** The limit is **Cooperative**. The Plugin **SHOULD** check `resource_used` or the system clock periodically and return early if the limit is exceeded.

#### 2.2.2 Zero-Allocation Guarantee

To support Real-Time control loops, the Host **MUST NOT** perform dynamic heap allocations (`malloc`/`free`) during the execution of `filament_weave` after the initial Arena setup. All serialization and validation must occur within pre-allocated buffers or the Arena itself.

### 2.3 Memory Management

By default, the Host **MUST** guarantee that the Plugin executes with a clean memory state at the start of every Weave. This state must be equivalent to a fresh instantiation. Any heap allocations or global variable mutations from previous cycles **MUST NOT** persist.

If the Plugin negotiates the `FILAMENT_CAP_STATEFUL` capability, the Host **MAY** allow the Plugin instance to persist. Even in Stateful Mode, the Plugin **MUST NOT** retain pointers to the Host-Managed Arena or any data marked as Zero-Copy across Weave cycles.

#### 2.3.1 Timeline Pruning

For memory-constrained deployments or high-throughput agents, Hosts **MAY** implement automatic timeline pruning.

- **Notification:** Before pruning, the Host **SHOULD** emit a `filament.sys.context.prune` event so the Plugin can adjust its state.
- **Causality:** Hosts **SHOULD** retain events referenced by `ref_id` from un-pruned events.
- Plugins **MUST NOT** assume infinite timeline retention.

### 2.4 Event Commit Semantics

The Host **MUST** commit returned events atomically. Either all events in the batch are appended to the timeline, or none are.

- **Event Metadata Ownership:** The Host **MUST** assign the following fields upon commit and **MUST** ignore any values written by the Plugin:
  - `id`: Assigned by Host. IDs **MUST** be monotonically increasing within a single Timeline.
  - `timestamp`: Assigned by Host (wall-clock time of commit).
  - `tick`: Assigned by Host (logical causal time).
  - `auth_agent_id`: Assigned by Host (verified agent identity).
  - `trace_ctx`: Assigned by Host **IF** the Plugin's `trace_ctx` is zeroed. If the Plugin provides a non-zero `trace_ctx`, the Host **MUST** preserve it.
- **Streaming Exception (Yield):** If the Plugin returns `FILAMENT_RESULT_YIELD`, the Host **MUST** commit the events returned in the current batch immediately. This facilitates streaming responses. The Host **MUST** then invoke `weave()` again to resume execution.
- **Ordering:** The Host **MUST** process events in the order they appear in the array. Event `i+1` **MUST NOT** be processed until Event `i` is successfully handled.

### 2.5 Failure Handling

If a Host operation fails, the Host **MUST** append an event to the timeline with `type_uri="filament.sys.error"` and `ref_id` matching the request. The Host **MUST** then invoke `weave()` on the next cycle to allow the Plugin to handle the error. Hosts **SHOULD** implement retry limits or exponential backoff for repeated errors with the same `ref_id` to prevent infinite error loops.

If a Plugin returns `FILAMENT_RESULT_ERROR`, the Host **MUST** discard any uncommitted events from that cycle and append a `filament.sys.error` event to the Timeline indicating the crash. The Host **SHOULD** include the contents of `err_buf` (if non-empty and valid UTF-8) in the error event's message field to aid debugging.

### 2.6 Concurrency and Isolation

The Host **MUST** guarantee that only one instance of a Plugin executes against a given timeline at any time. `weave()` invocations are serialized. Hosts **MAY** allow multiple agents to share a timeline if they enforce strict isolation via `auth_agent_id`. The Host **MUST** filter events returned by `filament_read_timeline` to ensure an Agent can only see events belonging to its `auth_agent_id`, unless explicitly configured otherwise.

#### 2.6.1 Multi-Agent Execution (Instance Pooling)

Hosts **MAY** execute multiple Agents concurrently using instance pooling.

- **Instance Reuse:** The Host **MUST** invoke `filament_prepare` before switching an instance to a different Agent ID.
- **State Clearing:** The Plugin **MUST** reset all Agent-specific state (e.g., session variables) during `filament_prepare`. Failure to do so is a security violation.
- **Native Pools:** Native Plugins intended for pooling **MUST** avoid process-global state (e.g., static variables). Hosts **SHOULD** restrict Native Plugins to process-isolated execution unless the artifact is verified to be thread-safe.

### 2.7 Resource Accounting

To ensure deterministic execution across different Host implementations, Filament uses an abstract resource model. `resource_used` and `resource_max` are **opaque** monotonic counters representing **Compute Units** (often mapped to Instruction Counts in Wasm). Plugins **MUST** treat the ratio of `resource_used` to `resource_max` as a unitless throttling metric. If `resource_used` exceeds `resource_max`, the Host **MAY** terminate the Weave cycle immediately and return `FILAMENT_ETIMEOUT`.

### 2.8 Input Boundary

To ensure determinism, replayability, and Time Travel Debugging, all non-deterministic inputs **MUST** enter the Plugin via the Timeline or the `FilamentWeaveInfo` structure. Plugins **SHOULD NOT** read directly from hardware, OS time, or shared memory side-channels unless they are `FILAMENT_CAP_STATEFUL` and acknowledge that this breaks replayability guarantees.

### 2.9 Execution Triggers (The Event Loop)

The Host acts as the scheduler and orchestrator. While the Host determines the precise timing of execution, it **MUST** invoke the Plugin's `weave` function in response to the following state changes:

1.  **Ingress:** New events are appended to the Timeline from external sources. The Host **MUST** apply backpressure or rejection if the Ingress queue is full.
2.  **Completion:** An asynchronous operation requested by a previous event has completed.
3.  **Continuation:** The previous cycle returned `FILAMENT_RESULT_YIELD`.
4.  **Error Recovery:** A System Error event was generated in response to a failed Host operation.

### 2.10 KV Consistency

KV Updates emitted via `filament.std.kv.update` are applied **atomically at Commit**. A Plugin reads the state as it existed at the _start_ of the Weave (Snapshot Isolation). It does not see its own writes until the next cycle. If multiple KV updates in a single Weave target the same key, the Host **MUST** apply them in array order; the final update wins.

### 2.11 Timekeeping Models

To support both real-time interaction and deterministic simulation, the Host **MUST** maintain two distinct clocks for every event.

1.  **Wall Clock (`timestamp`):** The Unix Epoch timestamp (nanoseconds). This value represents the physical time of event creation. It is useful for logging, timeouts, and user interfaces. It is **NOT** guaranteed to be unique or monotonic in distributed systems.
2.  **Causal Clock (`tick`):** A monotonically increasing 64-bit integer representing the logical step index of the timeline. This value **MUST** be unique per event within a timeline and **MUST** strictly order events by causality. It is useful for deterministic replay and resolving concurrency conflicts.

**Note on Control Laws:** Plugins implementing control laws (e.g., PID controllers, Kalman Filters) **MUST** use `delta_time_ns` from `FilamentWeaveInfo` for integration and differentiation. This value represents the idealized simulation step or the precise control period, devoid of OS scheduling jitter.

### 2.12 Execution Profiles

Filament defines two primary execution profiles to support diverse environments. A Host **MUST** document which profile it implements.

1.  **Strict Timing (Sandboxed):** Optimized for **Robotics and Hard Real-Time**.
    - **Artifacts:** Wasm, Bytecode.
    - **Constraint:** Host uses Preemptive Compute Metering (Instruction Counting).
    - **Constraint:** No GC allowed during Weave.
    - **Native Constraint:** Native Plugins **MUST** be isolated in separate OS processes if the Host requires enforced preemption. Otherwise, preemption is Cooperative.
2.  **Relaxed Timing (Trusted/Managed):** Optimized for **Cloud and Enterprise**.
    - **Artifacts:** Native Shared Objects, Python, JVM.
    - **Constraint:** Host uses Cooperative Metering.
    - **Constraint:** Garbage Collection pauses are tolerated.

### 2.13 Initialization State Machine

To prevent undefined behavior or sandbox violations, the Host **MUST** execute the Plugin lifecycle in the following strict order:

1.  `filament_get_info()`: Negotiate ABI version.
2.  `filament_reserve()`: Allocate the Arena within the Plugin's address space.
3.  **Host Write:** The Host populates the Arena with `FilamentConfig`.
4.  `filament_create()`: Initialize the plugin using the Arena-resident config.
5.  `filament_prepare()`: Reset transient state for the current Agent ID.
6.  `filament_weave()`: Execute the logic cycle.

Invoking these functions out of order is **Undefined Behavior**.

## 3. Constants and Enumerations

This section defines the normative values for all enumerations and constants used in the ABI.

### 3.1 Structure Types and Ranges

Use of `s_type` allows for safe extensibility. To prevent collisions, the following ID ranges are enforced.

| Range Start | Range End | Owner                                   |
| :---------- | :-------- | :-------------------------------------- |
| `0`         | `999`     | **Core Specification** (Reserved).      |
| `1000`      | `9999`    | **Standard Capabilities** (Reserved).   |
| `10000`     | `19999`   | **Vendor Extensions** (e.g., AWS, ROS). |
| `20000`     | `MAX`     | **User / Application Specific**.        |

**Core Types:**

| ID  | Symbol                   | Category |
| :-- | :----------------------- | :------- |
| `1` | `FILAMENT_ST_EVENT`      | Core     |
| `3` | `FILAMENT_ST_WEAVE_INFO` | Core     |
| `4` | `FILAMENT_ST_HOST_INFO`  | Core     |
| `5` | `FILAMENT_ST_CONFIG`     | Core     |
| `6` | `FILAMENT_ST_PLUGIN`     | Core     |

**Standard Library Types:**

| ID    | Symbol                    | Category |
| :---- | :------------------------ | :------- |
| `101` | `FILAMENT_ST_SYS_ERROR`   | Std      |
| `102` | `FILAMENT_ST_CTX_PRUNE`   | Std      |
| `200` | `FILAMENT_ST_HTTP_REQ`    | Std      |
| `201` | `FILAMENT_ST_HTTP_RES`    | Std      |
| `300` | `FILAMENT_ST_TOOL_DEF`    | Std      |
| `302` | `FILAMENT_ST_TOOL_INVOKE` | Std      |
| `303` | `FILAMENT_ST_TOOL_RESULT` | Std      |
| `400` | `FILAMENT_ST_KV_UPDATE`   | Std      |
| `500` | `FILAMENT_ST_ENV_GET`     | Std      |
| `600` | `FILAMENT_ST_BLOB`        | Std      |

### 3.2 Error Codes

| Value | Symbol                           | Description                                      | POSIX Equivalent     |
| :---- | :------------------------------- | :----------------------------------------------- | :------------------- |
| `0`   | `FILAMENT_OK`                    | Success.                                         | `0`                  |
| `1`   | `FILAMENT_ERR_PERMISSION_DENIED` | Capability missing or Sandbox violation.         | `EACCES` / `EPERM`   |
| `2`   | `FILAMENT_ERR_NOT_FOUND`         | Resource, Key, or Blob ID not found.             | `ENOENT`             |
| `3`   | `FILAMENT_ERR_IO_FAILURE`        | Physical I/O failure (Disk/Network).             | `EIO`                |
| `4`   | `FILAMENT_ERR_NOT_CONFIGURED`    | Plugin or Device present but not initialized.    | `ENXIO`              |
| `5`   | `FILAMENT_ERR_DATA_TOO_LARGE`    | Payload exceeds limits or Arena capacity.        | `E2BIG` / `EMSGSIZE` |
| `6`   | `FILAMENT_ERR_OUT_OF_MEMORY`     | Arena exhaustion (Hard limit).                   | `ENOMEM`             |
| `7`   | `FILAMENT_ERR_RESOURCE_BUSY`     | Resource locked or currently in use.             | `EBUSY`              |
| `8`   | `FILAMENT_ERR_MEMORY_ACCESS`     | Bad pointer, alignment, or overflow detected.    | `EFAULT`             |
| `9`   | `FILAMENT_ERR_INVALID_ARGUMENT`  | Logic error, bad enum, or malformed data.        | `EINVAL`             |
| `10`  | `FILAMENT_ERR_TIMED_OUT`         | Execution deadline exceeded (Preemptive).        | `ETIMEDOUT`          |
| `11`  | `FILAMENT_ERR_INTERNAL`          | Unrecoverable Host internal error.               | -                    |
| `12`  | `FILAMENT_ERR_PADDING`           | Non-zero padding bytes detected (Security risk). | -                    |
| `13`  | `FILAMENT_ERR_VERSION_MISMATCH`  | ABI version incompatible.                        | -                    |

### 3.3 Result Codes

| Value | Symbol                  | Description                                                                                                                                              |
| :---- | :---------------------- | :------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `0`   | `FILAMENT_RESULT_DONE`  | Weave completed successfully. Agent is idle.                                                                                                             |
| `1`   | `FILAMENT_RESULT_YIELD` | Weave incomplete. Host **SHOULD** resume immediately only if Compute Budget permits. Otherwise, Host **MUST** defer resumption to next scheduling cycle. |
| `2`   | `FILAMENT_RESULT_PANIC` | Logic assertion failure. Host **SHOULD** capture state for debugging but treat execution as failed.                                                      |
| `-1`  | `FILAMENT_RESULT_ERROR` | Fatal runtime error. Host **SHOULD** append `filament.sys.error` to timeline.                                                                            |

### 3.4 Value Types

| Value | Symbol                | C Type             |
| :---- | :-------------------- | :----------------- |
| `0`   | `FILAMENT_VAL_UNIT`   | `void` (Null/None) |
| `1`   | `FILAMENT_VAL_BOOL`   | `bool`             |
| `2`   | `FILAMENT_VAL_U64`    | `uint64_t`         |
| `3`   | `FILAMENT_VAL_I64`    | `int64_t`          |
| `4`   | `FILAMENT_VAL_F64`    | `double`           |
| `5`   | `FILAMENT_VAL_U32`    | `uint32_t`         |
| `6`   | `FILAMENT_VAL_I32`    | `int32_t`          |
| `7`   | `FILAMENT_VAL_F32`    | `float`            |
| `8`   | `FILAMENT_VAL_STRING` | `FilamentString`   |
| `9`   | `FILAMENT_VAL_BYTES`  | `FilamentString`   |
| `10`  | `FILAMENT_VAL_MAP`    | `FilamentArray`    |
| `11`  | `FILAMENT_VAL_LIST`   | `FilamentArray`    |
| `12`  | `FILAMENT_VAL_BLOB`   | `FilamentBlobRef`  |

### 3.5 Operation Codes

| Value | Symbol                | Description                   |
| :---- | :-------------------- | :---------------------------- |
| `0`   | `FILAMENT_OP_APPEND`  | Standard insertion.           |
| `1`   | `FILAMENT_OP_REPLACE` | Update previous event.        |
| `2`   | `FILAMENT_OP_DELETE`  | Logical deletion (Tombstone). |

### 3.6 Read Flags

| Value | Symbol                           | Description                            |
| :---- | :------------------------------- | :------------------------------------- |
| `0`   | `FILAMENT_READ_DEFAULT`          | Default behavior.                      |
| `1`   | `FILAMENT_READ_IGNORE_PAYLOADS`  | Copy headers only (no data).           |
| `2`   | `FILAMENT_READ_TRUNCATE`         | Allow partial reads if Arena fills.    |
| `4`   | `FILAMENT_READ_UNSAFE_ZERO_COPY` | **Native Only.** Return Host pointers. |

## 4. Binary Layout and Primitives

### 4.1 Byte Order

All multi-byte integers, including those inside Trace Contexts and Standard Structures, **MUST** use **Little-Endian** encoding.

Wire formats or network protocols that use Big-Endian or Network Byte Order **MUST** be byte-swapped by the Host before writing to Filament structures or reading from them.

### 4.2 Fixed Width Types

To ensure layout identity between 32-bit Wasm and 64-bit Native architectures:

| Type Name         | C Definition     | Size | Description                                                                                                                                                                                                          |
| :---------------- | :--------------- | :--- | :------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `FilamentAddress` | `uint64_t`       | 8    | **Memory Reference.** A 64-bit handle or linear memory offset. `0` is reserved as `FILAMENT_NULL`. **Guest Note:** 32-bit guests MUST NOT use native pointers; they MUST pad values to 8 bytes to match this layout. |
| `FilamentMsg`     | `FilamentString` | 16   | String view defined by pointer and length.                                                                                                                                                                           |

**Note:** For Native Plugins supporting Snapshots/Statefulness, `FilamentAddress` **MUST** be treated as an offset relative to the Arena Base, not a virtual pointer. Storing absolute pointers in persisted structures is Undefined Behavior.

### 4.3 Alignment and Padding

- **Alignment:** All structures **MUST** be 8-byte aligned.
- **Padding:** All padding fields are prefixed with an underscore, such as `_pad0`.
- **Initialization:** The Host **MUST** and the Plugin **SHOULD** initialize all padding to `0`.
- **Explicit Padding:** Structures defined in this specification include explicit `_pad` fields to guarantee 8-byte alignment for all 64-bit types, ensuring identical layout on 32-bit and 64-bit architectures without reliance on compiler packing attributes.

### 4.4 Universal Chain Header

Every chainable structure **MUST** begin with the following fields to ensure forward compatibility.

| Offset | Field    | Type              | Description                                                                    |
| :----- | :------- | :---------------- | :----------------------------------------------------------------------------- |
| 0      | `s_type` | `uint32_t`        | Structure Type Enum ID.                                                        |
| 4      | `flags`  | `uint32_t`        | Structure specific flags. Bits undefined by current ABI version **MUST** be 0. |
| 8      | `p_next` | `FilamentAddress` | Address of next struct or 0.                                                   |

### 4.5 Primitives

**FilamentString** (16 bytes)
Non-owning UTF-8 view. Strings are **NOT** null-terminated. They are length-prefixed views. C Plugins **MUST** use length-aware functions such as `printf("%.*s", len, ptr)` or copy data to a local null-terminated buffer.

| Offset | Field | Type              | Description             |
| :----- | :---- | :---------------- | :---------------------- |
| 0      | `ptr` | `FilamentAddress` | Address of UTF-8 bytes. |
| 8      | `len` | `uint64_t`        | Length in bytes.        |

**Recommendation:** While `FilamentString` is a view, the Host **SHOULD** append a null-terminator byte immediately following the data in the Arena when deep-copying, to facilitate debugging in C-based Plugins. The Host **MUST NOT** write out of bounds of the Arena allocation to add a null byte. Plugins **MUST** rely on `len` as the source of truth.

**FilamentArray** (16 bytes)
Reference to a contiguous list of items.

| Offset | Field   | Type              | Description       |
| :----- | :------ | :---------------- | :---------------- |
| 0      | `ptr`   | `FilamentAddress` | Pointer to items. |
| 8      | `count` | `uint64_t`        | Number of items.  |

**FilamentBlobRef** (16 bytes)
Reference to a large binary asset stored by the Host.

| Offset | Field     | Type       | Description          |
| :----- | :-------- | :--------- | :------------------- |
| 0      | `blob_id` | `uint64_t` | Unique handle.       |
| 8      | `size`    | `uint64_t` | Total size in bytes. |

## 5. Core Data Structures

### 5.1 Trace Context

**FilamentTraceContext** (32 bytes)
Internal representation of distributed tracing identifiers.

| Offset | Field           | Type         | Description                                  |
| :----- | :-------------- | :----------- | :------------------------------------------- |
| 0      | `version`       | `uint8_t`    | Trace Version (Default 0x00).                |
| 1      | `flags`         | `uint8_t`    | Trace Flags (Bit 0 = Sampled).               |
| 2      | `_pad0`         | `uint8_t[6]` | **MUST** be 0. Reserved. (6 bytes explicit). |
| 8      | `trace_id_high` | `uint64_t`   | High 64-bits of Trace ID.                    |
| 16     | `trace_id_low`  | `uint64_t`   | Low 64-bits of Trace ID.                     |
| 24     | `span_id`       | `uint64_t`   | Parent Span ID.                              |

### 5.2 Generic Value

**FilamentValue** (32 bytes)
A tagged union for primitive types and generic containers. The `data` field is a 24-byte union starting at offset 8. The union **MUST** be explicitly padded to 24 bytes to ensure consistent layout across 32/64-bit compilers and to allow future expansion.

| Offset | Field       | Type              | Description                                           |
| :----- | :---------- | :---------------- | :---------------------------------------------------- |
| 0      | `type`      | `uint32_t`        | Value Type Enum.                                      |
| 4      | `flags`     | `uint32_t`        | Metadata/Subtype flags.                               |
| 8      | `data`      | `union`           | **Union Start.**                                      |
| 8      | `u64_val`   | `uint64_t`        | Union Option: 64-bit unsigned.                        |
| 8      | `i64_val`   | `int64_t`         | Union Option: 64-bit signed.                          |
| 8      | `f64_val`   | `double`          | Union Option: 64-bit float.                           |
| 8      | `u32_val`   | `uint32_t`        | Union Option: 32-bit unsigned. Bytes 12-32 MUST be 0. |
| 8      | `i32_val`   | `int32_t`         | Union Option: 32-bit signed. Bytes 12-32 MUST be 0.   |
| 8      | `f32_val`   | `float`           | Union Option: 32-bit float. Bytes 12-32 MUST be 0.    |
| 8      | `bool_val`  | `uint8_t`         | Union Option: Boolean. 0=False, 1=True.               |
| 8      | `str_val`   | `FilamentString`  | Union Option: String View. Occupies bytes 8-24.       |
| 8      | `bytes_val` | `FilamentString`  | Union Option: Binary View. Occupies bytes 8-24.       |
| 8      | `map_val`   | `FilamentArray`   | Union Option: Map View (`FilamentPair[]`).            |
| 8      | `list_val`  | `FilamentArray`   | Union Option: List View (`FilamentValue[]`).          |
| 8      | `blob_val`  | `FilamentBlobRef` | Union Option: Blob Reference.                         |
| 8      | `_raw`      | `uint8_t[24]`     | Addressable raw bytes for zero-initialization.        |

**FilamentPair** (48 bytes)
A Key-Value pair for structured data.

| Offset | Field   | Type             | Description       |
| :----- | :------ | :--------------- | :---------------- |
| 0      | `key`   | `FilamentString` | Key (16 bytes).   |
| 16     | `value` | `FilamentValue`  | Value (32 bytes). |

### 5.3 Event Envelope

**FilamentEvent** (192 bytes)
**Alignment:** 16 bytes.
**Baseline Layout:** This structure size is standardized at 192 bytes. Future additions **MUST** use the `p_next` chain or `_ext_handle`.

| Offset | Field               | Type                   | Description                                |
| :----- | :------------------ | :--------------------- | :----------------------------------------- |
| 0      | `s_type`            | `uint32_t`             | **MUST** be `FILAMENT_ST_EVENT`.           |
| 4      | `flags`             | `uint32_t`             | Header flags.                              |
| 8      | `p_next`            | `FilamentAddress`      | Extension chain.                           |
| 16     | `id`                | `uint64_t`             | Unique Event ID.                           |
| 24     | `ref_id`            | `uint64_t`             | Correlation ID.                            |
| 32     | `type_uri`          | `FilamentString`       | Event Type URI (Canonical Type).           |
| 48     | `timestamp`         | `uint64_t`             | Unix Nanoseconds (Wall Clock).             |
| 56     | `tick`              | `uint64_t`             | Logical Tick (Causal Clock).               |
| 64     | `payload_ptr`       | `FilamentAddress`      | Payload Address.                           |
| 72     | `payload_size`      | `uint64_t`             | Payload Size.                              |
| 80     | `auth_agent_id`     | `uint64_t`             | Originating Agent ID.                      |
| 88     | `auth_principal_id` | `uint64_t`             | Originating User/Role ID.                  |
| 96     | `trace_ctx`         | `FilamentTraceContext` | Trace Context (32 bytes).                  |
| 128    | `resource_cost`     | `uint64_t`             | Token/Compute cost metric.                 |
| 136    | `op_code`           | `uint32_t`             | Timeline Operation Enum.                   |
| 140    | `payload_fmt`       | `uint32_t`             | Data Format Enum.                          |
| 144    | `event_flags`       | `uint64_t`             | Bitmask. Bit 1 = Truncated.                |
| 152    | `_ext_handle`       | `uint8_t[16]`          | Reserved for Extension Handles. Init to 0. |
| 168    | `_reserved`         | `uint64_t[3]`          | **Reserved.** Padding for v1.X updates.    |

**Correlation Semantics:**
The `ref_id` field links a response event to its originating request event. When a Plugin emits a request (e.g., HTTP, tool invocation), it **SHOULD** set `ref_id = 0` to indicate "no correlation". The Host **MUST** then populate this field with the committed `event.id` of the request before storing it, and **MUST** use that same value in the response event's `ref_id`. Alternatively, the Plugin **MAY** set `ref_id` to a previous event's ID to establish explicit causality chains.

**Type Identification:**
`type_uri` is the **Canonical Source of Truth** for the event schema. `s_type` is an optimization hint for common types. If the Host does not recognize the `s_type`, it **MUST** inspect `type_uri` to determine the schema or treat the event as generic. Plugins **MUST** set `type_uri` correctly even if `s_type` is used.

### 5.4 Execution Context

**FilamentWeaveInfo** (192 bytes)
Passed to the plugin at the beginning of every execution cycle.
**Baseline Layout:** This structure size is standardized at 192 bytes.

| Offset | Field             | Type                   | Description                              |
| :----- | :---------------- | :--------------------- | :--------------------------------------- |
| 0      | `s_type`          | `uint32_t`             | **MUST** be `FILAMENT_ST_WEAVE_INFO`.    |
| 4      | `flags`           | `uint32_t`             | Header flags.                            |
| 8      | `p_next`          | `FilamentAddress`      | Extension chain.                         |
| 16     | `ctx`             | `FilamentAddress`      | Opaque Host Handle.                      |
| 24     | `time_limit_ns`   | `uint64_t`             | Time budget hint.                        |
| 32     | `resource_used`   | `uint64_t`             | **Compute Units** Consumed.              |
| 40     | `resource_max`    | `uint64_t`             | **Compute Units** Limit.                 |
| 48     | `max_mem_bytes`   | `uint64_t`             | Hard memory limit for Plugin Allocation. |
| 56     | `recursion_depth` | `uint32_t`             | Generic recursion/stack depth.           |
| 60     | `_pad0`           | `uint32_t`             | Padding. Aligns `arena_handle`.          |
| 64     | `arena_handle`    | `FilamentAddress`      | Opaque Allocator Handle.                 |
| 72     | `timeline_len`    | `uint64_t`             | Total history count.                     |
| 80     | `last_event_id`   | `uint64_t`             | Recent event ID.                         |
| 88     | `random_seed`     | `uint64_t`             | Seed for deterministic randomness.       |
| 96     | `current_time`    | `uint64_t`             | Wall-Clock Time (Unix Nanoseconds).      |
| 104    | `trace_ctx`       | `FilamentTraceContext` | Trace Context (32 bytes).                |
| 136    | `weave_flags`     | `uint32_t`             | Cycle Flags.                             |
| 140    | `max_log_bytes`   | `uint32_t`             | Limit on total log volume per cycle.     |
| 144    | `max_event_bytes` | `uint32_t`             | Limit on single event payload size.      |
| 148    | `min_log_level`   | `uint32_t`             | Minimum log level to emit.               |
| 152    | `monotonic_time`  | `uint64_t`             | Host Monotonic Time (Nanoseconds).       |
| 160    | `delta_time_ns`   | `uint64_t`             | Time elapsed since last Weave.           |
| 168    | `_reserved`       | `uint64_t[3]`          | **Reserved.** Padding for v1.X updates.  |

### 5.5 Configuration

**FilamentConfig**
Passed to the Plugin during `create`.
**Alignment:** 8 bytes.

| Offset | Field     | Type              | Description                       |
| :----- | :-------- | :---------------- | :-------------------------------- |
| 0      | `s_type`  | `uint32_t`        | **MUST** be `FILAMENT_ST_CONFIG`. |
| 4      | `flags`   | `uint32_t`        | Header flags.                     |
| 8      | `p_next`  | `FilamentAddress` | Extension chain.                  |
| 16     | `count`   | `uint64_t`        | Number of pairs.                  |
| 24     | `entries` | `FilamentAddress` | Address of `FilamentPair[]`.      |

### 5.6 Host Capabilities

**FilamentHostInfo**
Passed to the Plugin during `create`.
**Alignment:** 8 bytes.

| Offset | Field                 | Type              | Description                                 |
| :----- | :-------------------- | :---------------- | :------------------------------------------ |
| 0      | `s_type`              | `uint32_t`        | **MUST** be `FILAMENT_ST_HOST_INFO`.        |
| 4      | `flags`               | `uint32_t`        | Header flags.                               |
| 8      | `p_next`              | `FilamentAddress` | Extension chain.                            |
| 16     | `supported_formats`   | `uint32_t`        | Bitmask of supported serialization formats. |
| 20     | `max_recursion_depth` | `uint32_t`        | Actual max depth supported by Host.         |
| 24     | `max_graph_nodes`     | `uint64_t`        | Actual max nodes supported by Host.         |
| 32     | `max_arena_bytes`     | `uint64_t`        | Actual max arena size supported by Host.    |

### 5.7 Plugin Manifest

**FilamentPluginInfo**
Returned by `filament_get_info` to allow Host-Plugin negotiation.
**Alignment:** 8 bytes.

| Offset | Field              | Type              | Description                                                                             |
| :----- | :----------------- | :---------------- | :-------------------------------------------------------------------------------------- |
| 0      | `magic`            | `uint32_t`        | **MUST** be `0x9D2F8A41` (High-entropy signature).                                      |
| 4      | `s_type`           | `uint32_t`        | **MUST** be `FILAMENT_ST_PLUGIN`.                                                       |
| 8      | `req_abi_version`  | `uint32_t`        | Minimum ABI version required by Plugin.                                                 |
| 12     | `flags`            | `uint32_t`        | Header flags.                                                                           |
| 16     | `p_next`           | `FilamentAddress` | Extension chain.                                                                        |
| 24     | `min_memory_bytes` | `uint64_t`        | Minimum Arena size required.                                                            |
| 32     | `min_stack_bytes`  | `uint64_t`        | Minimum Stack size required.                                                            |
| 40     | `lookback_hint`    | `uint64_t`        | Optimization Hint. Preferred number of history items to read. Host **MAY** ignore this. |
| 48     | `plugin_name`      | `FilamentString`  | Human-readable name.                                                                    |
| 64     | `plugin_version`   | `FilamentString`  | Human-readable version.                                                                 |

## 6. Host Interface Functions

These functions are exported by the Host environment.

**Safety Requirement:** Host Interface Functions are **Non-Reentrant** and **Terminal**. The Host **MUST NOT** invoke any exported function of the Plugin during the execution of a Host Interface function call. Examples include `malloc` or `free`. The Host **MUST** operate entirely within its own stack and heap context until it returns control to the Plugin.

### 6.1 Check Capability

Checks if the Plugin is permitted to access a named resource or use a specific Standard Library schema.

**Symbol:** `filament_has_capability`

| Parameter | Type              | In/Out | Description                  |
| :-------- | :---------------- | :----- | :--------------------------- |
| `ctx`     | `FilamentAddress` | In     | The Opaque Host Handle.      |
| `uri`     | `FilamentString`  | In     | The Capability URI to check. |

**Valid Usage:**

- `ctx` **MUST** be a valid Host Context handle.
- `uri.ptr` **MUST** point to a valid UTF-8 string.

**Return:** `bool` (0 for False, 1 for True).

### 6.2 Read Timeline

Retrieves a sequence of historical events from the Host's storage.

**Symbol:** `filament_read_timeline`

| Parameter           | Type              | In/Out | Description                                       |
| :------------------ | :---------------- | :----- | :------------------------------------------------ |
| `ctx`               | `FilamentAddress` | In     | The Opaque Host Handle.                           |
| `start_idx`         | `uint64_t`        | In     | The absolute 0-based index to start reading from. |
| `limit`             | `uint64_t`        | In     | Maximum number of events to retrieve.             |
| `flags`             | `uint32_t`        | In     | Read Options (Mask).                              |
| `out_buffer`        | `FilamentAddress` | In     | Pointer to `FilamentEvent[]` (Caller Allocated).  |
| `out_count`         | `FilamentAddress` | Out    | Pointer to `uint64_t` (Count read).               |
| `out_first_idx`     | `FilamentAddress` | Out    | Pointer to `uint64_t` (First index found).        |
| `out_bytes_written` | `FilamentAddress` | Out    | Pointer to `uint64_t` (Bytes consumed in Arena).  |
| `arena`             | `FilamentAddress` | In     | Handle to the Arena allocator.                    |

**Valid Usage:**

- If `out_buffer` is `FILAMENT_NULL` (0):
  - The Host **MUST** calculate the total size required to store `limit` events (including payloads).
  - The Host **MUST** write this size to `out_bytes_written`.
  - The Host **MUST** return `FILAMENT_OK`.
  - This allows the Plugin to query the size and allocate sufficient memory.
- If `out_buffer` is valid:
  - The Host **MUST** write up to `limit` events into the buffer.
  - If `flags` includes `FILAMENT_READ_UNSAFE_ZERO_COPY`, the Plugin **MUST** have negotiated `filament.cap.unsafe_zero_copy`.

**Return Codes:**

- `FILAMENT_OK`: Success.
- `FILAMENT_ENOTFOUND`: `start_idx` is greater than the last event index.
- `FILAMENT_ENOMEM`: Arena is full.

### 6.3 Log

Emits a structured log message to the Host's telemetry system.

**Symbol:** `filament_log`

| Parameter    | Type              | In/Out | Description                                  |
| :----------- | :---------------- | :----- | :------------------------------------------- |
| `ctx`        | `FilamentAddress` | In     | The Opaque Host Handle.                      |
| `level`      | `uint32_t`        | In     | Log Level: 0=Debug, 1=Info, 2=Warn, 3=Error. |
| `msg`        | `FilamentString`  | In     | The primary log message.                     |
| `pairs`      | `FilamentAddress` | In     | Pointer to `FilamentPair[]`.                 |
| `pair_count` | `uint64_t`        | In     | Number of pairs.                             |

**Valid Usage:**

- `pairs` **MUST** point to an array of `FilamentPair` of size `pair_count`.
- `msg.ptr` **MUST** point to valid memory.

**Return:** `void`.

### 6.4 Read Blob

Lazily reads a chunk of a large binary asset.

**Symbol:** `filament_read_blob`

| Parameter | Type              | In/Out | Description                          |
| :-------- | :---------------- | :----- | :----------------------------------- |
| `ctx`     | `FilamentAddress` | In     | The Opaque Host Handle.              |
| `blob_id` | `uint64_t`        | In     | The ID from `FilamentBlob`.          |
| `offset`  | `uint64_t`        | In     | Byte offset to start reading.        |
| `limit`   | `uint64_t`        | In     | Maximum bytes to read.               |
| `out_ptr` | `FilamentAddress` | In     | Destination buffer in Plugin Memory. |
| `out_len` | `FilamentAddress` | Out    | Pointer to `uint64_t` (Bytes Read).  |

**Valid Usage:**

- `blob_id` **MUST** be a valid ID provided by the Host (via Configuration or Event).
- `out_ptr` **MUST** point to a buffer of at least `limit` bytes.
- `out_len` **MUST** point to a valid `uint64_t`.

**Behavior:**

- If `out_ptr` is `NULL`, the Host **MUST** write the remaining blob size (from `offset`) into `out_len` and return `FILAMENT_OK`. This allows Plugins to query size.
- If `offset` is valid but `offset + limit` exceeds the total blob size, the Host **MUST** write the remaining available bytes and return `FILAMENT_OK`.

**Return Codes:**

- `FILAMENT_ENOTFOUND`: Invalid `blob_id`.
- `FILAMENT_EINVALID`: Offset out of bounds.

### 6.5 Blob Create

Allocates a new empty blob container on the Host to prepare for streaming writes.

**Symbol:** `filament_blob_create`

| Parameter   | Type              | In/Out | Description                         |
| :---------- | :---------------- | :----- | :---------------------------------- |
| `ctx`       | `FilamentAddress` | In     | The Opaque Host Handle.             |
| `size_hint` | `uint64_t`        | In     | Expected final size (0 if unknown). |

**Return:** `uint64_t` (The new Blob ID, or 0 on failure).

### 6.6 Blob Write

Streams a chunk of data into a specific blob.

**Symbol:** `filament_blob_write`

| Parameter | Type              | In/Out | Description                                |
| :-------- | :---------------- | :----- | :----------------------------------------- |
| `ctx`     | `FilamentAddress` | In     | The Opaque Host Handle.                    |
| `blob_id` | `uint64_t`        | In     | The ID returned by `filament_blob_create`. |
| `offset`  | `uint64_t`        | In     | Byte offset to write to.                   |
| `data`    | `FilamentAddress` | In     | Pointer to data buffer.                    |
| `len`     | `uint64_t`        | In     | Number of bytes to write.                  |

**Valid Usage:**

- `blob_id` **MUST** be an ID returned by a previous `filament_blob_create` call in the _current_ Weave cycle.
- `data` **MUST** point to valid memory of `len` bytes.
- Blobs created during a Weave are **Sealed** (become immutable) upon the successful Commit of the Weave. Writing to a Sealed blob returns `FILAMENT_EPERM`.

**Return Codes:**

- `FILAMENT_EINVALID`: `blob_id` not found or already finalized.
- `FILAMENT_EPERM`: Blob is sealed (from a previous Weave).

### 6.7 Read KV

Reads a value from the materialized Key-Value store.

**Symbol:** `filament_kv_get`

| Parameter | Type              | In/Out | Description                                              |
| :-------- | :---------------- | :----- | :------------------------------------------------------- |
| `ctx`     | `FilamentAddress` | In     | The Opaque Host Handle.                                  |
| `key`     | `FilamentString`  | In     | The key to look up.                                      |
| `out_ptr` | `FilamentAddress` | In     | Destination buffer in Plugin Memory.                     |
| `out_len` | `FilamentAddress` | In/Out | Pointer to `uint64_t`. Input: Capacity. Output: Written. |

**Valid Usage:**

- `key` **MUST** be a valid string.
- `out_len` **MUST** point to a valid `uint64_t` indicating the buffer capacity.

**Behavior:**

- If `out_ptr` is `NULL` or if `*out_len` is less than the required size:
  - The Host **MUST** write the required size (in bytes) to `*out_len`.
  - The Host **MUST** return `FILAMENT_ENOMEM` (if buffer was too small) or `FILAMENT_OK` (if `out_ptr` was NULL).
- If the key is not found, the Host **MUST** return `FILAMENT_ENOTFOUND`.

**Return:** `int` corresponding to `FilamentErrorCode`.

## 7. Plugin Interface Functions

All Plugins **MUST** export the following symbols. The Host **MUST** validate the presence of these symbols during the loading phase.

### 7.1 Get Plugin Info

Called immediately after loading to negotiate requirements.

**Symbol:** `filament_get_info`

| Parameter | Type   | In/Out | Description    |
| :-------- | :----- | :----- | :------------- |
| `void`    | `void` | -      | No parameters. |

**Return:** `FilamentAddress` (Pointer to static `FilamentPluginInfo`).

### 7.2 Reserve Memory

Called once during initialization. The Plugin must reserve a contiguous block of memory for the Host to use as the Arena.

**Symbol:** `filament_reserve`

| Parameter | Type       | In/Out | Description                              |
| :-------- | :--------- | :----- | :--------------------------------------- |
| `size`    | `uint64_t` | In     | The size in bytes requested by the Host. |

**Return:** `FilamentAddress` (Pointer to the reserved block).
**Constraints:** The returned address **MUST** be valid within the Plugin's address space. The Plugin **MUST NOT** return address `0`.

### 7.3 Create

Called once during initialization to configure the plugin instance.

**Symbol:** `filament_create`

| Parameter | Type              | In/Out | Description                           |
| :-------- | :---------------- | :----- | :------------------------------------ |
| `host`    | `FilamentAddress` | In     | Pointer to `FilamentHostInfo`.        |
| `cfg`     | `FilamentAddress` | In     | Pointer to `FilamentConfig`.          |
| `inst`    | `FilamentAddress` | Out    | Pointer to write the Instance Handle. |

**Constraints:** The Plugin **MUST** deep-copy any configuration data it intends to retain. Pointers contained within `cfg` are not guaranteed to be valid after this function returns.

**Return:** `int` (0 on Success).

### 7.4 Destroy

Called during teardown to free internal resources.

**Symbol:** `filament_destroy`

| Parameter | Type              | In/Out | Description                               |
| :-------- | :---------------- | :----- | :---------------------------------------- |
| `inst`    | `FilamentAddress` | In     | The instance handle returned by `create`. |

**Return:** `void`.

### 7.5 Prepare

Called before a session begins to clear caches or reset transient state.

**Symbol:** `filament_prepare`

| Parameter | Type              | In/Out | Description          |
| :-------- | :---------------- | :----- | :------------------- |
| `inst`    | `FilamentAddress` | In     | The instance handle. |

**Return:** `int` (0 on Success).

### 7.6 Weave

The core execution loop.

**Symbol:** `filament_weave`

| Parameter  | Type              | In/Out | Description                                  |
| :--------- | :---------------- | :----- | :------------------------------------------- |
| `inst`     | `FilamentAddress` | In     | Instance handle.                             |
| `info`     | `FilamentAddress` | In     | Pointer to `FilamentWeaveInfo`.              |
| `out_evts` | `FilamentAddress` | Out    | Pointer to `FilamentAddress` (Output Array). |
| `out_cnt`  | `FilamentAddress` | Out    | Pointer to `uint64_t` (Output Count).        |
| `err_buf`  | `FilamentAddress` | In     | Buffer for writing fatal error messages.     |
| `err_len`  | `uint64_t`        | In     | Size of `err_buf`.                           |

**Valid Usage (Host Side):**

- `info` **MUST** be a valid pointer to `FilamentWeaveInfo`.
- `out_evts` **MUST** point to a `FilamentAddress` where the Plugin will write the address of its output array.
- `out_cnt` **MUST** point to a `uint64_t` where the Plugin will write the count.

**Return:** `FilamentResult` enum.

## 8. Host Implementation Requirements

### 8.1 Validation

To ensure safety in both Sandboxed and Trusted profiles, the Host **MUST** perform Stateless Verification on all data structures before processing.

#### 8.1.1 Low-Memory Protection

The Host **MUST** verify that any `FilamentAddress` provided by the Plugin is either `FILAMENT_NULL` (0) or greater than `FILAMENT_MIN_VALID_OFFSET` (4096).

#### 8.1.2 Cyclic Graph Protection (DoS)

When validating `p_next` chains or object graphs, the Host **MUST** limit traversal node counts to `FILAMENT_MIN_GRAPH_NODES` (or a configured limit) to prevent denial-of-service via infinite loops or bombs. The verification algorithm **SHOULD** be $O(N)$.

#### 8.1.3 Integer Overflow

The Host **MUST** verify that `ptr + len` does not overflow before reading any buffer. `ptr + len` **MUST** be less than or equal to the Arena end address.

#### 8.1.4 Union Type Safety

When reading a `FilamentValue`, the Host **MUST** validate the `data` union member corresponding to the `type` tag. If `type` implies a pointer (String, Map, List), the Host **MUST** validate that pointer.

#### 8.1.5 Atomic Write

When executing `filament_read_timeline`, the Host **MUST** calculate the total required size of all events and payloads before writing any data. If the total exceeds the remaining Arena space, the Host **MUST** return `FILAMENT_ENOMEM` immediately and leave the Arena untouched.

#### 8.1.6 Swizzle Safety

The Host **MUST** verify that every relocated pointer (`p_next`, `payload_ptr`, `ptr`) points to a valid offset within the Plugin's current memory range (Base Address + Linear Memory Size). Pointers outside this range **MUST** trigger a Trap.

#### 8.1.7 Alignment Validation

The Host **MUST** verify that the address for any structure or primitive type respects its natural alignment (e.g., 8-byte alignment for `uint64_t` or pointers). Failure to align data correctly **MUST** result in `FILAMENT_EINVALID`.

#### 8.1.8 Arena Handle Validation

The Host **MUST** verify that the `arena` handle provided in `filament_read_timeline` matches the memory block originally returned by `filament_reserve`.

#### 8.1.9 Forward Compatibility Safety

The Host **MUST** reject any structure where `flags` contains bits undefined in the Host's supported ABI version.

#### 8.1.10 Structure Type Safety

The Host **MUST** reject any structure where `s_type` is unknown or out of the supported range.

### 8.2 Address Space Volatility

For Hosts supporting dynamic memory growth (e.g., Wasm `memory.grow` or JVM Compacting GC), physical addresses may change after any function call into the Plugin. The Host **MUST NOT** cache the physical pointer to the Plugin's memory or the Arena between different Host Function invocations. The Host **MUST** re-acquire the current base pointer from the Execution Context at the start of every Host Function call.

### 8.3 Serialization Order

The Host **MUST** serialize object graphs into the Arena using a **Depth-First, Field-Order** traversal. This strict ordering ensures deterministic memory layout for Plugin validation and zero-copy parsing optimizations.

### 8.4 Zero-Copy Safety

When `FILAMENT_READ_UNSAFE_ZERO_COPY` is active:

1.  **Permission:** The Host **MUST** verify that the Plugin has negotiated the `filament.cap.unsafe_zero_copy` capability.
2.  **Lifetime:** Pointers returned in `payload_ptr` or `ptr` fields point to Host-managed resources. The Host guarantees these pointers remain valid only for the duration of the current `weave()` cycle. Caching them across cycles is Undefined Behavior.
3.  **Read-Only:** Pointers are **Read-Only** by default. Writing to borrowed memory without explicit Capability negotiation is Undefined Behavior.
4.  **Yielding:** Zero-Copy pointers are invalidated by _any_ return of control to the Host, including `FILAMENT_RESULT_YIELD`.
5.  **Persistence:** If a Native Plugin returns a Zero-Copy pointer within an Event payload, the Host **MUST** deep-copy the data referenced by that pointer during the commit phase. Persisting raw pointers to disk is Undefined Behavior.
6.  **Concurrency:** Pointers obtained via `FILAMENT_READ_UNSAFE_ZERO_COPY` **MUST NOT** be accessed by background threads or persisted beyond the return of the `weave` function. Doing so is Undefined Behavior.
7.  **Sandboxed Fallback:** If a Sandboxed Plugin requests this flag, the Host **MUST** silently ignore it and perform a standard copy to the Arena to ensure isolation.

### 8.5 Instance Pooling and Multi-Tenancy

To enable high-density multi-tenancy, Hosts **MAY** maintain a pool of initialized Plugin Instances.

- **Instance Reuse:** The Host **MUST** invoke `filament_prepare` before switching an instance to a different Agent ID.
- **State Clearing:** The Plugin **MUST** reset all Agent-specific state (e.g., session variables) during `filament_prepare`. Failure to do so is a security violation.
- **Pooling Constraint:** Hosts **MUST NOT** pool Native Plugin instances that do not declare `filament.cap.poolable`. Such plugins MUST be unloaded or process-isolated after every Weave.
- **Process Isolation:** For Native Plugins in High-Security or Hard Real-Time environments, Hosts **SHOULD** isolate Plugins via OS Processes if Wasm is not viable, to prevent Priority Inversion or Crash Cascades.

### 8.6 Robotics and Real-Time Safety

To support Hard Real-Time control loops, compliant Hosts **MUST** adhere to the following additional requirements when operating in the **Strict Timing Profile**.

#### 8.6.1 Preemptive Scheduling

The Host **MUST** implement a mechanism to preempt Plugin execution when the `time_limit_ns` budget is exceeded.

- **Wasm:** Instruction Counting or Gas Metering.
- **Native:** Cooperative checks or Process Termination (SIGKILL). Thread cancellation is **NOT** recommended due to resource leaks.

#### 8.6.2 Zero-Allocation Weave

The Host **MUST NOT** perform dynamic heap allocations (`malloc`, `new`) during the execution of `filament_weave` after the initial Arena setup.

#### 8.6.3 Fail-Safe State

For Plugins controlling physical actuators, the Host **MUST** implement a mechanism to transition actuators to a **Safe State** (e.g., zero velocity, engage brakes) immediately if the `weave()` function returns `FILAMENT_RESULT_ERROR`, `FILAMENT_RESULT_TIMEOUT`, or if the Host detects a crash. The Host **MUST NOT** sustain the previous control output across a failed Weave cycle.

---

# Part IV: Standard Schemas & Capabilities

This part defines the normative schemas and capabilities provided by compliant Hosts.

## 1. The Reactor Model

Filament employs a **Reactor Model** for most capabilities. Instead of invoking synchronous Host functions (which may block), Plugins **MAY** emit events describing a requested operation. The Host processes these events asynchronously and appends the result to the Timeline in a subsequent Weave cycle.

## 2. Mandatory System Schemas

All Compliant Hosts **MUST** support the following schemas, as they are required for basic lifecycle and error handling.

### 2.1 System Error

**URI:** `filament.sys.error`

| Offset | Field     | Type              | Description                                    |
| :----- | :-------- | :---------------- | :--------------------------------------------- |
| 0      | `header`  | `ChainHeader`     | `s_type` = `FILAMENT_ST_SYS_ERROR`.            |
| 16     | `code`    | `uint32_t`        | Error code (0=Unknown, 1=Timeout, 2=HTTP).     |
| 20     | `_pad0`   | `uint32_t`        | 0.                                             |
| 24     | `message` | `FilamentString`  | Human-readable error description.              |
| 40     | `details` | `FilamentAddress` | Optional structured details (`FilamentValue`). |

### 2.2 Context Prune

**URI:** `filament.sys.context.prune`

| Offset | Field        | Type          | Description                                 |
| :----- | :----------- | :------------ | :------------------------------------------ |
| 0      | `header`     | `ChainHeader` | `s_type` = `FILAMENT_ST_CTX_PRUNE`.         |
| 16     | `before_idx` | `uint64_t`    | Prune all events with index less than this. |
| 24     | `_pad0`      | `uint64_t`    | 0.                                          |

**Multi-Agent Semantics:** In Multi-Agent Mode, pruning operations affect only events where `auth_agent_id` matches the requesting agent.

### 2.3 Blob Reference

**URI:** `filament.std.blob`

Use with [`filament_read_blob`](#64-read-blob-filament_read_blob) to lazily load content.

| Offset | Field       | Type             | Description                    |
| :----- | :---------- | :--------------- | :----------------------------- |
| 0      | `header`    | `ChainHeader`    | `s_type` = `FILAMENT_ST_BLOB`. |
| 16     | `blob_id`   | `uint64_t`       | Unique Handle.                 |
| 24     | `size`      | `uint64_t`       | Total size in bytes.           |
| 32     | `mime_type` | `FilamentString` | MIME Type.                     |

## 3. Optional Capability Schemas

Hosts **MAY** support the following capabilities. Plugins **MUST** check for support via `filament_has_capability`.

### 3.1 HTTP

**URI:** `filament.std.net.http`

#### 3.1.1 HTTP Request

**URI:** `filament.std.net.http.request`

| Offset | Field          | Type              | Description                        |
| :----- | :------------- | :---------------- | :--------------------------------- |
| 0      | `header`       | `ChainHeader`     | `s_type` = `FILAMENT_ST_HTTP_REQ`. |
| 16     | `method`       | `FilamentString`  | Method.                            |
| 32     | `url`          | `FilamentString`  | URL.                               |
| 48     | `header_count` | `uint32_t`        | Header count.                      |
| 52     | `timeout_ms`   | `uint32_t`        | Timeout in milliseconds.           |
| 56     | `headers`      | `FilamentAddress` | Array of `FilamentPair`.           |
| 64     | `body_type`    | `uint32_t`        | 0=Bytes, 1=Blob.                   |
| 68     | `_pad0`        | `uint32_t`        | 0.                                 |
| 72     | `body_ref`     | `union`           | Union of `ptr` or `blob_id`.       |
| 72     | `ptr`          | `FilamentAddress` | Pointer to bytes.                  |
| 72     | `blob_id`      | `uint64_t`        | Blob ID.                           |
| 80     | `body_len`     | `uint64_t`        | Body len.                          |

#### 3.1.2 HTTP Response

**URI:** `filament.std.net.http.response`

| Offset | Field          | Type              | Description                        |
| :----- | :------------- | :---------------- | :--------------------------------- |
| 0      | `header`       | `ChainHeader`     | `s_type` = `FILAMENT_ST_HTTP_RES`. |
| 16     | `status`       | `uint32_t`        | HTTP Status Code (e.g. 200).       |
| 20     | `header_count` | `uint32_t`        | Header count.                      |
| 24     | `body_type`    | `uint32_t`        | 0=Bytes, 1=Blob.                   |
| 28     | `_pad0`        | `uint32_t`        | 0.                                 |
| 32     | `headers`      | `FilamentAddress` | Array of `FilamentPair`.           |
| 40     | `body_ref`     | `union`           | Union of `ptr` or `blob_id`.       |
| 40     | `ptr`          | `FilamentAddress` | Body bytes.                        |
| 40     | `blob_id`      | `uint64_t`        | Body blob ID.                      |
| 48     | `body_len`     | `uint64_t`        | Body len.                          |
| 56     | `latency_ns`   | `uint64_t`        | Request Latency (Nanoseconds).     |

### 3.2 Tool Use

**URI:** `filament.std.tool`

#### 3.2.1 Tool Definition

**URI:** `filament.std.tool.def`

| Offset | Field          | Type             | Description                        |
| :----- | :------------- | :--------------- | :--------------------------------- |
| 0      | `header`       | `ChainHeader`    | `s_type` = `FILAMENT_ST_TOOL_DEF`. |
| 16     | `name`         | `FilamentString` | Name.                              |
| 32     | `desc`         | `FilamentString` | Description.                       |
| 48     | `schema`       | `FilamentString` | Schema.                            |
| 64     | `input_format` | `uint32_t`       | Format Enum.                       |
| 68     | `_pad0`        | `uint32_t`       | 0.                                 |

#### 3.2.2 Tool Invocation

**URI:** `filament.std.tool.invoke`

| Offset | Field        | Type             | Description                           |
| :----- | :----------- | :--------------- | :------------------------------------ |
| 0      | `header`     | `ChainHeader`    | `s_type` = `FILAMENT_ST_TOOL_INVOKE`. |
| 16     | `tool_name`  | `FilamentString` | Name of tool to call.                 |
| 32     | `input_data` | `FilamentValue`  | Arguments (Map or Bytes).             |
| 64     | `timeout_ms` | `uint32_t`       | Execution timeout.                    |
| 68     | `_pad0`      | `uint32_t`       | 0.                                    |

#### 3.2.3 Tool Result

**URI:** `filament.std.tool.result`

| Offset | Field         | Type             | Description                           |
| :----- | :------------ | :--------------- | :------------------------------------ |
| 0      | `header`      | `ChainHeader`    | `s_type` = `FILAMENT_ST_TOOL_RESULT`. |
| 16     | `tool_name`   | `FilamentString` | Name of tool called.                  |
| 32     | `output_data` | `FilamentValue`  | Result (Map or Bytes).                |
| 64     | `duration_ns` | `uint64_t`       | Execution time.                       |
| 72     | `status`      | `uint32_t`       | 0=Success, 1=Error.                   |
| 76     | `_pad0`       | `uint32_t`       | 0.                                    |

### 3.3 Key-Value Store

**URI:** `filament.std.kv`

**URI:** `filament.std.kv.update`

| Offset | Field       | Type              | Description                         |
| :----- | :---------- | :---------------- | :---------------------------------- |
| 0      | `header`    | `ChainHeader`     | `s_type` = `FILAMENT_ST_KV_UPDATE`. |
| 16     | `key`       | `FilamentString`  | Lookup Key.                         |
| 32     | `mode`      | `uint32_t`        | 0=Overwrite, 1=NoOverwrite.         |
| 36     | `_pad0`     | `uint32_t`        | 0.                                  |
| 40     | `value`     | `FilamentAddress` | Value bytes.                        |
| 48     | `value_len` | `uint64_t`        | Value len.                          |

### 3.4 Environment

**URI:** `filament.std.env`

**URI:** `filament.std.env.get`

| Offset | Field    | Type             | Description                       |
| :----- | :------- | :--------------- | :-------------------------------- |
| 0      | `header` | `ChainHeader`    | `s_type` = `FILAMENT_ST_ENV_GET`. |
| 16     | `key`    | `FilamentString` | Env Var Name.                     |

---

## Appendix A: Glossary [Informative]

| Term                   | Definition                                                                                                                                                                                                                         |
| :--------------------- | :--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **Agent**              | A logical identity consisting of a Timeline and a set of Plugins.                                                                                                                                                                  |
| **Arena**              | A transient bump-pointer allocator provided by the Host, valid only for a single execution turn. The memory backing this allocator **MUST** reside within the Plugin's Address Space.                                              |
| **Blob**               | A large binary asset such as an image, video, or file stored by the Host and accessed lazily by the Plugin via reference.                                                                                                          |
| **Compute Unit**       | An opaque, unitless measurement of resource consumption used to prevent infinite loops and resource exhaustion.                                                                                                                    |
| **Host**               | The runtime environment responsible for loading Plugins and enforcing the safety contract.                                                                                                                                         |
| **Plugin**             | The immutable binary artifact — e.g. a Wasm module or Shared Object — that implements the Filament Interface.                                                                                                                      |
| **Timeline**           | The immutable, strictly ordered sequence of events representing the Agent's history.                                                                                                                                               |
| **Timeline Isolation** | The guarantee that a Timeline belongs to a specific, unique Agent ID. While specific implementation is up to the Host, swapping the Plugin for a logically distinct task generally results in a new Agent ID and a fresh Timeline. |
| **Weave**              | The atomic unit of execution where the Host transfers control to the Agent.                                                                                                                                                        |

## Appendix B: Core Header

**Filename:** `filament.h`

```c
#ifndef FILAMENT_H
#define FILAMENT_H

#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>

#if defined(_WIN32)
    #ifdef FILAMENT_EXPORTS
        #define FILAMENT_API __declspec(dllexport)
    #else
        #define FILAMENT_API __declspec(dllimport)
    #endif
    #define FILAMENT_CALL __cdecl
#else
    #define FILAMENT_API __attribute__((visibility("default")))
    #define FILAMENT_CALL
#endif

// Compile-time ABI validation
#if defined(__cplusplus)
    #define FILAMENT_ASSERT(cond, msg) static_assert(cond, msg)
#else
    #define FILAMENT_ASSERT(cond, msg) _Static_assert(cond, msg)
#endif

#if defined(_MSC_VER)
    #define FILAMENT_ALIGN(x) __declspec(align(x))
#else
    #define FILAMENT_ALIGN(x) __attribute__((aligned(x)))
#endif

#define FILAMENT_VER_PACK(ma, mi, pa) (((ma) << 22) | ((mi) << 12) | (pa))
#define FILAMENT_VERSION_0_1_0 FILAMENT_VER_PACK(0, 1, 0)
#define FILAMENT_MAGIC 0x9D2F8A41

#define FILAMENT_MIN_ARENA_BYTES  (64 * 1024 * 1024)
#define FILAMENT_MIN_RECURSION    64
#define FILAMENT_MIN_GRAPH_NODES  4096
#define FILAMENT_MAX_URI_LEN      2048
#define FILAMENT_MIN_VALID_OFFSET 4096

#define FILAMENT_NULL 0

#if defined(__cplusplus)
extern "C" {
#endif

// --------------------------------------------------------------------------
// Fixed Width Handles
// --------------------------------------------------------------------------
typedef uint64_t FilamentAddress;

typedef struct FILAMENT_ALIGN(8) FilamentChainHeader {
    uint32_t        s_type;
    uint32_t        flags;
    FilamentAddress p_next;
} FilamentChainHeader;

FILAMENT_ASSERT(sizeof(FilamentChainHeader) == 16, "Header size mismatch");

// --------------------------------------------------------------------------
// Enumerations
// --------------------------------------------------------------------------
typedef enum FilamentResult {
    FILAMENT_RESULT_DONE    = 0,
    FILAMENT_RESULT_YIELD   = 1,
    FILAMENT_RESULT_PANIC   = 2,
    FILAMENT_RESULT_ERROR   = -1
} FilamentResult;

typedef enum FilamentErrorCode {
    FILAMENT_OK                     = 0,
    FILAMENT_ERR_PERMISSION_DENIED  = 1,
    FILAMENT_ERR_NOT_FOUND          = 2,
    FILAMENT_ERR_IO_FAILURE         = 3,
    FILAMENT_ERR_NOT_CONFIGURED     = 4,
    FILAMENT_ERR_DATA_TOO_LARGE     = 5,
    FILAMENT_ERR_OUT_OF_MEMORY      = 6,
    FILAMENT_ERR_RESOURCE_BUSY      = 7,
    FILAMENT_ERR_MEMORY_ACCESS      = 8,
    FILAMENT_ERR_INVALID_ARGUMENT   = 9,
    FILAMENT_ERR_TIMED_OUT          = 10,
    FILAMENT_ERR_INTERNAL           = 11,
    FILAMENT_ERR_PADDING            = 12,
    FILAMENT_ERR_VERSION_MISMATCH   = 13
} FilamentErrorCode;

typedef enum FilamentValueType {
    FILAMENT_VAL_UNIT   = 0,
    FILAMENT_VAL_BOOL   = 1,
    FILAMENT_VAL_U64    = 2,
    FILAMENT_VAL_I64    = 3,
    FILAMENT_VAL_F64    = 4,
    FILAMENT_VAL_U32    = 5,
    FILAMENT_VAL_I32    = 6,
    FILAMENT_VAL_F32    = 7,
    FILAMENT_VAL_STRING = 8,
    FILAMENT_VAL_BYTES  = 9,
    FILAMENT_VAL_MAP    = 10,
    FILAMENT_VAL_LIST   = 11,
    FILAMENT_VAL_BLOB   = 12
} FilamentValueType;

typedef enum FilamentOpCode {
    FILAMENT_OP_APPEND   = 0,
    FILAMENT_OP_REPLACE  = 1,
    FILAMENT_OP_DELETE   = 2
} FilamentOpCode;

typedef enum FilamentReadFlags {
    FILAMENT_READ_DEFAULT         = 0,
    FILAMENT_READ_IGNORE_PAYLOADS = 1,
    FILAMENT_READ_TRUNCATE        = 2,
    FILAMENT_READ_UNSAFE_ZERO_COPY = 4
} FilamentReadFlags;

typedef enum FilamentDataFormat {
    FILAMENT_FMT_JSON   = 0,
    FILAMENT_FMT_UTF8   = 1,
    FILAMENT_FMT_BYTES  = 2,
    FILAMENT_FMT_STRUCT = 3,
    FILAMENT_FMT_VALUE  = 4
} FilamentDataFormat;

typedef enum FilamentWeaveFlags {
    FILAMENT_WEAVE_FLAG_NEW_VERSION = 1
} FilamentWeaveFlags;

typedef enum FilamentEventFlags {
    FILAMENT_EVENT_FLAG_TRUNCATED = 2
} FilamentEventFlags;

// --------------------------------------------------------------------------
// Core Structures
// --------------------------------------------------------------------------

typedef struct FILAMENT_ALIGN(8) FilamentString {
    FilamentAddress ptr;
    uint64_t        len;
} FilamentString;

typedef struct FILAMENT_ALIGN(8) FilamentArray {
    FilamentAddress ptr;
    uint64_t        count;
} FilamentArray;

typedef struct FILAMENT_ALIGN(8) FilamentBlobRef {
    uint64_t blob_id;
    uint64_t size;
} FilamentBlobRef;

typedef struct FILAMENT_ALIGN(8) FilamentTraceContext {
    uint8_t  version;
    uint8_t  flags;
    uint8_t  _pad0[6];
    uint64_t trace_id_high;
    uint64_t trace_id_low;
    uint64_t span_id;
} FilamentTraceContext;

FILAMENT_ASSERT(sizeof(FilamentTraceContext) == 32, "TraceContext size mismatch");

typedef struct FILAMENT_ALIGN(8) FilamentValue {
    uint32_t type;
    uint32_t flags;
    union FILAMENT_ALIGN(8) {
        uint64_t        u64_val;
        int64_t         i64_val;
        double          f64_val;
        uint32_t        u32_val;
        int32_t         i32_val;
        float           f32_val;
        uint8_t         bool_val;
        FilamentString  str_val;
        FilamentString  bytes_val;
        FilamentArray   map_val;
        FilamentArray   list_val;
        FilamentBlobRef blob_val;
        uint8_t         _raw[24]; // Explicit Union Size & Zeroing Field
    } data;
} FilamentValue;

typedef struct FILAMENT_ALIGN(8) FilamentPair {
    FilamentString key;
    FilamentValue  value;
} FilamentPair;

FILAMENT_ASSERT(sizeof(FilamentString) == 16, "String size mismatch");
FILAMENT_ASSERT(sizeof(FilamentArray) == 16, "Array size mismatch");
FILAMENT_ASSERT(sizeof(FilamentValue) == 32, "Value size mismatch");
FILAMENT_ASSERT(offsetof(FilamentValue, data) == 8, "Value Union offset mismatch");
FILAMENT_ASSERT(sizeof(FilamentPair) == 48, "Pair size mismatch");

typedef struct FILAMENT_ALIGN(16) FilamentEvent {
    uint32_t             s_type;
    uint32_t             flags;
    FilamentAddress      p_next;
    uint64_t             id;
    uint64_t             ref_id;
    FilamentString       type_uri;
    uint64_t             timestamp;        // Wall Clock
    uint64_t             tick;             // Logical Clock
    FilamentAddress      payload_ptr;
    uint64_t             payload_size;
    uint64_t             auth_agent_id;
    uint64_t             auth_principal_id;
    FilamentTraceContext trace_ctx;
    uint64_t             resource_cost;
    uint32_t             op_code;
    uint32_t             payload_fmt;
    uint64_t             event_flags;
    uint8_t              _ext_handle[16];
    uint64_t             _reserved[3];
} FilamentEvent;

FILAMENT_ASSERT(sizeof(FilamentEvent) == 192, "Event size mismatch");

typedef struct FILAMENT_ALIGN(16) FilamentWeaveInfo {
    FilamentChainHeader  header;
    FilamentAddress      ctx;
    uint64_t             time_limit_ns;
    uint64_t             resource_used;
    uint64_t             resource_max;
    uint64_t             max_mem_bytes;
    uint32_t             recursion_depth;
    uint32_t             _pad0;
    FilamentAddress      arena_handle;
    uint64_t             timeline_len;
    uint64_t             last_event_id;
    uint64_t             random_seed;
    uint64_t             current_time;
    FilamentTraceContext trace_ctx;
    uint32_t             weave_flags;
    uint32_t             max_log_bytes;
    uint32_t             max_event_bytes;
    uint32_t             min_log_level;
    uint64_t             monotonic_time;
    uint64_t             delta_time_ns;
    uint64_t             _reserved[3];
} FilamentWeaveInfo;

FILAMENT_ASSERT(sizeof(FilamentWeaveInfo) == 192, "WeaveInfo size mismatch");

typedef struct FILAMENT_ALIGN(8) FilamentHostInfo {
    FilamentChainHeader header;
    uint32_t            supported_formats;
    uint32_t            max_recursion_depth;
    uint64_t            max_graph_nodes;
    uint64_t            max_arena_bytes;
} FilamentHostInfo;

typedef struct FILAMENT_ALIGN(8) FilamentConfig {
    FilamentChainHeader header;
    uint64_t            count;
    FilamentAddress     entries; // FilamentPair*
} FilamentConfig;

typedef struct FILAMENT_ALIGN(8) FilamentPluginInfo {
    uint32_t            magic;
    uint32_t            s_type;
    uint32_t            req_abi_version;
    uint32_t            flags;
    FilamentAddress     p_next;
    uint64_t            min_memory_bytes;
    uint64_t            min_stack_bytes;
    uint64_t            lookback_hint;
    FilamentString      plugin_name;
    FilamentString      plugin_version;
} FilamentPluginInfo;

FILAMENT_ASSERT(sizeof(FilamentPluginInfo) == 80, "PluginInfo size mismatch");

// --------------------------------------------------------------------------
// Plugin Exports
// --------------------------------------------------------------------------

typedef FilamentAddress (FILAMENT_CALL *PFN_FilamentGetInfo)(void);

typedef FilamentAddress (FILAMENT_CALL *PFN_FilamentReserve)(uint64_t size);

typedef int (FILAMENT_CALL *PFN_FilamentCreate)(
    const FilamentHostInfo* host,
    const FilamentConfig* cfg,
    FilamentAddress* inst);

typedef void (FILAMENT_CALL *PFN_FilamentDestroy)(FilamentAddress inst);

typedef int (FILAMENT_CALL *PFN_FilamentPrepare)(FilamentAddress inst);

typedef FilamentResult (FILAMENT_CALL *PFN_FilamentWeave)(
    FilamentAddress inst,
    const FilamentWeaveInfo* info,
    FilamentAddress* out_evts,
    uint64_t* out_cnt,
    FilamentAddress err_buf,
    uint64_t err_len);

typedef uint64_t (FILAMENT_CALL *PFN_FilamentSnapshot)(
    FilamentAddress inst,
    FilamentAddress ctx);

typedef int (FILAMENT_CALL *PFN_FilamentRestore)(
    FilamentAddress inst,
    FilamentAddress ctx,
    uint64_t blob_id);

typedef int (FILAMENT_CALL *PFN_FilamentReadTimeline)(
    FilamentAddress ctx,
    uint64_t start_idx,
    uint64_t limit,
    uint32_t flags,
    FilamentAddress out_buffer,
    FilamentAddress out_count,
    FilamentAddress out_first_idx,
    FilamentAddress out_bytes_written,
    FilamentAddress arena
);

typedef int (FILAMENT_CALL *PFN_FilamentReadBlob)(
    FilamentAddress ctx,
    uint64_t blob_id,
    uint64_t offset,
    uint64_t limit,
    FilamentAddress out_ptr,
    FilamentAddress out_len
);

typedef uint64_t (FILAMENT_CALL *PFN_FilamentBlobCreate)(
    FilamentAddress ctx,
    uint64_t size_hint
);

typedef int (FILAMENT_CALL *PFN_FilamentBlobWrite)(
    FilamentAddress ctx,
    uint64_t blob_id,
    uint64_t offset,
    FilamentAddress data,
    uint64_t len
);

typedef int (FILAMENT_CALL *PFN_FilamentKVGet)(
    FilamentAddress ctx,
    FilamentString key,
    FilamentAddress out_ptr,
    FilamentAddress out_len
);

typedef void (FILAMENT_CALL *PFN_FilamentLog)(
    FilamentAddress ctx,
    uint32_t level,
    FilamentString msg,
    FilamentAddress pairs,
    uint64_t pair_count
);

#if defined(__cplusplus)
}
#endif
#endif // FILAMENT_H
```

## Appendix C: Standard Library Header

**Filename:** `filament_std.h`

```c
#ifndef FILAMENT_STD_H
#define FILAMENT_STD_H

#include "filament.h"

#if defined(__cplusplus)
extern "C" {
#endif

// --------------------------------------------------------------------------
// Constants: Capabilities & URIs
// --------------------------------------------------------------------------

#define FILAMENT_CAP_NET_HTTP    "filament.std.net.http"
#define FILAMENT_CAP_TOOL        "filament.std.tool"
#define FILAMENT_CAP_KV          "filament.std.kv"
#define FILAMENT_CAP_ENV         "filament.std.env"
#define FILAMENT_CAP_STATEFUL    "filament.cap.stateful"
#define FILAMENT_CAP_ZERO_COPY   "filament.cap.unsafe_zero_copy"
#define FILAMENT_CAP_POOLABLE    "filament.cap.poolable"

#define FILAMENT_URI_CTX_PRUNE   "filament.sys.context.prune"
#define FILAMENT_URI_SYS_ERROR   "filament.sys.error"
#define FILAMENT_URI_HTTP_REQ    "filament.std.net.http.request"
#define FILAMENT_URI_HTTP_RES    "filament.std.net.http.response"
#define FILAMENT_URI_TOOL_DEF    "filament.std.tool.def"
#define FILAMENT_URI_TOOL_INVOKE "filament.std.tool.invoke"
#define FILAMENT_URI_TOOL_RESULT "filament.std.tool.result"
#define FILAMENT_URI_KV_UPDATE   "filament.std.kv.update"
#define FILAMENT_URI_ENV_GET     "filament.std.env.get"
#define FILAMENT_URI_BLOB        "filament.std.blob"

// --------------------------------------------------------------------------
// ID Ranges
// --------------------------------------------------------------------------
// Core: 0-99
// Std:  100-999
// User: 1000+

#define FILAMENT_ST_SYS_ERROR     101
#define FILAMENT_ST_CONTEXT_PRUNE 102
#define FILAMENT_ST_HTTP_REQ      200
#define FILAMENT_ST_HTTP_RES      201
#define FILAMENT_ST_TOOL_DEF      300
#define FILAMENT_ST_TOOL_INVOKE   302
#define FILAMENT_ST_TOOL_RESULT   303
#define FILAMENT_ST_KV_UPDATE     400
#define FILAMENT_ST_ENV_GET       500
#define FILAMENT_ST_BLOB          600

// --------------------------------------------------------------------------
// Standard Structs
// --------------------------------------------------------------------------

typedef struct FILAMENT_ALIGN(8) FilamentSystemError {
    FilamentChainHeader header;
    uint32_t            code;
    uint32_t            _pad0;
    FilamentString      message;
    FilamentAddress     details;
} FilamentSystemError;

typedef struct FILAMENT_ALIGN(8) FilamentContextPrune {
    FilamentChainHeader header;
    uint64_t            before_idx;
    uint64_t            _pad0;
} FilamentContextPrune;

#define FILAMENT_BODY_BYTES 0
#define FILAMENT_BODY_BLOB  1

typedef struct FILAMENT_ALIGN(8) FilamentHttpRequest {
    FilamentChainHeader header;
    FilamentString      method;
    FilamentString      url;
    uint32_t            header_count;
    uint32_t            timeout_ms;
    FilamentAddress     headers;
    uint32_t            body_type;
    uint32_t            _pad0;
    union {
        FilamentAddress ptr;
        uint64_t        blob_id;
    } body_ref;
    uint64_t            body_len;
} FilamentHttpRequest;

typedef struct FILAMENT_ALIGN(8) FilamentHttpResponse {
    FilamentChainHeader header;
    uint32_t            status;
    uint32_t            header_count;
    uint32_t            body_type;
    uint32_t            _pad0;
    FilamentAddress     headers;
    union {
        FilamentAddress ptr;
        uint64_t        blob_id;
    } body_ref;
    uint64_t            body_len;
    uint64_t            latency_ns;
} FilamentHttpResponse;

typedef struct FILAMENT_ALIGN(8) FilamentToolDefinition {
    FilamentChainHeader header;
    FilamentString      name;
    FilamentString      description;
    FilamentString      input_schema;
    uint32_t            input_format;
    uint32_t            _pad0;
} FilamentToolDefinition;

typedef struct FILAMENT_ALIGN(8) FilamentToolInvoke {
    FilamentChainHeader header;
    FilamentString      tool_name;
    FilamentValue       input_data;
    uint32_t            timeout_ms;
    uint32_t            _pad0;
} FilamentToolInvoke;

typedef struct FILAMENT_ALIGN(8) FilamentToolResult {
    FilamentChainHeader header;
    FilamentString      tool_name;
    FilamentValue       output_data;
    uint64_t            duration_ns;
    uint32_t            status;
    uint32_t            _pad0;
} FilamentToolResult;

typedef enum FilamentKVUpdateMode {
    FILAMENT_KV_OVERWRITE    = 0,
    FILAMENT_KV_NO_OVERWRITE = 1
} FilamentKVUpdateMode;

typedef struct FILAMENT_ALIGN(8) FilamentKVUpdate {
    FilamentChainHeader header;
    FilamentString      key;
    uint32_t            mode;
    uint32_t            _pad0;
    FilamentAddress     value;
    uint64_t            value_len;
} FilamentKVUpdate;

typedef struct FILAMENT_ALIGN(8) FilamentEnvGet {
    FilamentChainHeader header;
    FilamentString      key;
} FilamentEnvGet;

typedef struct FILAMENT_ALIGN(8) FilamentBlob {
    FilamentChainHeader header;
    uint64_t            blob_id;
    uint64_t            size;
    FilamentString      mime_type;
} FilamentBlob;

// Assertions updated to match actual layout
FILAMENT_ASSERT(sizeof(FilamentSystemError) == 48, "SystemError size mismatch");
FILAMENT_ASSERT(sizeof(FilamentContextPrune) == 32, "ContextPrune size mismatch");
FILAMENT_ASSERT(sizeof(FilamentHttpRequest) == 88, "HttpRequest size mismatch");
FILAMENT_ASSERT(sizeof(FilamentHttpResponse) == 64, "HttpResponse size mismatch");
FILAMENT_ASSERT(sizeof(FilamentToolDefinition) == 72, "ToolDefinition size mismatch");
FILAMENT_ASSERT(sizeof(FilamentToolInvoke) == 72, "ToolInvoke size mismatch");
FILAMENT_ASSERT(sizeof(FilamentToolResult) == 80, "ToolResult size mismatch");
FILAMENT_ASSERT(sizeof(FilamentKVUpdate) == 56, "KVUpdate size mismatch");
FILAMENT_ASSERT(sizeof(FilamentBlob) == 48, "Blob size mismatch");

#if defined(__cplusplus)
}
#endif
#endif // FILAMENT_STD_H
```

## Appendix D: SDK Helpers [Informative]

**Filename:** `filament_sdk.h`

```c
#ifndef FILAMENT_SDK_H
#define FILAMENT_SDK_H

#include <stdint.h>

// Casts a local pointer to a FilamentAddress (Host Handle)
// Usage: evt.payload_ptr = FILAMENT_TO_ADDR(my_local_struct_ptr);
#define FILAMENT_TO_ADDR(ptr) ((FilamentAddress)(uintptr_t)(ptr))

// Casts a FilamentAddress (Host Handle) to a local pointer
// Usage: MyStruct* s = (MyStruct*)FILAMENT_FROM_ADDR(addr);
#define FILAMENT_FROM_ADDR(addr) ((void*)(uintptr_t)(addr))

// Helper for Native Hosts to read the Plugin Info struct
// Usage: FilamentPluginInfo* info = FILAMENT_GET_INFO_PTR(filament_get_info());
#define FILAMENT_GET_INFO_PTR(addr) ((FilamentPluginInfo*)(uintptr_t)(addr))

// Boilerplate macro for exporting mandatory plugin symbols
// Usage: FILAMENT_DEFINE_PLUGIN(my_create, my_weave, ...)
#define FILAMENT_DEFINE_PLUGIN(fn_create, fn_weave, fn_reserve, fn_destroy, fn_prepare, fn_info) \
    FILAMENT_API FilamentAddress FILAMENT_CALL filament_reserve(uint64_t size) { return fn_reserve(size); } \
    FILAMENT_API int FILAMENT_CALL filament_create(const FilamentHostInfo* h, const FilamentConfig* c, FilamentAddress* i) { return fn_create(h, c, i); } \
    FILAMENT_API void FILAMENT_CALL filament_destroy(FilamentAddress i) { fn_destroy(i); } \
    FILAMENT_API int FILAMENT_CALL filament_prepare(FilamentAddress i) { return fn_prepare(i); } \
    FILAMENT_API FilamentResult FILAMENT_CALL filament_weave(FilamentAddress i, const FilamentWeaveInfo* w, FilamentAddress* o, uint64_t* c, FilamentAddress e, uint64_t l) { return fn_weave(i, w, o, c, e, l); } \
    FILAMENT_API FilamentAddress FILAMENT_CALL filament_get_info(void) { return fn_info(); }

// Helper to initialize a string view
// Usage: FilamentString s = FILAMENT_LIT("Hello");
#define FILAMENT_LIT(s) (FilamentString){ .ptr = FILAMENT_TO_ADDR(s), .len = sizeof(s)-1 }

// Helper to expand FilamentString for printf
// Usage: printf("Key: %.*s\n", FILAMENT_STR_FMT(key));
#define FILAMENT_STR_FMT(s) (int)((s).len), (const char*)((s).ptr)

// Helper to zero-initialize a structure including padding
// Usage: FILAMENT_INIT_STRUCT(&my_event);
#define FILAMENT_INIT_STRUCT(ptr) \
    do { \
        unsigned char* p = (unsigned char*)(ptr); \
        for(size_t i=0; i<sizeof(*(ptr)); ++i) p[i] = 0; \
    } while(0)

#endif // FILAMENT_SDK_H
```

## Appendix E: Conformance & Verification [Informative]

This section outlines the requirements for verifying a Filament Host implementation.

### E.1 The Host Compliance Kit (HCK)

The Filament Project maintains a normative **Host Compliance Kit (HCK)** consisting of a suite of hostile Plugin artifacts (Wasm and Native). A Host is considered Compliant if and only if it successfully executes the HCK test suite without:

1.  Crashing the Host process.
2.  Leaking memory between Weave cycles.
3.  Violating any `MUST` directive defined in Section 8.

### E.2 Test Coverage

The HCK validates specific safety constraints including:

- **hck_01_alignment:** Sends misaligned pointers. Host MUST return `FILAMENT_ERR_MEMORY_ACCESS`.
- **hck_02_overflow:** Sends `ptr + len` wrapping around `UINT64_MAX`. Host MUST return `FILAMENT_ERR_MEMORY_ACCESS`.
- **hck_03_graph_bomb:** Sends a cyclic `p_next` chain. Host MUST return error or abort safely.
- **hck_04_bad_enum:** Sends undefined `s_type` values. Host MUST reject the structure.
- **hck_05_oom:** Requests a 1GB event. Host MUST handle the allocation failure gracefully.
- **hck_06_zero_copy_yield:** Attempts to access a Zero-Copy pointer after yielding. Host MUST detect access violation or document unsafe behavior.
- **hck_07_pool_leak:** Tests if state persists across `prepare` calls in a pooled environment.
