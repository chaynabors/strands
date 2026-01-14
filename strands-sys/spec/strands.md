# Strands Specification 0.1.0

**Status:** Draft (Stable Candidate)
**Date:** 2026-01-13
**Distribution:** Public

## Table of Contents

1.  [**Key Words**](#1-key-words)
2.  [**Versioning Strategy**](#2-versioning-strategy)
3.  [**Asynchronous Execution**](#3-asynchronous-execution)
4.  [**Timeline**](#4-timeline)
5.  [**Binary Interface**](#5-binary-interface)
6.  [**Memory Model**](#6-memory-model)
7.  [**Host Interface**](#7-host-interface)
8.  [**Data Structure Reference**](#8-data-structure-reference)
9.  [**Standard Library Capabilities**](#9-standard-library-capabilities)
10. [**Safety Contract & Undefined Behavior**](#10-safety-contract--undefined-behavior)
11. [**Appendix A: Core Header**](#appendix-a-core-header)
12. [**Appendix B: Standard Library Header**](#appendix-b-standard-library-header)
13. [**Appendix C: Canonical Binary Layouts**](#appendix-c-canonical-binary-layouts)

---

## 1. Key Words

The key words "MUST", "MUST NOT", "REQUIRED", "SHALL", "SHALL NOT", "SHOULD", "SHOULD NOT", "RECOMMENDED", "NOT RECOMMENDED", "MAY", and "OPTIONAL" in this document are to be interpreted as described in BCP 14 [RFC2119] [RFC8174] when, and only when, they appear in all capitals, as shown here.

## 2. Versioning Strategy

Both Strands Core and the Standard Library adhere to **Semantic Versioning 2.0.0**.

### 2.1 Bit Packed Versioning

To ensure efficient ABI comparisons at runtime, version numbers are packed into a single 32-bit integer. This allows Hosts to verify Plugin compatibility with a single bitwise operation.

| Field     | Bits | Range  | Shift   |
| :-------- | :--- | :----- | :------ |
| **Major** | 10   | 0–1023 | `<< 22` |
| **Minor** | 10   | 0–1023 | `<< 12` |
| **Patch** | 12   | 0–4095 | `<< 0`  |

**Compatibility Check:** A Plugin is compatible if `(host.major == plugin.major) && (host.minor >= plugin.minor)`.

---

## 3. Asynchronous Execution

Strands employs a non-blocking reactive state machine model. The Agent does not run continuously; it reacts to wake signals, processes a slice of work, and yields control back to the Host.

### 3.1 The Weave Cycle

The `weave` function is the primary entry point for execution. It defines a single transition of the state machine.

1.  **Invocation:** The Host invokes `weave` with the **[StrandsWeaveInfo](#83-execution-context-strandsweaveinfo)** structure and a buffer for error reporting.
2.  **Processing:** The Plugin analyzes the Timeline.
3.  **Result:** The Plugin returns a `StrandsResult` code:
    - `STRANDS_RESULT_SUCCESS`: The turn is complete. The Plugin returns an array of output events.
    - `STRANDS_RESULT_PENDING`: The Plugin has started a background task and yields execution.
    - `STRANDS_RESULT_ERROR`: A fatal error (crash/panic) occurred. The Plugin MUST write a descriptive message to `error_buf`.

**Error Safety:** To prevent reading from corrupted Plugin memory during a crash, the Host provides a fixed-size buffer (`error_buf`, `error_buf_size`) for error messages. The Plugin writes directly to this Host-owned memory.

### 3.2 Wake Signaling

If a Plugin returns `STRANDS_RESULT_PENDING`, it assumes responsibility for resuming execution via the Host-provided worker mechanism.

1.  **Spawn:** The Plugin prepares a **Flat** data packet (no internal pointers) and calls `host->spawn_worker(ctx, worker_id, flat_data, flat_len)`.
    - _Note:_ `worker_id` is a Plugin-defined **Correlation Tag**. It is not a unique system handle. The Host preserves this ID and returns it in the next `weave` cycle to help the Plugin identify which task completed.
2.  **Execution:** The Host block-copies the data and runs `plugin->run_worker` on a background thread.
3.  **Completion:** When the worker finishes, it calls `host->wake(ctx, reason, result_data, result_len)`.
4.  **Resumption:** The Host copies `result_data` to a main-thread buffer and schedules a new call to `weave`. This result is passed to the Plugin via `StrandsWeaveInfo::signal_data`.

**Data Safety:** This "Loop-Back" design ensures the Plugin instance remains stateless regarding thread synchronization. The Plugin receives the worker's result as an input to the next Weave cycle.

---

## 4. Timeline

The **Timeline** is the mutable history of the Agent's existence. The Timeline allows the Agent to manage context window limits through self-truncation or summarization.

### 4.1 Immutable Event IDs

Every event ingested into the Timeline MUST be assigned a **Unique, Monotonic 64-bit ID** by the Host.

- **Persistence:** Once assigned, an ID MUST NOT change.
- **Addressing:** All Timeline mutations MUST reference events by their ID.

### 4.2 Mutation Operations

To modify the Timeline, the Plugin emits events with specific `op_code` values.

- **Transient Commands:** Events with an `op_code` other than `STRANDS_OP_APPEND` are treated as **Transient Commands**. They are consumed by the Host to modify the Timeline state and are **NOT** appended to the Timeline history.
- **Data Events:** Events with `STRANDS_OP_APPEND` are added to the end of the Timeline.
- **Latency:** Mutations are applied by the Host after the current `weave` cycle completes and before the next cycle begins.
- **Safety:** A Plugin MUST NOT modify the Timeline during the read phase of the `weave` cycle.

**Performance Note:** `STRANDS_OP_REPLACE` and `STRANDS_OP_DELETE` (for non-tail events) may require the Host to invalidate KV Caches in LLM inference engines. Plugins SHOULD prefer Append-only workflows or Tail-Truncation whenever possible to maximize performance.

---

## 5. Binary Interface

The Strands ABI is designed for strict cross-language and cross-architecture compatibility.

### 5.1 Byte Order (Endianness)

All scalar types (integers, pointers, sizes, enumerations) defined in this specification are **Little Endian**.

- This applies to both Host and Plugin.
- **Exception:** `trace_id` and `parent_span_id` arrays are opaque byte sequences that follow W3C standards (Network Byte Order).

### 5.2 Cross-Architecture Compatibility (Wasm)

When a 32-bit Plugin (e.g., WebAssembly) communicates with a 64-bit Host:

1.  **Zero Padding:** The Plugin MUST ensure that all padding bytes generated by `STRANDS_PAD_PTR` and `STRANDS_PAD_SIZE` are strictly set to **zero**. This ensures the Host reads valid 64-bit integers and prevents garbage data from being interpreted as high address bits.
2.  **Pointer Translation:** Pointers passed from Wasm are **Linear Memory Offsets**. The Host MUST treat pointers received from a 32-bit Plugin as offsets relative to the Plugin's linear memory base address, not as valid Host Virtual Addresses.

---

## 6. Memory Model

Strands employs a **Hybrid Arena** model to balance performance and safety.

### 6.1 Host Arena

The Host provides a linear bump-pointer allocator via `info->arena_alloc`.

- **Usage:** Plugins SHOULD use this for temporary event allocations, output arrays, and short-lived strings.
- **Lifetime:** Memory allocated in the Arena is valid **ONLY** until the `weave` function returns.
- **Reset:** The Host resets the Arena immediately after the `weave` call returns.
- **Failure:** If the Host runs out of Arena memory, `arena_alloc` returns `NULL`. The Plugin MUST handle this case.

### 6.2 Persistence Strategy

If a Plugin emits an event using Arena memory that must survive beyond the current `weave` cycle (or be processed by an async worker), it MUST set the `STRANDS_FLAG_PERSIST` bit (or use an appending OpCode).

**Rules:**

1.  **Host Responsibility:** For `STRANDS_FMT_UTF8` and `STRANDS_FMT_BYTES` (flat buffers), the Host MUST perform a deep block copy of the data.
2.  **Plugin Responsibility:** Plugins **MUST NOT** set `STRANDS_FLAG_PERSIST` on formats containing internal pointers (like `StrandsHttpRequest`) unless they serialize the data into a flat buffer first.
3.  **Standard Library Structs:** These are designed for **Synchronous / Transient IPC**. They are not storage formats. If a Plugin wishes to persist an HTTP Request log, it SHOULD serialize the request to JSON or a custom binary format before appending it to the Timeline.

### 6.3 Timeline Access

Pointers returned by `host->read_timeline` are **Ephemeral**.

- They are valid **ONLY** for the duration of the current `weave` call.
- Plugins **MUST** copy data if they wish to retain it past the current weave cycle.
- **Worker Access:** Background workers cannot access the Timeline. If a worker requires historical context, the Plugin MUST copy that data into the worker payload buffer before spawning.

### 6.4 Extension Chains (`p_next`)

Structures linked via `p_next` MUST follow the same memory ownership and lifetime rules as their parent structure. If a struct is allocated in the Arena, its extension chain must also reside in the Arena (or static memory).

### 6.5 String Ownership

The `StrandsString` structure is a **Non-Owning View**.

- Receivers of a `StrandsString` **MUST NOT** attempt to `free()` the pointer.
- `ptr` is **NOT** guaranteed to be null-terminated. Usage with C string functions like `printf("%s")` is unsafe; use the provided macros or `printf("%.*s")`.

---

## 7. Host Interface

The Host Interface separates logical capability negotiation from data format negotiation.

### 7.1 Lifecycle and Configuration

The Host initializes the Plugin via the `create` function. This function accepts a **[StrandsConfig](#84-lifecycle-configuration)** structure, allowing the Host to inject environment variables, security capabilities, and runtime settings directly into the Plugin instance.

### 7.2 Logical Capabilities

Plugins query `host->has_capability` to determine if a specific URI is supported.

### 7.3 Data Formats

The `supported_formats_mask` allows the Host to declare which data layouts it can parse.

- **Discovery:** The Host provides a bitmask of `StrandsDataFormat` values.
- **Requirement:** All Hosts MUST support `STRANDS_FMT_BYTES` for system calls defined in the Standard Library.

### 7.4 Background Execution

The Host provides `spawn_worker` to allow Plugins to execute long-running tasks without blocking the main thread. This abstracts the underlying threading model, which may be POSIX threads, Windows threads, or Web Workers. The Host calls the Plugin's `run_worker` function in the new thread context.

**Warning:** Data passed to `spawn_worker` must be **Flat**. Do not pass pointers to ephemeral Arena memory. The Host performs a simple block copy of the `flat_data` buffer.
**Minimum Guarantee:** All Hosts MUST support a worker payload size of at least **64KB**.
**Safety Valve:** To prevent resource exhaustion, the Host MAY enforce a maximum size limit on `flat_data` (e.g., 1MB) and return an error if the Plugin exceeds it.

---

## 8. Data Structure Reference

This section defines the canonical binary layouts for all structures used in the Strands ABI. All padding bytes (`reserved`, `_pad`) **MUST** be initialized to zero.

### 8.1 Event Envelope (`StrandsEvent`)

The core unit of communication. Aligned to 128 bytes (two cache lines).

| Offset | Type          | Field            | Description                              |
| :----- | :------------ | :--------------- | :--------------------------------------- |
| 0      | `uint32_t`    | `s_type`         | Structure Type ID                        |
| 4      | `uint32_t`    | `flags`          | Bitmask for Persistence and Status       |
| 8      | `void*`       | `p_next`         | Pointer to extension structure           |
| 16     | `uint64_t`    | `id`             | Host ID or Target ID                     |
| 24     | `uint64_t`    | `correlation_id` | Origin Event ID                          |
| 32     | `const char*` | `kind.ptr`       | URI string pointer                       |
| 40     | `size_t`      | `kind.len`       | URI string length                        |
| 48     | `const void*` | `data`           | Payload pointer (8-byte aligned)         |
| 56     | `size_t`      | `data_len`       | Payload size                             |
| 64     | `uint64_t`    | `timestamp`      | Nanoseconds since Unix Epoch             |
| 72     | `uint8_t[16]` | `trace_id`       | OTEL Trace ID (Network Byte Order)       |
| 88     | `uint8_t[8]`  | `parent_span_id` | OTEL Parent Span ID (Network Byte Order) |
| 96     | `uint32_t`    | `op_code`        | Timeline Operation                       |
| 100    | `uint32_t`    | `format`         | Data Format                              |
| 104    | `uint8_t[24]` | `reserved`       | Padding to 128 bytes                     |

### 8.2 Primitives

**String (`StrandsString`)**
Non-owning string view. Not null-terminated.

| Offset | Type     | Field | Description             |
| :----- | :------- | :---- | :---------------------- |
| 0      | `char*`  | `ptr` | Pointer to string bytes |
| 8      | `size_t` | `len` | Length in bytes         |

**Pair (`StrandsPair`)**
Key-Value pair used for headers and configuration.

| Offset | Type            | Field   | Description  |
| :----- | :-------------- | :------ | :----------- |
| 0      | `StrandsString` | `key`   | Key string   |
| 16     | `StrandsString` | `value` | Value string |

### 8.3 Execution Context (`StrandsWeaveInfo`)

Passed to `weave` on every turn. 128-byte aligned.

| Offset | Type             | Field                 | Description                |
| :----- | :--------------- | :-------------------- | :------------------------- |
| 0      | `uint32_t`       | `s_type`              | Structure Type ID          |
| 4      | `uint32_t`       | `_pad0`               | Alignment padding          |
| 8      | `void*`          | `p_next`              | Extension pointer          |
| 16     | `StrandsContext` | `ctx`                 | Host Context Handle        |
| 24     | `uint64_t`       | `time_budget_hint_ns` | Time budget remaining (ns) |
| 32     | `uint64_t`       | `context_tokens_used` | Current token usage        |
| 40     | `uint64_t`       | `context_tokens_max`  | Maximum token window       |
| 48     | `uint32_t`       | `signal_worker_id`    | Worker Correlation ID      |
| 52     | `uint32_t`       | `signal_reason`       | Worker Wake Reason         |
| 56     | `const void*`    | `signal_data`         | Result data from worker    |
| 64     | `size_t`         | `signal_data_len`     | Size of result data        |
| 72     | `PFN_Alloc`      | `arena_alloc`         | Arena Allocator Function   |
| 80     | `void*`          | `arena_user_data`     | Allocator Context          |
| 88     | `size_t`         | `timeline_len`        | Total Timeline Events      |
| 96     | `uint64_t`       | `last_event_id`       | ID of most recent event    |
| 104    | `uint8_t[24]`    | `reserved`            | Padding to 128 bytes       |

### 8.4 Lifecycle Configuration (`StrandsConfig`)

Passed once during `create`.

| Offset | Type           | Field               | Description              |
| :----- | :------------- | :------------------ | :----------------------- |
| 0      | `uint32_t`     | `s_type`            | Structure Type ID        |
| 4      | `uint32_t`     | `debug_mode`        | 1 = Enable extra checks  |
| 8      | `void*`        | `p_next`            | Extension pointer        |
| 16     | `uint64_t`     | `capabilities_mask` | Allowed capabilities     |
| 24     | `size_t`       | `entry_count`       | Number of config pairs   |
| 32     | `StrandsPair*` | `entries`           | Array of Key/Value pairs |

### 8.5 Standard Library: Context Management (`StrandsContextTruncate`)

| Offset | Type        | Field        | Description               |
| :----- | :---------- | :----------- | :------------------------ |
| 0      | `uint32_t`  | `s_type`     | Structure Type ID         |
| 4      | `uint32_t`  | `reserved`   | Padding (Zero)            |
| 8      | `void*`     | `p_next`     | Extension pointer         |
| 16     | `uint32_t`  | `count`      | Items to remove from head |
| 20     | `uint32_t`  | `keep_count` | Size of `keep_ids` array  |
| 24     | `uint64_t*` | `keep_ids`   | IDs to preserve           |

### 8.6 Standard Library: Tooling

**Definition (`StrandsToolDefinition`)**

| Offset | Type            | Field          | Description       |
| :----- | :-------------- | :------------- | :---------------- |
| 0      | `uint32_t`      | `s_type`       | Structure Type ID |
| 4      | `uint32_t`      | `reserved`     | Padding (Zero)    |
| 8      | `void*`         | `p_next`       | Extension pointer |
| 16     | `StrandsString` | `name`         | Tool Name         |
| 32     | `StrandsString` | `description`  | Description       |
| 48     | `StrandsString` | `input_schema` | Schema string     |
| 64     | `uint32_t`      | `input_format` | Expected format   |
| 68     | `uint32_t`      | `_pad0`        | Alignment padding |

**Result (`StrandsToolResult`)**

| Offset | Type            | Field           | Description           |
| :----- | :-------------- | :-------------- | :-------------------- |
| 0      | `uint32_t`      | `s_type`        | Structure Type ID     |
| 4      | `uint32_t`      | `status`        | 0=OK, Non-zero=Error  |
| 8      | `void*`         | `p_next`        | Extension pointer     |
| 16     | `StrandsString` | `tool_name`     | Name of tool executed |
| 32     | `void*`         | `output`        | Output Data           |
| 40     | `size_t`        | `output_len`    | Output size           |
| 48     | `uint32_t`      | `output_format` | Format of output      |
| 52     | `uint32_t`      | `_pad0`         | Alignment padding     |

### 8.7 Standard Library: HTTP

**Request (`StrandsHttpRequest`)**

| Offset | Type            | Field          | Description       |
| :----- | :-------------- | :------------- | :---------------- |
| 0      | `uint32_t`      | `s_type`       | Structure Type ID |
| 4      | `uint32_t`      | `reserved`     | Padding (Zero)    |
| 8      | `void*`         | `p_next`       | Extension pointer |
| 16     | `StrandsString` | `method`       | Method String     |
| 32     | `StrandsString` | `url`          | URL String        |
| 48     | `uint32_t`      | `header_count` | Number of headers |
| 52     | `uint32_t`      | `pad0`         | Alignment padding |
| 56     | `StrandsPair*`  | `headers`      | Header Array      |
| 64     | `void*`         | `body`         | Body Payload      |
| 72     | `size_t`        | `body_len`     | Body Size         |

**Response (`StrandsHttpResponse`)**

| Offset | Type            | Field          | Description       |
| :----- | :-------------- | :------------- | :---------------- |
| 0      | `uint32_t`      | `s_type`       | Structure Type ID |
| 4      | `uint32_t`      | `reserved`     | Padding (Zero)    |
| 8      | `void*`         | `p_next`       | Extension pointer |
| 16     | `uint32_t`      | `status`       | HTTP Status Code  |
| 20     | `uint32_t`      | `header_count` | Number of headers |
| 24     | `StrandsPair*`  | `headers`      | Header Array      |
| 32     | `void*`         | `body`         | Body Payload      |
| 40     | `size_t`        | `body_len`     | Body Size         |
| 48     | `StrandsString` | `error_msg`    | Transport error   |

---

## 9. Standard Library Capabilities

### 9.1 Context Management

**Capability:** `strands.sys`
**URI:** `strands.sys.context.truncate`

Requests the Host to remove older events to free token context.

- **OpCode:** `STRANDS_OP_COMMAND` (Transient)
- **Payload:** `StrandsContextTruncate`
- **Behavior:** The Host deletes the `count` oldest events from the Timeline. Any Event IDs specified in `keep_ids` are exempted from deletion.

### 9.2 Networking

**Capability:** `strands.std.net.http`

#### Request

**URI:** `strands.std.net.http.request`
**Payload:** `StrandsHttpRequest`

- **OpCode:** `STRANDS_OP_COMMAND` (Transient)
- **Behavior:** Asynchronous. The Plugin emits this event and returns. The Host performs the network request.
- **Async Execution:** Because the Host performs the request asynchronously, it MUST copy the `StrandsHttpRequest` and its contents (headers/strings) before the Arena is reset. Hosts MUST support deep copying this specific Standard Library structure for execution purposes.

#### Response

**URI:** `strands.std.net.http.response`
**Payload:** `StrandsHttpResponse`

- **Behavior:** The Host appends this event to the Timeline with the `correlation_id` matching the Request.

### 9.3 Tooling & Orchestration

**Capability:** `strands.std.tool`

#### Definition

**URI:** `strands.std.tool.def`
**Payload:** `StrandsToolDefinition`

- **OpCode:** `STRANDS_OP_APPEND` (Historical)
- **Purpose:** Advertises a tool (Function) availability to the Agent.
- **Lifecycle:** To remove a tool, the Host or Plugin emits a `STRANDS_OP_DELETE` event targeting the ID of the original Tool Definition.

#### Invocation

**URI:** `strands.std.tool.invoke`
**Payload:** JSON or CBOR (Application Specific)

- **OpCode:** `STRANDS_OP_COMMAND` (Transient)
- **Purpose:** The Agent emits this command to call a tool. The Host routes it to the appropriate handler (Native function or Micro-Plugin).

#### Result

**URI:** `strands.std.tool.result`
**Payload:** `StrandsToolResult`

- **OpCode:** `STRANDS_OP_APPEND` (Historical)
- **Purpose:** The Host (or Micro-Plugin) appends the tool's output to the Timeline, setting the `correlation_id` to the ID of the Invocation command.

### 9.4 Process Execution

**Capability:** `strands.std.process`

#### Execution

**URI:** `strands.std.process.exec`

- `bin`: Path to executable.
- `args`: Arguments.
- `env`: Environment variables.

#### Result

**URI:** `strands.std.process.result`

- `code`: Exit code.
- `stdout`: Standard Output capture.
- `stderr`: Standard Error capture.

---

## 10. Safety Contract & Undefined Behavior

To ensure stability across language and privilege boundaries, this specification defines a strict Safety Contract.

### 10.1 Host Responsibilities

1.  **Validation:** The Host MUST validate that `s_type` fields match known enumerations and that `data_len` does not exceed accessible memory bounds.
2.  **Pointer Translation:** For Wasm/Sandboxed plugins, the Host MUST translate linear memory offsets to virtual addresses securely.
3.  **Isolation:** The Host MUST zero-initialize all padding bytes in structures passed to the Plugin to prevent information leaks.

### 10.2 Plugin Responsibilities

1.  **Flat Data:** When calling `spawn_worker`, the Plugin guarantees that the `flat_data` payload contains no internal pointers to ephemeral memory (Arena).
2.  **Termination:** The Plugin guarantees that `p_next` chains are correctly terminated with `NULL`.
3.  **Strings:** The Plugin guarantees that it will not read past `len` bytes of a `StrandsString`.

### 10.3 Violation Consequences

If the Host detects a violation of this contract (e.g., OOB access, invalid enum, non-flat data in workers), it SHOULD immediately transition the Plugin to `STRANDS_RESULT_ERROR` and revoke execution rights. Passing pointers in `flat_data` results in **Undefined Behavior**.

---

## Appendix A: Core Header

**Filename:** `strands.h`

```c
#ifndef STRANDS_H
#define STRANDS_H

#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>

// --------------------------------------------------------------------------
// Linkage Helper Macros
// --------------------------------------------------------------------------
#if defined(_WIN32)
    #define STRANDS_EXPORT_API __declspec(dllexport)
    #define STRANDS_IMPORT_API __declspec(dllimport)
#else
    #define STRANDS_EXPORT_API __attribute__((visibility("default")))
    #define STRANDS_IMPORT_API
#endif

// Polyglot Static Assertion
#if defined(__cplusplus)
    #if __cplusplus >= 201103L
        #define STRANDS_STATIC_ASSERT(cond, msg) static_assert(cond, msg)
    #else
        #define STRANDS_STATIC_ASSERT(cond, msg)
    #endif
#elif defined(__STDC_VERSION__) && __STDC_VERSION__ >= 201112L
    #define STRANDS_STATIC_ASSERT(cond, msg) _Static_assert(cond, msg)
#else
    #define STRANDS_STATIC_ASSERT(cond, msg)
#endif

// Architecture-Agnostic Pointer Padding
// Ensures 64-bit alignment of subsequent fields even on 32-bit systems (Wasm).
// Assumes Little Endian architecture (Wasm, x86, ARM).
// IMPORTANT: Writers MUST zero-initialize these padding fields.
#if UINTPTR_MAX == 0xffffffff
    #define STRANDS_PAD_PTR(name) uint32_t name##_pad
    #define STRANDS_PAD_SIZE(name) uint32_t name##_pad
#else
    #define STRANDS_PAD_PTR(name)
    #define STRANDS_PAD_SIZE(name)
#endif

// String Formatting Helper
#define STRANDS_FMT "%.*s"
#define STRANDS_ARG(s) (int)(s).len, (s).ptr

#if defined(__cplusplus)
    extern "C" {
#endif

// Version 0.1.1
#define STRANDS_MAKE_VERSION(major, minor, patch) \
    (((major) << 22) | ((minor) << 12) | (patch))

#define STRANDS_VERSION_0_1_1 STRANDS_MAKE_VERSION(0, 1, 1)

#define STRANDS_VERSION_COMPATIBLE(host_ver, plugin_ver) \
    ((((host_ver) >> 22) == ((plugin_ver) >> 22)) && \
     (((host_ver) >> 12) & 0x3FF) >= (((plugin_ver) >> 12) & 0x3FF))

// Use this for time_budget_hint_ns to indicate no limit
#define STRANDS_TIME_BUDGET_UNLIMITED UINT64_MAX

// --------------------------------------------------------------------------
// Enums
// --------------------------------------------------------------------------
typedef enum StrandsResult {
    STRANDS_RESULT_SUCCESS = 0,
    STRANDS_RESULT_PENDING = 1,
    STRANDS_RESULT_ERROR   = -1 // Fatal Crash. Write to error_buf.
} StrandsResult;

typedef enum StrandsDataFormat {
    STRANDS_FMT_UNKNOWN = 0,
    STRANDS_FMT_CBOR    = 1,
    STRANDS_FMT_BYTES   = 2, // Raw Flat Bytes
    STRANDS_FMT_UTF8    = 3,
    STRANDS_FMT_JSON    = 4,
    STRANDS_FMT_MAX     = 0x7FFFFFFF
} StrandsDataFormat;

typedef enum StrandsOpCode {
    STRANDS_OP_APPEND   = 0, // Persist to Timeline
    STRANDS_OP_REPLACE  = 1, // Persist and Overwrite
    STRANDS_OP_INSERT   = 2, // Persist and Insert
    STRANDS_OP_DELETE   = 3, // Remove from Timeline
    STRANDS_OP_COMMAND  = 4, // Transient/IPC (Do not persist)
    STRANDS_OP_MAX      = 0x7FFFFFFF
} StrandsOpCode;

typedef enum StrandsEventFlags {
    STRANDS_FLAG_NONE    = 0,
    STRANDS_FLAG_PERSIST = 1 << 0,
    STRANDS_FLAG_ERROR   = 1 << 1, // Logic Error (e.g. HTTP 404)
    STRANDS_FLAG_MAX     = 0x7FFFFFFF
} StrandsEventFlags;

typedef enum StrandsStructureType {
    STRANDS_STRUCTURE_TYPE_EVENT            = 0,
    STRANDS_STRUCTURE_TYPE_CONTEXT_TRUNCATE = 1,
    STRANDS_STRUCTURE_TYPE_WEAVE_INFO       = 2,
    STRANDS_STRUCTURE_TYPE_CONFIG           = 3,
    STRANDS_STRUCTURE_TYPE_HTTP_REQUEST     = 100,
    STRANDS_STRUCTURE_TYPE_HTTP_RESPONSE    = 101,
    STRANDS_STRUCTURE_TYPE_TOOL_DEF         = 102,
    STRANDS_STRUCTURE_TYPE_TOOL_RESULT      = 103,
    STRANDS_STRUCTURE_TYPE_MAX              = 0x7FFFFFFF
} StrandsStructureType;

// --------------------------------------------------------------------------
// Common Types
// --------------------------------------------------------------------------
#if defined(_MSC_VER)
    #define STRANDS_ALIGN(x) __declspec(align(x))
#else
    #define STRANDS_ALIGN(x) __attribute__((aligned(x)))
#endif

// Non-Owning View. Do not free. Not null-terminated.
typedef struct StrandsString {
    const char* ptr;
    STRANDS_PAD_PTR(ptr);
    size_t      len;
    STRANDS_PAD_SIZE(len);
} StrandsString;

typedef struct STRANDS_ALIGN(8) StrandsPair {
    StrandsString key;
    StrandsString value;
} StrandsPair;

typedef struct StrandsConfig {
    StrandsStructureType s_type;
    uint32_t             debug_mode;
    void*                p_next;
    STRANDS_PAD_PTR(p_next);

    uint64_t           capabilities_mask;
    size_t             entry_count;
    STRANDS_PAD_SIZE(entry_count);
    const StrandsPair* entries;
    STRANDS_PAD_PTR(entries);
} StrandsConfig;

// --------------------------------------------------------------------------
// Event Struct (128 Bytes)
// --------------------------------------------------------------------------

typedef struct STRANDS_ALIGN(16) StrandsEvent {
    StrandsStructureType s_type;
    uint32_t             flags;
    void*                p_next;
    STRANDS_PAD_PTR(p_next);

    uint64_t             id;
    uint64_t             correlation_id;

    StrandsString        kind;
    const void*          data;
    STRANDS_PAD_PTR(data);
    size_t               data_len;
    STRANDS_PAD_SIZE(data_len);

    uint64_t             timestamp; // Unix Nanoseconds

    // OpenTelemetry W3C Trace Context (Opaque Bytes, Big Endian)
    uint8_t              trace_id[16];
    uint8_t              parent_span_id[8];

    uint32_t             op_code;
    uint32_t             format;

    // Explicit padding to reach 128 bytes. MUST be zeroed.
    uint8_t              reserved[24];
} StrandsEvent;

STRANDS_STATIC_ASSERT(sizeof(StrandsEvent) == 128, "StrandsEvent must be 128 bytes");

// --------------------------------------------------------------------------
// Interfaces
// --------------------------------------------------------------------------
typedef struct StrandsContext_T* StrandsContext;

// Host Arena Allocator
// align: Memory alignment requirement (e.g. 16, 64)
typedef void* (*PFN_StrandsAlloc)(void* user_data, size_t size, size_t align);

typedef struct STRANDS_ALIGN(16) StrandsWeaveInfo {
    StrandsStructureType s_type;
    uint32_t             _pad0;
    void*                p_next;
    STRANDS_PAD_PTR(p_next);

    StrandsContext   ctx;
    STRANDS_PAD_PTR(ctx);

    // Wall-clock time budget remaining for this turn (latency limit)
    uint64_t         time_budget_hint_ns;

    // Token Accounting
    uint64_t         context_tokens_used;
    uint64_t         context_tokens_max;

    // Result from async worker (valid only for current turn)
    uint32_t         signal_worker_id; // Matches ID passed to spawn_worker
    uint32_t         signal_reason;    // Matches reason passed to wake

    const void*      signal_data;
    STRANDS_PAD_PTR(signal_data);
    size_t           signal_data_len;
    STRANDS_PAD_SIZE(signal_data_len);

    PFN_StrandsAlloc arena_alloc;
    STRANDS_PAD_PTR(arena_alloc);
    void*            arena_user_data;
    STRANDS_PAD_PTR(arena_user_data);

    size_t           timeline_len;
    STRANDS_PAD_SIZE(timeline_len);
    uint64_t         last_event_id;

    // Explicit padding to reach 128 bytes. MUST be zeroed.
    uint8_t          reserved[24];
} StrandsWeaveInfo;

STRANDS_STATIC_ASSERT(sizeof(StrandsWeaveInfo) == 128, "StrandsWeaveInfo must be 128 bytes");

typedef struct StrandsHostInterface {
    uint32_t api_version;
    uint32_t _pad0;
    void*    p_next; // Interface extension chain
    STRANDS_PAD_PTR(p_next);

    // Negotiation
    bool     (*has_capability)(StrandsContext ctx, StrandsString uri);
    STRANDS_PAD_PTR(has_capability);
    uint32_t supported_formats_mask;
    uint32_t _pad1;

    // Async Signaling
    // Host block-copies `len` bytes of `flat_data` to a safe location.
    // WARNING: `flat_data` MUST NOT contain pointers to Arena memory.
    // Host MAY reject if flat_len exceeds implementation limit (e.g. 1MB).
    int    (*spawn_worker)(StrandsContext ctx, uint32_t worker_id,
                           const void* flat_data, size_t flat_len);
    STRANDS_PAD_PTR(spawn_worker);

    // `result_data` is copied by Host and passed to next weave via signal_data.
    void   (*wake)(StrandsContext ctx, uint32_t reason,
                   const void* result_data, size_t result_len);
    STRANDS_PAD_PTR(wake);

    void   (*log)(StrandsContext ctx, uint32_t level, StrandsString msg);
    STRANDS_PAD_PTR(log);

    // Random Access
    size_t (*get_timeline_len)(StrandsContext ctx);
    STRANDS_PAD_PTR(get_timeline_len);

    // Returns 0 on success, < 0 on error.
    // Pointers in out_event (like kind.ptr) point into aux_buf.
    int    (*read_timeline)(StrandsContext ctx, size_t idx,
                            StrandsEvent* out_event,
                            void* aux_buf, size_t aux_buf_size);
    STRANDS_PAD_PTR(read_timeline);

    int    (*find_event)(StrandsContext ctx, uint64_t id,
                         StrandsEvent* out_event,
                         void* aux_buf, size_t aux_buf_size);
    STRANDS_PAD_PTR(find_event);

    // O(1) Tool Discovery (Uses aux_buf for tool name strings)
    // Returns active tool count. If out_tools != NULL, fills array.
    size_t (*get_active_tools)(StrandsContext ctx,
                               void* out_tools, // StrandsToolDefinition*
                               size_t out_tools_size,
                               void* aux_buf,
                               size_t aux_buf_size);
    STRANDS_PAD_PTR(get_active_tools);

} StrandsHostInterface;

typedef struct StrandsPluginInterface {
    uint32_t api_version;
    uint32_t _pad0;
    void*    p_next; // Interface extension chain
    STRANDS_PAD_PTR(p_next);

    // Configuration passed during creation
    int           (*create)(const StrandsHostInterface* host,
                            const StrandsConfig* config,
                            void** instance);
    STRANDS_PAD_PTR(create);

    void          (*destroy)(void* instance);
    STRANDS_PAD_PTR(destroy);
    int           (*prepare)(void* instance);
    STRANDS_PAD_PTR(prepare);

    // Returns success/pending/error code.
    // On STRANDS_RESULT_ERROR, plugin writes explanation to error_buf.
    // Plugin sets *out_events to point to the array allocated in the Arena.
    StrandsResult (*weave)(void* instance, const StrandsWeaveInfo* info,
                           StrandsEvent** out_events, size_t* out_count,
                           char* error_buf, size_t error_buf_size);
    STRANDS_PAD_PTR(weave);

    // Worker Dispatch
    // `user_data` points to the safe copy created by `spawn_worker`
    void          (*run_worker)(void* instance, StrandsContext ctx,
                                uint32_t worker_id, void* user_data);
    STRANDS_PAD_PTR(run_worker);

} StrandsPluginInterface;

// --------------------------------------------------------------------------
// Bootstrap (Entry Point)
// --------------------------------------------------------------------------
// The Plugin must export a function matching this signature.
typedef const StrandsPluginInterface* (*PFN_StrandsGetPlugin)(void);

#define STRANDS_ENTRY_POINT_NAME strands_get_plugin

// Helper Macro for declaring the entry point
#define STRANDS_PLUGIN_ENTRY(FuncImpl) \
    STRANDS_EXPORT_API const StrandsPluginInterface* STRANDS_ENTRY_POINT_NAME(void) { \
        return FuncImpl(); \
    }

#if defined(__cplusplus)
}
#endif
#endif // STRANDS_H
```

## Appendix B: Standard Library Header

**Filename:** `strands_std.h`

```c
#ifndef STRANDS_STD_H
#define STRANDS_STD_H

#if defined(__cplusplus)
    extern "C" {
#endif

// Capability Declarations
#define STRANDS_CAP_STD         "strands.std"
#define STRANDS_CAP_NET         "strands.std.net"
#define STRANDS_CAP_NET_HTTP    "strands.std.net.http"
#define STRANDS_CAP_ORCH        "strands.std.orch"
#define STRANDS_CAP_PROCESS     "strands.std.process"
#define STRANDS_CAP_TOOL        "strands.std.tool"

// Event URI Definitions
#define STRANDS_URI_CTX_TRUNC   "strands.sys.context.truncate"

#define STRANDS_URI_HTTP_REQ    "strands.std.net.http.request"
#define STRANDS_URI_HTTP_RES    "strands.std.net.http.response"

#define STRANDS_URI_TOOL_DEF    "strands.std.tool.def"
#define STRANDS_URI_TOOL_INVOKE "strands.std.tool.invoke"
#define STRANDS_URI_TOOL_RESULT "strands.std.tool.result"

#define STRANDS_URI_ORCH_SPAWN  "strands.std.orch.spawn"

#define STRANDS_URI_PROC_EXEC   "strands.std.process.exec"
#define STRANDS_URI_PROC_RES    "strands.std.process.result"

#if defined(__cplusplus)
}
#endif

#endif // STRANDS_STD_H
```

## Appendix C: Canonical Binary Layouts

**Filename:** `strands_std_structs.h`

**Usage:** These structures MUST be used when `format` is `STRANDS_FMT_BYTES`.
**Note:** These structures contain internal pointers and are for **Transient IPC only**. They MUST NOT be persisted to the Timeline.

```c
#ifndef STRANDS_STD_STRUCTS_H
#define STRANDS_STD_STRUCTS_H

#include <stdint.h>
#include "strands.h"

#if defined(__cplusplus)
extern "C" {
#endif

// --------------------------------------------------------------------------
// Context Truncation
// --------------------------------------------------------------------------
typedef struct STRANDS_ALIGN(8) StrandsContextTruncate {
    StrandsStructureType s_type;
    uint32_t             reserved;
    void*                p_next;
    STRANDS_PAD_PTR(p_next);

    uint32_t count;
    uint32_t keep_count;
    const uint64_t* keep_ids;
    STRANDS_PAD_PTR(keep_ids);
} StrandsContextTruncate;

// --------------------------------------------------------------------------
// Tool Definition
// --------------------------------------------------------------------------
typedef struct STRANDS_ALIGN(8) StrandsToolDefinition {
    StrandsStructureType s_type;
    uint32_t             reserved;
    void*                p_next;
    STRANDS_PAD_PTR(p_next);

    StrandsString        name;
    StrandsString        description;
    StrandsString        input_schema;  // JSON Schema or similar
    uint32_t             input_format;  // e.g. STRANDS_FMT_JSON
    uint32_t             _pad0;
} StrandsToolDefinition;

// --------------------------------------------------------------------------
// Tool Result
// --------------------------------------------------------------------------
typedef struct STRANDS_ALIGN(8) StrandsToolResult {
    StrandsStructureType s_type;
    uint32_t             status; // 0=OK, >0=Error
    void*                p_next;
    STRANDS_PAD_PTR(p_next);

    StrandsString        tool_name; // Optional context

    const void*          output;
    STRANDS_PAD_PTR(output);
    size_t               output_len;
    STRANDS_PAD_SIZE(output_len);

    uint32_t             output_format; // e.g. STRANDS_FMT_JSON
    uint32_t             _pad0;
} StrandsToolResult;

// --------------------------------------------------------------------------
// HTTP Request
// --------------------------------------------------------------------------
typedef struct STRANDS_ALIGN(8) StrandsHttpRequest {
    StrandsStructureType s_type;
    uint32_t             reserved;
    void*                p_next;
    STRANDS_PAD_PTR(p_next);

    StrandsString        method;
    StrandsString        url;

    uint32_t             header_count;
    uint32_t             pad0;
    const StrandsPair*   headers;
    STRANDS_PAD_PTR(headers);

    const void*          body;
    STRANDS_PAD_PTR(body);
    size_t               body_len;
    STRANDS_PAD_SIZE(body_len);
} StrandsHttpRequest;

// --------------------------------------------------------------------------
// HTTP Response
// --------------------------------------------------------------------------
typedef struct STRANDS_ALIGN(8) StrandsHttpResponse {
    StrandsStructureType s_type;
    uint32_t             reserved;
    void*                p_next;
    STRANDS_PAD_PTR(p_next);

    uint32_t             status;
    uint32_t             header_count;
    const StrandsPair*   headers;
    STRANDS_PAD_PTR(headers);

    const void*          body;
    STRANDS_PAD_PTR(body);
    size_t               body_len;
    STRANDS_PAD_SIZE(body_len);

    StrandsString        error_msg;
} StrandsHttpResponse;

// Verify Binary Layout Consistency
STRANDS_STATIC_ASSERT(sizeof(StrandsContextTruncate) % 8 == 0, "StrandsContextTruncate alignment error");
STRANDS_STATIC_ASSERT(sizeof(StrandsToolDefinition) % 8 == 0, "StrandsToolDefinition alignment error");
STRANDS_STATIC_ASSERT(sizeof(StrandsToolResult) % 8 == 0, "StrandsToolResult alignment error");
STRANDS_STATIC_ASSERT(sizeof(StrandsHttpRequest) % 8 == 0, "StrandsHttpRequest alignment error");
STRANDS_STATIC_ASSERT(sizeof(StrandsHttpResponse) % 8 == 0, "StrandsHttpResponse alignment error");

#if defined(__cplusplus)
}
#endif

#endif // STRANDS_STD_STRUCTS_H
```
