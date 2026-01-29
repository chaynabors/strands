# Filament Specification 0.2.0

**Status:** Draft

**Date:** 2026-01-26

**Distribution:** Public

---

**Table of Contents**

**Part I: Concepts**

1.  [**Preamble**](#1-preamble)
2.  [**Architecture Overview [Informative]**](#2-architecture-overview-informative)
3.  [**Execution Contexts**](#3-execution-contexts)
4.  [**The Kernel Model**](#4-the-kernel-model)
5.  [**Data Lifecycle**](#5-data-lifecycle)

**Part II: The Kernel Interface**

1.  [**System Conventions**](#1-system-conventions)
2.  [**Constants and Enumerations**](#2-constants-and-enumerations)
3.  [**Data Structures**](#3-data-structures)
4.  [**Kernel Interface Functions**](#4-kernel-interface-functions)
5.  [**Module Interface Functions**](#5-module-interface-functions)

**Part III: The Capability Model**

1.  [**Capability Concepts**](#1-capability-concepts)
2.  [**Capability Catalog**](#2-capability-catalog)

**Appendices**

- [**Appendix A: Glossary**](#appendix-a-glossary)
- [**Appendix B: Conformance**](#appendix-b-conformance)

---

# Part I: Concepts

## 1. Preamble

### 1.1. Scope

This document defines the **Filament Specification**, a standard for a deterministic, event-sourced partitioning kernel. It defines the two normative layers required for compliance with the standard:

1.  **The Kernel Interface:** The low-level execution contract and binary interface.
2.  **The Capability Model:** The security and capability interfaces of a kernel.

### 1.2. Status

This is a draft specification. The Application Binary Interface and Schema defined herein are **not** guaranteed to be forward or backward compatible.

### 1.3. Normative Status

Unless explicitly marked as **Informative**, all sections in this document are **Normative**.

### 1.4. Keywords

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHALL**, **SHALL NOT**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY**, and **OPTIONAL** in this document are to be interpreted as described in RFC 2119.

---

## 2. Architecture Overview [Informative]

### 2.1 System Model

Filament implements a partitioned, event-sourced architecture. The system is composed of three hierarchical abstractions:

1.  **The Process:** The static definition of the workload, including topology, configuration, and permissions.
2.  **The Kernel:** The runtime executive that manages scheduling, memory, and I/O.
3.  **The Module:** The individual unit of logic that executes within a sandboxed or trusted context.

Execution is transactional. The Kernel advances the system state through atomic cycles called **Weaves**. During a Weave, the Kernel injects a deterministic **Virtual Time** and provides a transient **Staging Area** for inter-module communication.

### 2.2 The Kernel

The Kernel acts as a user-space scheduler and memory manager. It is responsible for loading Modules, enforcing the Application Binary Interface (ABI), and driving the Weave cycle.

Unlike a general-purpose OS kernel which manages contention for hardware, the Filament Kernel manages the logical flow of causality. It enforces isolation by strictly mediating all data exchange through the Staging Area. If a Module violates its resource budget (time or memory) or attempts an illegal operation, the Kernel discards the pending state of the current cycle, ensuring the underlying Timeline remains consistent.

### 2.3 The Process

A Process is the container for the execution timeline. It is defined declaratively by a Manifest, which creates an immutable binding between a set of Modules and their configuration.

The Process state is defined exclusively by its **Timeline**, an append-only log of committed events. By decoupling the Process definition from the host hardware, Filament ensures that a Process creates identical Timeline entries given the same input history, regardless of the underlying host topology.

### 2.4 The Module

A Module is a binary artifact that implements the Filament ABI. It functions as a pure transformation logic, accepting input events from the Staging Area and producing uncommitted output events.

Modules operate within specific **Execution Contexts** that determine their capabilities:

- **Logic Contexts** rely on WebAssembly to enforce strict memory isolation and deterministic arithmetic.
- **Unmanaged Contexts** utilize native linkage to provide low-latency access to host hardware.

### 2.5 Deployment Topologies

The architecture supports two distinct host mappings:

- **Hosted Topology:** The Kernel executes as a standard process within a Host OS. The Kernel schedules Modules internally, effectively functioning as a green-thread scheduler. This is used for simulation and high-density cloud workloads.
- **Native Topology:** The Kernel executes directly on the hardware metal. The Kernel manages physical interrupts and CPU cycles directly. This is used for real-time safety controllers where jitter from a Host OS is unacceptable.

### 2.6 Documentation Conventions

Throughout the specification **Compliance Tables** are used to define normative constraints.

- **Common Requirements:** Constraints that both the **Kernel** and the **Module** must honor. Violation of common requirements is the same as violating either module or kernel requirements respectively.
- **Kernel Requirements:** Constraints the **Kernel** must honor. Violating these renders the Kernel Non-Compliant.
- **Module Requirements:** Constraints the **Module** must honor. Violation of module requirements results in the following behaviour:
  - If the violation is detected by the kernel at load time, the kernel will refuse to load the module.
  - If the violation is detected by the kernel at runtime, the kernel may terminate the weave with an error code.
  - Managed module violations that are not captured by the kernel may result in undefined behaviour in the context of a module.
  - Unmanaged module violations cannot be captured by the kernel and result in undefined behaviour in the context of the kernel.

In future versions:

- Existing IDs **SHALL NOT** be renumbered.
- Deleted requirements **SHALL** leave a gap in the sequence.
- New requirements **SHALL** be appended to the end of the list.

---

## 3. Execution Contexts

Because Filament supports diverse execution environments, the specification defines **Execution Contexts** to manage this diversity safely. These contexts determine the loader type, the available capabilities, and the preemption guarantees enforced by the Kernel.

### 3.1 Unmanaged Context

This context is designed for hardware abstraction, high-frequency sensor ingestion, and actuator control.

- **Artifacts:** Native Shared Objects are preferred. WebAssembly is supported only if the Kernel provides specific **HAL Intrinsics** (Hardware Abstraction Layer).
- **Privilege:** High. Native modules in this context may access memory-mapped I/O and interrupts.
- **Constraint:** Real-Time. Execution is subject to strict watchdog timers. Native code in this context is implicitly trusted and must yield cooperatively.
  - _Note on Hosted Topology:_ In a Hosted deployment, "Hard Real-Time" guarantees are subject to the capabilities of the underlying Host OS (e.g., requiring `PREEMPT_RT` patches).

### 3.2 Managed Context

This context is designed for **Orchestration**: network I/O, database connectivity, and external API integration.

- **Artifacts:** WebAssembly using Managed Runtimes or Process-Isolated Native binaries (on hosted topologies).
- **Privilege:** Networked. Modules may access TCP/HTTP interfaces and persistent key-value storage.
- **Memory Model:**
  - **Stateful (Default):** The instance persists between Weaves.
  - **Stateless (Pooled):** The instance is wiped and returned to a pool after execution.
- **Constraints:**
  - The Kernel **MUST** reject a Module defined as `stateless` if it requests any capability marked as **[Pinned]** in the Capability Model (e.g., Hardware Drivers).

### 3.3 Logic Context

This context is designed for **Pure Computation**: business logic, control algorithms, and safety verifiers.

- **Artifacts:** Strict WebAssembly.
- **Privilege:** Restricted. Modules may only access the Staging Area via the `filament.bus` capability. They have **NO** network access and **NO** hardware access.
- **Constraints:**
  - **State Hygiene:** The Kernel **MUST** reset the Module's linear memory and global state to its initial definition at the start of every Weave. Memory MUST NOT persist between cycles.
  - **Stochastic Isolation:** The execution is closed over its inputs. Pseudo-random operations must derive strictly from the cryptographic seed provided by the Kernel.
  - **Monotonicity:** The operation preserves history. If the result is a successful timeline commit, the input history is strictly a prefix of the new timeline.
  - **Preemption:** The Kernel may preempt execution at any instruction boundary using instruction metering.

### 3.4 Arithmetic Determinism

| ID           | Requirement                                                                                              |
| :----------- | :------------------------------------------------------------------------------------------------------- |
| **KR-3.4.1** | The Kernel **MUST** configure the execution engine for WASM modules to enforce **NaN Canonicalization**. |

## 4. The Kernel Model

### 4.1 The Pipeline Transaction

Execution in Filament is transactional and cyclical. This cycle is called a **Weave**.

1.  **Ingress:** The Kernel places pending Input Events onto the Staging Area.
2.  **Execution:** The Kernel invokes the Modules in sequential order defined by the Pipeline.
3.  **Commit:** If **ALL** Modules in the pipeline complete successfully, the Kernel commits the resulting Events to the Timeline and **flushes pending Hardware/Side-Effect Buffers** to physical interfaces.
4.  **Discard:** If **ANY** Module traps, panics, or exceeds its resource budget, the Kernel discards the Staging Area and **clears all pending Hardware/Side-Effect Buffers**. No physical actuation occurs.

### 4.2 Timekeeping Models

To support both real-time interaction and deterministic simulation, the Kernel **MUST** maintain two distinct clocks:

1.  **Wall Clock (`timestamp`):** Unix Epoch (ns). Represents physical time. Not guaranteed to be unique or monotonic.
2.  **Causal Clock (`tick`):** Monotonically increasing 64-bit integer. Represents the logical step index. **MUST** be unique per event.

### 4.3 Resource Accounting

To ensure deterministic execution across different Host implementations, Filament uses an abstract resource model. `resource_used` and `resource_max` (in `FilamentWeaveInfo`) are **opaque** monotonic counters representing **Compute Units** (often mapped to Instruction Counts in Wasm). Plugins **MUST** treat the ratio of `resource_used` to `resource_max` as a unitless throttling metric. If `resource_used` exceeds `resource_max`, the Kernel **MAY** terminate the Weave cycle immediately and return `FILAMENT_ERR_TIMEOUT`.

### 4.4 Concurrency and Isolation

The Kernel **MUST** guarantee that only one instance of a Module executes against a given timeline at any time. `weave()` invocations are serialized.

- **Instance Pooling:** Hosts **MAY** execute multiple Agents concurrently using instance pooling. Pooled modules **MUST** declare themselves to be stateless.
- **State Clearing:**
  - The Host **MUST** zero-initialize the Module's linear memory and global state for non-native modules any time a module is invoked with a new Agent ID.
  - The Host **MAY** zero-initialize the Module's linear memory and global state for non-native modules between Weaves for the same Agent ID.

---

## 5. Data Lifecycle

Filament treats data as a stream of events. Modules communicate by reading and writing events to a transient timeline, which is then persisted based on policy.

### 5.1 The Timeline

The **Timeline** is the authoritative log of the Process.

- **Index:** Events are referenced by a 64-bit unsigned integer. Indices **MUST** be strictly monotonically increasing.
- **Access:** Modules read historical data via a cursor API to support pagination of large datasets.

### 5.2 Timeline Retention Policies

Different domains require different data retention policies. The Kernel enforces one of the following policies per Process, defined in the Manifest.

- **Strict Policy (Append-Only):** Events are immutable once committed. No deletion or modification is permitted. Suitable for audit logs.
- **Prunable Policy (Retention):** The Timeline maintains a strictly ordered tail of events. The Kernel **MAY** delete the oldest events to reclaim storage space. Existing indices are **NOT** shifted; deleted indices become permanently invalid.
- **Mutable Policy (Redaction):** Events may be logically redacted using **Tombstones**. The event payload is wiped, but the index and header remain to preserve causal ordering.

### 5.3 The Staging Area

The **Staging Area** is the transient medium for inter-module communication. It is zero-initialized at the start of every Weave. Events written here are tentative proposals until the Weave commits.

### 5.4 Blob Mapping

For high-bandwidth assets, such as Lidar point clouds or Video frames, the Kernel uses **Blob Mapping** to avoid copy overhead.

- **Creation:** Unmanaged contexts allocate Blobs via `filament_blob_alloc`. This may return a pointer to DMA-safe memory.
- **Ephemerality:** Blobs created during a Weave are destroyed at the end of that Weave unless they are explicitly referenced in the committed Timeline or retained.
- **Retention:** Stateful modules wishing to preserve a Blob across Weaves without publishing it **MUST** call `filament_blob_retain`.
- **Safety Trap:** Accessing a Blob ID that was not retained or committed in a previous Weave **MUST** trigger a Kernel Trap.

---

# Part II: The Kernel Interface

## 1. Type Aliases

| Alias                   | Fundamental | Description                                                       |
| :---------------------- | :---------- | :---------------------------------------------------------------- |
| `FilamentAddress`       | `u64`       | Pointer in unmanaged modules or linear offset in managed modules. |
| `FilamentContextHandle` | `u64`       | Opaque handle to the current execution context.                   |
| `FilamentBlobHandle`    | `u64`       | Opaque handle to a kernel-managed Blob.                           |
| `FilamentCursorHandle`  | `u64`       | Opaque handle to a timeline cursor.                               |
| `FilamentProcessHandle` | `u64`       | Opaque handle to a process                                        |

## 2. Constants and Enumerations

### 2.1 Orphan Constants

| Symbol           | Type  | Value        | Description             |
| :--------------- | :---- | :----------- | :---------------------- |
| `FILAMENT_NULL`  | `u64` | `0`          | Null pointer or handle. |
| `FILAMENT_MAGIC` | `u32` | `0x9D2F8A41` | Filament module magic.  |

### 2.2 System Limits

| Symbol                    | Type  | Value   | Description                                       |
| :------------------------ | :---- | :------ | :------------------------------------------------ |
| `FILAMENT_MAX_RECURSION`  | `u32` | `64`    | Max depth for nested generic value structures.    |
| `FILAMENT_MAX_URI_LEN`    | `u32` | `2048`  | Max length for URI strings.                       |
| `FILAMENT_MIN_BLOB_BYTES` | `u64` | `128`   | Threshold size before kernel prefers indirection. |
| `FILAMENT_MIN_BUS_BYTES`  | `u64` | `65536` | Minimum staging area size.                        |

### 2.3 Success Codes (`i64`)

| Value | Symbol           | Description                                                                                                             |
| :---- | :--------------- | :---------------------------------------------------------------------------------------------------------------------- |
| `0`   | `FILAMENT_PARK`  | The module has no runnable work and is waiting for external events. The Kernel **SHOULD** remove it from the run queue. |
| `1`   | `FILAMENT_YIELD` | The module has pending work but is yielding time. The Kernel **SHOULD** keep it in the run queue.                       |

### 2.3 Error Codes (`i64`)

| Symbol                   | Value | Description                |
| :----------------------- | :---- | :------------------------- |
| `FILAMENT_ERR_UNKNOWN`   | `-1`  | Unknown error.             |
| `FILAMENT_ERR_PERM`      | `-2`  | Permission denied.         |
| `FILAMENT_ERR_NOT_FOUND` | `-3`  | Resource not found.        |
| `FILAMENT_ERR_IO`        | `-4`  | Physical I/O failure.      |
| `FILAMENT_ERR_OOM`       | `-5`  | Out of memory.             |
| `FILAMENT_ERR_INVALID`   | `-6`  | Invalid argument.          |
| `FILAMENT_ERR_TIMEOUT`   | `-7`  | Execution budget exceeded. |
| `FILAMENT_ERR_TYPE`      | `-8`  | Schema or type mismatch.   |

### 2.4 Value Types (`u32`)

| Symbol               | ID  | Description              |
| :------------------- | :-- | :----------------------- |
| `FILAMENT_VAL_UNIT`  | `0` | None.                    |
| `FILAMENT_VAL_BOOL`  | `1` | Boolean.                 |
| `FILAMENT_VAL_I64`   | `2` | Signed 64-bit integer.   |
| `FILAMENT_VAL_U64`   | `3` | Unsigned 64-bit integer. |
| `FILAMENT_VAL_F64`   | `4` | Double precision float.  |
| `FILAMENT_VAL_STR`   | `5` | UTF-8 string view.       |
| `FILAMENT_VAL_BLOB`  | `6` | Blob reference.          |
| `FILAMENT_VAL_MAP`   | `7` | Key-value array.         |
| `FILAMENT_VAL_LIST`  | `8` | Value array.             |
| `FILAMENT_VAL_BYTES` | `9` | Opaque byte array.       |

### 2.5 I/O Flags (`u32`)

| Symbol                     | Bit | Description                                                                                |
| :------------------------- | :-- | :----------------------------------------------------------------------------------------- |
| `FILAMENT_IO_RAW`          | `0` | Payload is raw bytes.                                                                      |
| `FILAMENT_IO_VAL`          | `1` | Payload is `FilamentValue`.                                                                |
| `FILAMENT_IO_DMA`          | `2` | Request direct memory access.                                                              |
| `FILAMENT_IO_DMA_OPTIONAL` | `3` | Request direct memory access if available; otherwise, fall back to indirect memory access. |

### 2.6 Data Formats (`u32`)

| Symbol                | ID  | Description         |
| :-------------------- | :-- | :------------------ |
| `FILAMENT_FMT_BINARY` | `0` | Opaque binary data. |
| `FILAMENT_FMT_JSON`   | `1` | UTF-8 JSON.         |
| `FILAMENT_FMT_PROTO`  | `2` | Protocol Buffers.   |
| `FILAMENT_FMT_TEXT`   | `3` | UTF-8 Text.         |

### 2.7 Scheduling Policies (`u8`)

| Symbol                     | Value | Description                                                                                                                     |
| :------------------------- | :---- | :------------------------------------------------------------------------------------------------------------------------------ |
| `FILAMENT_SCHED_SHARED`    | `0`   | Scheduled on the main thread pool. The process participates in a global weave barrier.                                          |
| `FILAMENT_SCHED_DEDICATED` | `1`   | Mapped to a dedicated core. The process executes on an independent timeline and is **not** blocked by the global weave barrier. |

### 2.8 Execution Contexts (`u8`)

| Symbol                       | Value | Description                                                                                     |
| :--------------------------- | :---- | :---------------------------------------------------------------------------------------------- |
| `FILAMENT_CONTEXT_LOGIC`     | `0`   | No I/O allowed except via the Bus. Deterministic arithmetic is enforced via WASM.               |
| `FILAMENT_CONTEXT_MANAGED`   | `1`   | Access to system resources is allowed, including network access. **MAY** use a managed runtime. |
| `FILAMENT_CONTEXT_UNMANAGED` | `2`   | Access to system hardware is allowed. Real-time scheduling constraints apply.                   |

### 2.9 Wake Flags (`u32`)

Bitmask indicating why the module was scheduled.

| Symbol                    | Bit | Description                                         |
| :------------------------ | :-- | :-------------------------------------------------- |
| `FILAMENT_WAKE_INIT`      | `0` | First execution.                                    |
| `FILAMENT_WAKE_IO`        | `1` | Data is available on a subscribed Topic or Channel. |
| `FILAMENT_WAKE_TIMER`     | `2` | A requested timer has expired.                      |
| `FILAMENT_WAKE_YIELD`     | `3` | Resuming from a voluntary `FILAMENT_YIELD`.         |
| `FILAMENT_WAKE_LIFECYCLE` | `4` | A lifecycle event is pending.                       |

### 2.10 Memory Map Flags (`u32`)

| Symbol                | Bit | Description                                |
| :-------------------- | :-- | :----------------------------------------- |
| `FILAMENT_MMAP_READ`  | `0` | Request read access to the memory region.  |
| `FILAMENT_MMAP_WRITE` | `1` | Request write access to the memory region. |
| `FILAMENT_MMAP_EXEC`  | `2` | Request execute access.                    |

## 3. Data Structures

### 3.1 FilamentString

**Size:** 16 bytes

A non-owning view into a UTF-8 string buffer.

| Offset | Field | Type              | Description              |
| :----- | :---- | :---------------- | :----------------------- |
| 0      | `ptr` | `FilamentAddress` | Offset in module memory. |
| 8      | `len` | `u64`             | Length in bytes.         |

### 3.2 FilamentBlob

**Size:** 24 bytes

A handle to a kernel-managed memory buffer.

| Offset | Field    | Type                 | Description               |
| :----- | :------- | :------------------- | :------------------------ |
| 0      | `handle` | `FilamentBlobHandle` | Opaque handle.            |
| 8      | `ptr`    | `FilamentAddress`    | Address in module memory. |
| 16     | `size`   | `u64`                | Total size in bytes.      |

### 3.3 FilamentArray

**Size:** 16 bytes

A generic container for contiguous sequences.

| Offset | Field | Type              | Description               |
| :----- | :---- | :---------------- | :------------------------ |
| 0      | `ptr` | `FilamentAddress` | Pointer to data elements. |
| 8      | `len` | `u64`             | Number of elements.       |

### 3.4 FilamentPair

**Size:** 48 bytes

A key-value pair used in maps.

| Offset | Field   | Type             | Description                 |
| :----- | :------ | :--------------- | :-------------------------- |
| 0      | `key`   | `FilamentString` | Key string (16 bytes).      |
| 16     | `value` | `FilamentValue`  | Value container (32 bytes). |

### 3.5 FilamentValue

**Size:** 32 bytes

Generic tagged union container. All fields in the `data` union start at Offset 8.

| Offset | Field           | Type             | Description                              |
| :----- | :-------------- | :--------------- | :--------------------------------------- |
| 0      | `tag`           | `u32`            | Type enum ID.                            |
| 4      | `flags`         | `u32`            | Metadata flags.                          |
| 8      | `data.as_u64`   | `u64`            | Unsigned 64-bit integer.                 |
| 8      | `data.as_i64`   | `i64`            | Signed 64-bit integer.                   |
| 8      | `data.as_f64`   | `f64`            | Double precision float.                  |
| 8      | `data.as_bool`  | `u8`             | Boolean (0=False, 1=True).               |
| 8      | `data.as_str`   | `FilamentString` | String view (16 bytes).                  |
| 8      | `data.as_blob`  | `FilamentBlob`   | Blob reference (24 bytes).               |
| 8      | `data.as_map`   | `FilamentArray`  | Pointer to `FilamentPair[]` (16 bytes).  |
| 8      | `data.as_list`  | `FilamentArray`  | Pointer to `FilamentValue[]` (16 bytes). |
| 8      | `data.as_bytes` | `FilamentArray`  | Pointer to byte array (16 bytes).        |

**Kernel Requirements:**

| ID           | Requirement                                                                                                            |
| :----------- | :--------------------------------------------------------------------------------------------------------------------- |
| **KR-3.5.1** | When reading a `FilamentValue`, the Kernel **MUST** validate the `data` union member corresponding to the `tag` field. |
| **KR-3.5.2** | If `tag` is `FILAMENT_VAL_STR`, the Kernel **MUST** validate that the data is valid UTF-8.                             |
| **KR-3.5.3** | The Kernel **MUST** limit recursion depth when validating nested structures to `FILAMENT_MAX_RECURSION`.               |

### 3.6 FilamentTraceContext

**Size:** 32 bytes

W3C Trace Parent compatible propagation context.

| Offset | Field         | Type    | Description               |
| :----- | :------------ | :------ | :------------------------ |
| 0      | `trace_id_hi` | `u64`   | High 64-bits of trace ID. |
| 8      | `trace_id_lo` | `u64`   | Low 64-bits of trace ID.  |
| 16     | `span_id`     | `u64`   | Parent span ID.           |
| 24     | `flags`       | `u8`    | Trace flags.              |
| 25     | `_pad`        | `u8[7]` | Reserved.                 |

### 3.7 FilamentEventHeader

**Size:** 128 bytes

The fixed-size header prefixing every event in the staging area.

| Offset | Field        | Type                   | Description                                             |
| :----- | :----------- | :--------------------- | :------------------------------------------------------ |
| 0      | `total_len`  | `u32`                  | Total byte size including header, payload, and padding. |
| 4      | `flags`      | `u32`                  | Copy of I/O Flags used to write event.                  |
| 8      | `id`         | `u64`                  | Sequence Index.                                         |
| 16     | `timestamp`  | `u64`                  | Virtual Time of commit.                                 |
| 24     | `schema_id`  | `u64`                  | Hash of the Data Schema.                                |
| 32     | `auth_agent` | `u64`                  | Source Process/Agent ID.                                |
| 40     | `auth_user`  | `u64`                  | Source User/Role ID.                                    |
| 48     | `trace`      | `FilamentTraceContext` | Trace propagation data (32 bytes).                      |
| 80     | `topic_len`  | `u32`                  | Length of Topic URI.                                    |
| 84     | `data_len`   | `u32`                  | Length of Payload Data.                                 |
| 88     | `encoding`   | `u32`                  | MIME Enum.                                              |
| 92     | `_pad`       | `u8[36]`               | Reserved.                                               |

### 3.8 FilamentResourceLimits

**Size:** 24 bytes

Universal resource quota structure.

| Offset | Field        | Type    | Description                                |
| :----- | :----------- | :------ | :----------------------------------------- |
| 0      | `mem_max`    | `u64`   | Hard memory limit in bytes.                |
| 8      | `time_limit` | `u64`   | Execution budget per weave in nanoseconds. |
| 16     | `priority`   | `u8`    | Preemption level.                          |
| 17     | `policy`     | `u8`    | Scheduling strategy.                       |
| 18     | `_pad`       | `u8[6]` | Reserved.                                  |

### 3.9 FilamentHostInfo

**Size:** 48 bytes

| Offset | Field      | Type                     | Description                                  |
| :----- | :--------- | :----------------------- | :------------------------------------------- |
| 0      | `limits`   | `FilamentResourceLimits` | The resource quota assigned to this process. |
| 24     | `bus_size` | `u64`                    | Max staging area size.                       |
| 32     | `formats`  | `u32`                    | Bitmask of supported encodings.              |
| 36     | `cores`    | `u32`                    | Available cores.                             |
| 40     | `_pad`     | `u8[8]`                  | Reserved.                                    |

### 3.10 FilamentModuleInfo

**Size:** 56 bytes

Returned by `filament_get_info`.

| Offset | Field      | Type             | Description               |
| :----- | :--------- | :--------------- | :------------------------ |
| 0      | `magic`    | `u32`            | Must be `FILAMENT_MAGIC`. |
| 4      | `abi_ver`  | `u32`            | Packed ABI Version.       |
| 8      | `mod_type` | `u32`            | Lifecycle ID.             |
| 12     | `_pad`     | `u32`            | Reserved.                 |
| 16     | `mem_req`  | `u64`            | Minimum memory bytes.     |
| 24     | `name`     | `FilamentString` | Module Name.              |
| 40     | `version`  | `FilamentString` | Module Version.           |

### 3.11 FilamentConfig

**Size:** 16 bytes

Container for injected configuration.

| Offset | Field     | Type              | Description                  |
| :----- | :-------- | :---------------- | :--------------------------- |
| 0      | `count`   | `u64`             | Number of pairs.             |
| 8      | `entries` | `FilamentAddress` | Pointer to `FilamentPair[]`. |

### 3.12 FilamentChannelDefinition

**Size:** 40 bytes

Embedded definition for channel creation.

| Offset | Field       | Type             | Description                          |
| :----- | :---------- | :--------------- | :----------------------------------- |
| 0      | `schema`    | `FilamentString` | Schema URI.                          |
| 16     | `capacity`  | `u64`            | Ring buffer size in event count.     |
| 24     | `msg_size`  | `u64`            | Max size of event envelope in bytes. |
| 36     | `root_tag`  | `u32`            | `FilamentValueType` of the channel.  |
| 32     | `direction` | `u32`            | 1=Inbound, 2=Outbound.               |

### 3.13 FilamentModuleDefinition

**Size:** 64 bytes

Defines a single executable unit within the Process Pipeline.

| Offset | Field     | Type              | Description                                                          |
| :----- | :-------- | :---------------- | :------------------------------------------------------------------- |
| 0      | `alias`   | `FilamentString`  | Unique identifier for this instance (e.g., "audio_engine").          |
| 16     | `source`  | `FilamentString`  | URI to the binary artifact.                                          |
| 32     | `digest`  | `FilamentString`  | **SHA-256** Content Hash of the artifact for integrity verification. |
| 48     | `config`  | `FilamentAddress` | Pointer to `FilamentConfig` (Static Configuration Map).              |
| 56     | `context` | `u32`             | Execution Context ID                                                 |
| 60     | `_pad`    | `u32`             | Reserved.                                                            |

### 3.14 FilamentProcessStatus

**Size:** 24 bytes

Payload for the `filament/process/status` event.

| Offset | Field    | Type                    | Description                               |
| :----- | :------- | :---------------------- | :---------------------------------------- |
| 0      | `handle` | `FilamentProcessHandle` | Opaque handle to the process              |
| 8      | `code`   | `i64`                   | Exit code or error ID.                    |
| 16     | `state`  | `u32`                   | 0=Started, 1=Exited, 2=Killed, 3=Crashed. |
| 20     | `_pad`   | `u32`                   | Reserved.                                 |

### 3.15 FilamentProcessLifecycleEvent

**Size:** 16 bytes

Payload for the `filament/process/lifecycle` protocol.

| Offset | Field     | Type  | Description                             |
| :----- | :-------- | :---- | :-------------------------------------- |
| 0      | `timeout` | `u64` | Nanoseconds remaining before hard kill. |
| 8      | `cmd`     | `u32` | 1=Stop, 2=Reload, 3=HealthCheck.        |
| 12     | `_pad`    | `u32` | Reserved.                               |

### 3.16 FilamentReadArgs

**Size:** 40 bytes

Arguments for `filament_read`.

| Offset | Field     | Type              | Description                                                 |
| :----- | :-------- | :---------------- | :---------------------------------------------------------- |
| 0      | `topic`   | `FilamentString`  | Filter topic URI.                                           |
| 16     | `start`   | `u64`             | The sequential index of the event within the current batch. |
| 24     | `out_ptr` | `FilamentAddress` | Destination buffer pointer.                                 |
| 32     | `out_cap` | `u64`             | Capacity of destination buffer.                             |

### 3.17 FilamentWriteArgs

**Size:** 40 bytes

Arguments for `filament_write`.

| Offset | Field   | Type              | Description                   |
| :----- | :------ | :---------------- | :---------------------------- |
| 0      | `topic` | `FilamentString`  | Topic URI.                    |
| 16     | `data`  | `FilamentAddress` | Payload data pointer.         |
| 24     | `len`   | `u64`             | Length of payload.            |
| 32     | `flags` | `u32`             | Bitmask of `FilamentIOFlags`. |
| 36     | `_pad`  | `u32`             | Reserved.                     |

### 3.18 FilamentBlobAllocArgs

**Size:** 24 bytes

Arguments for `filament_blob_alloc`.

| Offset | Field     | Type              | Description                                          |
| :----- | :-------- | :---------------- | :--------------------------------------------------- |
| 0      | `out_ref` | `FilamentAddress` | Pointer to `FilamentBlob` struct to fill.            |
| 8      | `size`    | `u64`             | Size in bytes.                                       |
| 16     | `flags`   | `u32`             | `FILAMENT_IO_DMA`, `FILAMENT_IO_DMA_OPTIONAL`, or 0. |
| 20     | `_pad`    | `u32`             | Reserved.                                            |

### 3.19 FilamentBlobMapArgs

**Size:** 24 bytes

Arguments for `filament_blob_map`.

| Offset | Field     | Type                 | Description                               |
| :----- | :-------- | :------------------- | :---------------------------------------- |
| 0      | `out_ref` | `FilamentAddress`    | Pointer to `FilamentBlob` struct to fill. |
| 8      | `handle`  | `FilamentBlobHandle` | The blob to map.                          |
| 16     | `flags`   | `u32`                | Bitmask of `FilamentMapFlags`.            |
| 20     | `_pad`    | `u32`                | Reserved.                                 |

### 3.20 FilamentBlobRetainArgs

**Size:** 8 bytes

Arguments for `filament_blob_retain`.

| Offset | Field    | Type                 | Description         |
| :----- | :------- | :------------------- | :------------------ |
| 0      | `handle` | `FilamentBlobHandle` | The blob to retain. |

### 3.21 FilamentChannelCreateArgs

**Size:** 56 bytes

Arguments for `filament_channel_create`.

| Offset | Field     | Type                        | Description                             |
| :----- | :-------- | :-------------------------- | :-------------------------------------- |
| 0      | `def`     | `FilamentChannelDefinition` | Channel properties (Embedded).          |
| 40     | `out_ptr` | `FilamentAddress`           | Pointer to buffer for the topic string. |
| 48     | `out_cap` | `u64`                       | Capacity of the output buffer.          |

### 3.22 FilamentProcessSpawnArgs

**Size:** 64 bytes

Arguments for `filament_process_spawn`.

| Offset | Field      | Type                     | Description                                                                 |
| :----- | :--------- | :----------------------- | :-------------------------------------------------------------------------- |
| 0      | `modules`  | `FilamentArray`          | Pointer to `FilamentModuleDefinition[]`. Defines the execution pipeline.    |
| 16     | `bindings` | `FilamentArray`          | Pointer to `FilamentPair[]`. Maps internal Topics to external Channel URIs. |
| 32     | `limits`   | `FilamentResourceLimits` | Resource limits to assign to the child.                                     |
| 56     | `_pad`     | `u8[8]`                  | Reserved.                                                                   |

### 3.23 FilamentProcessTerminateArgs

**Size:** 8 bytes

Arguments for `filament_process_terminate`.

| Offset | Field    | Type                    | Description                                |
| :----- | :------- | :---------------------- | :----------------------------------------- |
| 0      | `handle` | `FilamentProcessHandle` | Opaque handle to the process to terminate. |

### 3.24 FilamentTimelineOpenArgs

**Size:** 48 bytes

Arguments for `filament_tl_open`.

| Offset | Field   | Type             | Description                        |
| :----- | :------ | :--------------- | :--------------------------------- |
| 0      | `topic` | `FilamentString` | Filter by topic URI.               |
| 16     | `start` | `u64`            | Inclusive start timestamp or tick. |
| 24     | `end`   | `u64`            | Exclusive end timestamp or tick.   |
| 32     | `limit` | `u64`            | Max items to return.               |
| 40     | `desc`  | `u8`             | 1=Descending order.                |
| 41     | `_pad`  | `u8[7]`          | Reserved.                          |

### 3.25 FilamentTimelineNextArgs

**Size:** 24 bytes

Arguments for `filament_tl_next`.

| Offset | Field     | Type                   | Description                 |
| :----- | :-------- | :--------------------- | :-------------------------- |
| 0      | `handle`  | `FilamentCursorHandle` | Cursor handle.              |
| 8      | `out_ptr` | `FilamentAddress`      | Destination buffer pointer. |
| 16     | `buf_cap` | `u64`                  | Buffer capacity.            |

### 3.26 FilamentTimelineCloseArgs

**Size:** 8 bytes

Arguments for `filament_tl_close`.

| Offset | Field    | Type                   | Description    |
| :----- | :------- | :--------------------- | :------------- |
| 0      | `handle` | `FilamentCursorHandle` | Cursor handle. |

### 3.27 FilamentInitArgs

**Size:** 32 bytes

Arguments for `filament_init`.

| Offset | Field    | Type              | Description                    |
| :----- | :------- | :---------------- | :----------------------------- |
| 0      | `host`   | `FilamentAddress` | Pointer to `FilamentHostInfo`. |
| 8      | `config` | `FilamentAddress` | Pointer to `FilamentConfig`.   |
| 16     | `_pad`   | `u8[16]`          | Reserved.                      |

#### 3.28 FilamentWeaveArgs

**Size:** 128 bytes

Arguments for `filament_weave`.

| Offset | Field        | Type                   | Description                                          |
| :----- | :----------- | :--------------------- | ---------------------------------------------------- |
| 0      | `time_limit` | `u64`                  | Execution budget in nanoseconds.                     |
| 8      | `res_used`   | `u64`                  | Compute units consumed.                              |
| 16     | `res_max`    | `u64`                  | Compute unit limit.                                  |
| 24     | `mem_max`    | `u64`                  | Hard memory limit.                                   |
| 32     | `rand_seed`  | `u64`                  | Cryptographic seed.                                  |
| 40     | `virt_time`  | `u64`                  | Virtual simulated time.                              |
| 48     | `trace`      | `FilamentTraceContext` | Trace context (32 bytes).                            |
| 80     | `delta_ns`   | `u64`                  | Time since last tick.                                |
| 88     | `tick`       | `u64`                  | Monotonic tick count.                                |
| 96     | `wake_flags` | `u32`                  | Bitmask of `FilamentWakeFlags`.                      |
| 100    | `_pad`       | `u32`                  | Reserved.                                            |
| 104    | `user_data`  | `u64`                  | Opaque value preserved from the previous Yield/Park. |
| 112    | `_pad2`      | `[u8; 16]`             | Reserved.                                            |

### 3.29 FilamentLogRecord

**Size:** 32 bytes

Payload for the `filament/core/log` event. This structure defines the standard format for structured logging.

| Offset | Field     | Type              | Description                                                        |
| :----- | :-------- | :---------------- | :----------------------------------------------------------------- |
| 0      | `level`   | `u32`             | 0=Debug, 1=Info, 2=Warn, 3=Error.                                  |
| 4      | `_pad`    | `u32`             | Reserved.                                                          |
| 8      | `msg`     | `FilamentString`  | Log message text.                                                  |
| 24     | `context` | `FilamentAddress` | Pointer to `FilamentValue` (Map) containing structured attributes. |

### 3.30 FilamentPanicRecord

**Size:** 24 bytes

Payload for the `filament/core/panic` event.

| Offset | Field    | Type             | Description                           |
| :----- | :------- | :--------------- | :------------------------------------ |
| 0      | `code`   | `i64`            | Application-specific error code.      |
| 8      | `reason` | `FilamentString` | Human-readable debugging information. |

## 4. Kernel Interface Functions

### 4.1 filament_read

Reads events from an input source.

**Args:**

| Parameter | Type                    | Description                             |
| :-------- | :---------------------- | :-------------------------------------- |
| `ctx`     | `FilamentContextHandle` | Handle to the current Filament context. |
| `args`    | `FilamentAddress`       | Pointer to `FilamentReadArgs`.          |

**Return:** `i64` - Total bytes written or error code.

**Kernel Requirements:**

| ID           | Requirement                                                                                                                                                                                                                          |
| :----------- | :----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **KR-4.1.1** | If `args->topic` matches a verified dynamic channel URI, the Kernel **MUST** destructively read the oldest available event from the ring buffer. The `args->start` field **MUST** be ignored.                                        |
| **KR-4.1.2** | If `args->topic` is a static manifest topic or `FILAMENT_NULL`, the Kernel **MUST** read sequentially from the staging area using `args->start` as the cursor.                                                                       |
| **KR-4.1.3** | If `args->out_ptr` is `FILAMENT_NULL`, the Kernel **MUST** return the required size in bytes without performing a write.                                                                                                             |
| **KR-4.1.4** | The Kernel **MUST NOT** copy partial events. If `args->out_cap` is insufficient for the next complete event, the Kernel **MUST** stop writing and return the current total bytes (or `FILAMENT_ERR_OOM` if zero bytes were written). |
| **KR-4.1.5** | If the event contains `FILAMENT_VAL_BLOB` types, the Kernel **MUST** copy the event structure and handle into `args->out_ptr` but **MUST NOT** copy the backing blob data.                                                           |
| **KR-4.1.6** | The Kernel **MUST** ensure that all pointers (e.g., in `FilamentString`, `FilamentArray`) within the returned events point to valid offsets within the destination buffer (Pointer Relocation).                                      |

**Module Requirements:**

| ID           | Constraint                                                                                                                                                  |
| :----------- | :---------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **MR-4.1.1** | `args->out_ptr` **MUST** point to a valid buffer of at least `args->out_cap` bytes in the module's memory space, unless `args->out_ptr` is `FILAMENT_NULL`. |
| **MR-4.1.2** | If `args->topic` is provided, it **MUST** be a valid UTF-8 string.                                                                                          |

### 4.2 filament_write

Writes an event to an output destination.

**Args:**

| Parameter | Type                    | Description                             |
| :-------- | :---------------------- | :-------------------------------------- |
| `ctx`     | `FilamentContextHandle` | Handle to the current Filament context. |
| `args`    | `FilamentAddress`       | Pointer to `FilamentWriteArgs`.         |

**Return:** `i64` - Bytes written or error code.

**Kernel Requirements:**

| ID           | Requirement                                                                                                                                                                                 |
| :----------- | :------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| **KR-4.2.1** | If `args->flags` contains `FILAMENT_IO_VAL`, the Kernel **MUST** recursively validate and serialize the payload structure against the schema bound to the destination topic.                |
| **KR-4.2.2** | The Kernel **MUST** check if `args->topic` matches a verified dynamic channel URI.                                                                                                          |
| **KR-4.2.3** | If the topic is a dynamic channel, the Kernel **MUST** append the event to the ring buffer. If the buffer is full, the Kernel **MUST** return `FILAMENT_ERR_IO` immediately (Non-Blocking). |
| **KR-4.2.4** | If the topic is a static topic, the Kernel **MUST** write the event to the staging area.                                                                                                    |
| **KR-4.2.5** | If the payload contains `FILAMENT_VAL_BLOB`, the Kernel **MUST** transfer the blob reference count to the channel or recipient. The blob data itself is **NOT** copied.                     |
| **KR-4.2.6** | If the topic is a dynamic channel, the Kernel **MUST** return `FILAMENT_ERR_INVALID` if `args->len` exceeds the `msg_size` configured in the Channel Definition.                            |

**Module Requirements:**

| ID           | Constraint                                                                                                     |
| :----------- | :------------------------------------------------------------------------------------------------------------- |
| **MR-4.2.1** | `args->topic` **MUST** be a valid UTF-8 string.                                                                |
| **MR-4.2.2** | If `args->flags` contains `FILAMENT_IO_VAL`, `args->data` **MUST** point to a valid `FilamentValue` structure. |

### 4.3 filament_blob_alloc

Allocates a new blob.

**Args:**

| Parameter | Type                    | Description                             |
| :-------- | :---------------------- | :-------------------------------------- |
| `ctx`     | `FilamentContextHandle` | Handle to the current Filament context. |
| `args`    | `FilamentAddress`       | Pointer to `FilamentBlobAllocArgs`.     |

**Return:** `i64` - `FILAMENT_OK` or error code.

**Kernel Requirements:**

| ID           | Requirement                                                                                                                                       |
| :----------- | :------------------------------------------------------------------------------------------------------------------------------------------------ |
| **KR-4.3.1** | If `args->flags` includes `FILAMENT_IO_DMA` but direct memory access is unavailable, the Kernel **MUST** return `FILAMENT_ERR_OOM`.               |
| **KR-4.3.2** | If `args->flags` includes `FILAMENT_IO_DMA_OPTIONAL` and direct memory access is unavailable, the Kernel **MUST** allocate standard memory.       |
| **KR-4.3.3** | If `args->flags` includes `FILAMENT_IO_DMA` AND `FILAMENT_IO_DMA_OPTIONAL`, the kernel must operate as if only `FILAMENT_IO_DMA_OPTIONAL` is set. |
| **KR-4.3.4** | The Kernel **MUST** deduct `args->size` from the module's available memory quota.                                                                 |
| **KR-4.3.5** | For system contexts, the Kernel **MUST NOT** perform dynamic heap allocations (`malloc`) during this call; it must use pre-allocated pools.       |

**Module Requirements:**

| ID           | Constraint                                                                                                       |
| :----------- | :--------------------------------------------------------------------------------------------------------------- |
| **MR-4.3.1** | The pointer returned in `args->out_ref` is valid **ONLY** for the duration of the current Weave unless retained. |

### 4.4 filament_blob_map

Maps an existing blob into the module's address space.

**Args:**

| Parameter | Type                    | Description                             |
| :-------- | :---------------------- | :-------------------------------------- |
| `ctx`     | `FilamentContextHandle` | Handle to the current Filament context. |
| `args`    | `FilamentAddress`       | Pointer to `FilamentBlobMapArgs`.       |

**Return:** `i64` - `FILAMENT_OK` or error code.

**Kernel Requirements:**

| ID           | Requirement                                                                                                                                 |
| :----------- | :------------------------------------------------------------------------------------------------------------------------------------------ |
| **KR-4.4.1** | The Kernel **MUST** verify the module has access rights to `args->id`.                                                                      |
| **KR-4.4.2** | The pointer returned in `args->out_ref` **MUST** be valid within the module's virtual memory space.                                         |
| **KR-4.4.3** | For native contexts, the returned pointer **MUST** be a direct pointer to the underlying buffer (Zero-Copy).                                |
| **KR-4.4.4** | The Kernel **MUST** return `FILAMENT_ERR_PERM` if `args->flags` requests permissions that are not granted by the underlying Blob reference. |

**Module Requirements:**

| ID           | Constraint                                                                                    |
| :----------- | :-------------------------------------------------------------------------------------------- |
| **MR-4.4.1** | The module **MUST** have access rights to the blob ID via an event or resource configuration. |

### 4.5 filament_blob_retain

Pins a blob ID to prevent garbage collection.

**Args:**

| Parameter | Type                    | Description                             |
| :-------- | :---------------------- | :-------------------------------------- |
| `ctx`     | `FilamentContextHandle` | Handle to the current Filament context. |
| `args`    | `FilamentAddress`       | Pointer to `FilamentBlobRetainArgs`.    |

**Return:** `i64` - `FILAMENT_OK` or error code.

**Kernel Requirements:**

| ID           | Requirement                                                                                                                                                                          |
| :----------- | :----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **KR-4.5.1** | The Kernel **MUST** verify that the calling module currently owns or holds a reference to `args->id`.                                                                                |
| **KR-4.5.2** | The Kernel **MUST** charge the retained size against the module's persistent storage quota.                                                                                          |
| **KR-4.5.3** | The Kernel **MUST** treat the retention request as provisional. If the Weave is discarded the Kernel **MUST** revert the reference count increment, ensuring the Blob is cleaned up. |

**Module Requirements:**

| ID           | Constraint                                                         |
| :----------- | :----------------------------------------------------------------- |
| **MR-4.5.1** | Modules in `stateless` contexts **SHOULD NOT** call this function. |

### 4.6 filament_tl_open

Opens a cursor to the process timeline.

**Args:**

| Parameter | Type                    | Description                             |
| :-------- | :---------------------- | :-------------------------------------- |
| `ctx`     | `FilamentContextHandle` | Handle to the current Filament context. |
| `args`    | `FilamentAddress`       | Pointer to `FilamentTimelineOpenArgs`.  |

**Return:** `i64` - `FilamentCursorHandle` or error code.

**Kernel Requirements:**

| ID           | Requirement                                                                                                                              |
| :----------- | :--------------------------------------------------------------------------------------------------------------------------------------- |
| **KR-4.6.1** | The Kernel **MUST** enforce access control lists, ensuring a module cannot query events it is not authorized to see via manifest inputs. |
| **KR-4.6.2** | Cursors **MUST** be invalidated if the underlying timeline segment is pruned.                                                            |

**Module Requirements:**

| ID           | Constraint                                                   |
| :----------- | :----------------------------------------------------------- |
| **MR-4.6.1** | `args->topic` **MUST** point to a valid UTF-8 string prefix. |

### 4.7 filament_tl_next

Streams a batch of events from the cursor.

**Args:**

| Parameter | Type                    | Description                             |
| :-------- | :---------------------- | :-------------------------------------- |
| `ctx`     | `FilamentContextHandle` | Handle to the current Filament context. |
| `args`    | `FilamentAddress`       | Pointer to `FilamentTimelineNextArgs`.  |

**Return:** `i64` - Bytes written, `0` for EOF, or error code.

**Kernel Requirements:**

| ID           | Requirement                                                                                               |
| :----------- | :-------------------------------------------------------------------------------------------------------- |
| **KR-4.7.1** | If no more events are available, the Kernel **MUST** return `0` (EOF).                                    |
| **KR-4.7.2** | The Kernel **MUST NOT** write partial events.                                                             |
| **KR-4.7.3** | The Kernel **MUST** relocate pointers within the output buffer to be valid in the module's address space. |

### 4.8 filament_tl_close

Closes the cursor.

**Args:**

| Parameter | Type                    | Description                             |
| :-------- | :---------------------- | :-------------------------------------- |
| `ctx`     | `FilamentContextHandle` | Handle to the current Filament context. |
| `args`    | `FilamentAddress`       | Pointer to `FilamentTimelineCloseArgs`. |

**Return:** `i64` - `FILAMENT_OK`.

**Kernel Requirements:**

| ID           | Requirement                                                               |
| :----------- | :------------------------------------------------------------------------ |
| **KR-4.8.1** | The Kernel **MUST** release any resources associated with `args->handle`. |

### 4.9 filament_channel_create

Allocates a typed communication channel.

**Args:**

| Parameter | Type                    | Description                             |
| :-------- | :---------------------- | :-------------------------------------- |
| `ctx`     | `FilamentContextHandle` | Handle to the current Filament context. |
| `args`    | `FilamentAddress`       | Pointer to `FilamentChannelCreateArgs`. |

**Return:** `i64` - Positive channel handle ID or error code.

**Kernel Requirements:**

| ID           | Requirement                                                                                                                                                                                              |
| :----------- | :------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **KR-4.9.1** | The Kernel **MUST** generate a unique virtual topic URI within the reserved namespace and write it to `args->out_ptr`.                                                                                   |
| **KR-4.9.2** | The Kernel **MUST** allocate a fixed-size ring buffer of `args->def.capacity` slots.                                                                                                                     |
| **KR-4.9.3** | The Kernel **MUST** deduct `capacity * max_msg_size` from the creator's memory budget. If insufficient funds exist, return `FILAMENT_ERR_OOM`.                                                           |
| **KR-4.9.4** | When a channel is destroyed, the Kernel **MUST** iterate over all pending events in the ring buffer. If an event contains a `FILAMENT_VAL_BLOB`, the Kernel **MUST** decrement the blob reference count. |
| **KR-4.9.5** | When a channel is destroyed, the Kernel **MUST** immediately unblock any threads suspended on that channel. These operations **MUST** return `FILAMENT_ERR_NOT_FOUND`.                                   |

**Module Requirements:**

| ID           | Constraint                                       |
| :----------- | :----------------------------------------------- |
| **MR-4.9.1** | `args->def.capacity` **MUST** be greater than 0. |

### 4.10 filament_process_spawn

Instantiates a new child process.

**Args:**

| Parameter | Type                    | Description                             |
| :-------- | :---------------------- | :-------------------------------------- |
| `ctx`     | `FilamentContextHandle` | Handle to the current Filament context. |
| `args`    | `FilamentAddress`       | Pointer to `FilamentProcessSpawnArgs`.  |

**Return:** `i64` - `FilamentProcessHandle` or error code.

**Kernel Requirements:**

| ID             | Requirement                                                                                                                                                                                                          |
| :------------- | :------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **KR-4.10.1**  | The Kernel **MUST** perform a deep copy of the `modules`, `bindings`, and `limits` structures to ensure isolation.                                                                                                   |
| **KR-4.10.2**  | For every topic in the `channels` binding map, the Kernel **MUST** enforce exact string equality between the child's declared topic schema URI and the channel's schema URI.                                         |
| **KR-4.10.3**  | The Kernel **MUST** deduct `limits.mem_max` from the parent's memory budget.                                                                                                                                         |
| **KR-4.10.4**  | The Kernel **MUST** assign process IDs using a strictly monotonic counter.                                                                                                                                           |
| **KR-4.10.5**  | If `limits.policy` is `FILAMENT_SCHED_DEDICATED`, the Kernel **MUST** allocate a dedicated thread if exokernel or physical core if unikernel.                                                                        |
| **KR-4.10.6**  | If `limits.policy` is `FILAMENT_SCHED_DEDICATED`, the Kernel **MUST NOT** block the global weave cycle waiting for this process. Synchronization must occur solely via Channels.                                     |
| **KR-4.10.7**  | For each module in `modules`, the Kernel **MUST** verify that the Content Hash of the loaded binary matches the `digest` string provided. If they do not match, the spawn **MUST** fail with `FILAMENT_ERR_INVALID`. |
| **KR-4.10.8**  | The Kernel **MUST** validate that the requested `context` is permitted by the Host policy (e.g., preventing a `LOGIC` process from spawning a `DRIVER` module).                                                      |
| **KR-4.10.9**  | For every binding in `channels`, the Kernel **MUST** verify that the `root_tag` declared by the Child's input/output definition matches the `root_tag` of the bound Channel.                                         |
| **KR-4.10.10** | The Kernel **MUST** invoke `filament_init` on the child process immediately upon instantiation. If `init` fails, the spawn is aborted. The execution cost of `init` is charged to the parent budget.                 |

### 4.11 filament_process_terminate

Schedules immediate termination of a process.

**Args:**

| Parameter | Type                    | Description                                |
| :-------- | :---------------------- | :----------------------------------------- |
| `ctx`     | `FilamentContextHandle` | Handle to the current Filament context.    |
| `args`    | `FilamentAddress`       | Pointer to `FilamentProcessTerminateArgs`. |

**Return:** `i64` - `FILAMENT_OK` or error code.

**Kernel Requirements:**

| ID            | Requirement                                                                                                                                                                        |
| :------------ | :--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **KR-4.11.1** | The Kernel **MUST** execute teardown in strict order: suspend child, destroy channels owned by child, and free child resources.                                                    |
| **KR-4.11.2** | Upon successful termination, the Kernel **MUST** credit the child's memory limit back to the parent's available budget.                                                            |
| **KR-4.11.3** | If a process is terminated in the same Weave cycle it was spawned, the Kernel **MUST** simply discard the pending spawn record. The Child **MUST NOT** be initialized or executed. |

## 5. Module Interface Functions

### 5.1 filament_get_info

Negotiates ABI version and memory requirements. This is the **Bootstrap Entry Point**.

| Parameter        | Type  | Description                                   |
| :--------------- | :---- | :-------------------------------------------- |
| `kernel_version` | `u32` | The packed ABI version supported by the Host. |
| `capabilities`   | `u64` | Reserved.                                     |

**Return:** `u64` - Pointer to `FilamentModuleInfo` struct within the module's static data segment.

**Kernel Requirements:**

| ID           | Requirement                                                                                                                           |
| :----------- | :------------------------------------------------------------------------------------------------------------------------------------ |
| **KR-5.1.1** | The Kernel **MUST** validate that the `version` string returned by the Module matches the version specified in the Module Definition. |
| **KR-5.1.2** | The Kernel **MUST** verify that the returned `mem_req` does not exceed the context limits.                                            |
| **KR-5.1.3** | The Kernel **MUST** reject the module if the returned `abi_ver` is incompatible with the Kernel's supported version logic.            |

**Module Requirements:**

| ID           | Constraint                                                                                                                                                  |
| :----------- | :---------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **MR-5.1.1** | The Module **MUST** return a pointer to a valid `FilamentModuleInfo` structure residing in the module's pre-allocated data segment (e.g., `.data` section). |
| **MR-5.1.2** | The `magic` field in the returned struct **MUST** match the system constant `FILAMENT_MAGIC`.                                                               |

### 5.2 filament_reserve

Allocates a contiguous block of memory within the module. This acts as the system allocator (e.g., `sbrk`) for the Kernel to inject configuration data.

| Parameter   | Type  | Description                       |
| :---------- | :---- | :-------------------------------- |
| `size`      | `u64` | Bytes requested by the Kernel.    |
| `alignment` | `u64` | Requested alignment (Power of 2). |
| `flags`     | `u32` | Reserved (0).                     |

**Return:** `FilamentAddress` - Pointer to the allocated block start, or `FILAMENT_NULL` on failure.

**Kernel Requirements:**

| ID           | Requirement                                                    |
| :----------- | :------------------------------------------------------------- |
| **KR-5.2.1** | The Kernel **MUST** provide a non-zero size.                   |
| **KR-5.2.2** | The Kernel **MUST** provide an alignment that is a power of 2. |

**Module Requirements:**

| ID           | Constraint                                                                                                                          |
| :----------- | :---------------------------------------------------------------------------------------------------------------------------------- |
| **MR-5.2.1** | The Module **MUST** return a valid, non-zero pointer to a contiguous region of at least `size` bytes.                               |
| **MR-5.2.2** | The returned pointer **MUST** be aligned to a multiple of `alignment`.                                                              |
| **MR-5.2.3** | The Module **MUST NOT** modify the contents of this region after returning the pointer, as the Kernel will populate it immediately. |

### 5.3 filament_init

Initializes the module using memory-resident configuration.

**Args:**

| Parameter | Type                    | Description                             |
| :-------- | :---------------------- | :-------------------------------------- |
| `ctx`     | `FilamentContextHandle` | Handle to the current Filament context. |
| `args`    | `FilamentAddress`       | Pointer to `FilamentInitArgs`.          |

**Return:** `i64` - `0` on Success, `-1` on Failure.

**Kernel Requirements:**

| ID           | Requirement                                                                                                                                                  |
| :----------- | :----------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **KR-5.3.1** | The Kernel **MUST** ensure `args_ptr` points to valid initialized memory within the module address space.                                                    |
| **KR-5.3.2** | The Kernel **MUST** populate `args->host->limits` with the exact resource quota enforced on the module for the current execution cycle.                      |
| **KR-5.3.3** | The Kernel **MUST** populate `args->host->cores` with the number of concurrent execution cores currently available. For single-threaded, this **MUST** be 0. |

**Module Requirements:**

| ID           | Constraint                                                                                                                                        |
| :----------- | :------------------------------------------------------------------------------------------------------------------------------------------------ |
| **MR-5.3.1** | The Module **MUST** deep-copy any data from `args_ptr` that it needs to persist, as the pointers are valid **ONLY** for the duration of the call. |
| **MR-5.3.2** | The Module **MUST** return `0` if initialization completes successfully.                                                                          |

### 5.4 filament_weave

Executes the logic cycle.

**Args:**

| Parameter | Type                    | Description                             |
| :-------- | :---------------------- | :-------------------------------------- |
| `ctx`     | `FilamentContextHandle` | Handle to the current Filament context. |
| `args`    | `FilamentAddress`       | Pointer to `FilamentWeaveArgs`.         |

**Return:** `i64`

- `FILAMENT_PARK`: Commit state. Suspend execution until new events arrive.
- `FILAMENT_YIELD`: Commit state. Reschedule execution as soon as possible.
- Negative Value: Error/Rollback.

**Kernel Requirements:**

| ID           | Requirement                                                                                                                                                                                                                                  |
| :----------- | :------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **KR-5.4.1** | The Kernel **MUST** populate `args->wake_flags` to indicate the cause of scheduling.                                                                                                                                                         |
| **KR-5.4.2** | The Kernel **MUST** guarantee that `args->ctx` is a valid handle for the duration of the call.                                                                                                                                               |
| **KR-5.4.3** | The Kernel **MUST** enforce the `args->time_limit` by preempting execution if the budget is exceeded.                                                                                                                                        |
| **KR-5.4.4** | If the Module returns `FILAMENT_YIELD`, the Kernel **MUST** commit the Staging Area but **SHOULD** keep the process in the "Ready" queue for immediate rescheduling (Cooperative Multitasking).                                              |
| **KR-5.4.5** | If the Module returns `FILAMENT_PARK`, the Kernel **MUST** inspect all input sources. If unread data exists, the Kernel **MUST** ignore the `PARK` request and treat it as `FILAMENT_YIELD` immediately rescheduling the module.             |
| **KR-5.4.6** | If the Module returned `FILAMENT_YIELD` or `FILAMENT_PARK` in the previous cycle, the Kernel **MUST** restore the stored `user_data` value into `args->user_data` for the current cycle. For `FILAMENT_WAKE_INIT`, this value **MUST** be 0. |
| **KR-5.4.7** | If the Execution Context is configured as **Stateless**, the Kernel **MUST** always provide `0` in `args->user_data` at the start of a Weave, ignoring any value set by the previous cycle.                                                  |

**Module Requirements:**

| ID           | Constraint                                                                                                            |
| :----------- | :-------------------------------------------------------------------------------------------------------------------- |
| **MR-5.4.1** | The Module **MUST NOT** use any entropy source other than `args->rand_seed`.                                          |
| **MR-5.4.2** | The Module **MUST** populate `args->user_data` before returning if it requires context restoration on the next cycle. |
| **MR-5.4.3** | The Module **MUST NOT** call `filament_weave` recursively.                                                            |
| **MR-5.4.4** | If running in a system context, the Module **MUST NOT** perform dynamic heap allocations during this call.            |

---

## 6. Additional Requirements

**Common Requirements:**

| ID           | Requirement                                                                                                                                                                                                            |
| :----------- | :--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **CR-6.0.1** | All multi-byte integers **MUST** use Little-Endian encoding.                                                                                                                                                           |
| **CR-6.0.2** | All ABI structures **MUST** be aligned to 8-byte boundaries.                                                                                                                                                           |
| **CR-6.0.3** | Native Hosts and Unmanaged Modules **MUST** adhere to the standard C Application Binary Interface of the **Host Platform**.                                                                                            |
| **CR-6.0.4** | WASM Hosts and Managed/Logical Modules **MUST** use the standard Wasm import/export convention, where arguments are passed on the operand stack or via locals, matching the `extern "C"` layout of the guest language. |

**Kernel Requirements:**

| ID           | Requirement                                                                                                                                                                 |
| :----------- | :-------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **KR-6.0.1** | The Kernel **MUST** synchronize access to dynamic channels across Execution Contexts ensuring that they are thread safe.                                                    |
| **KR-6.0.1** | The Kernel **MUST** perform a byte-wise comparison when matching URIs.                                                                                                      |
| **KR-6.0.2** | The Kernel **MUST** reject URIs containing embedded null characters (`\0`) or non-printable ASCII control characters with `FILAMENT_ERR_INVALID`.                           |
| **KR-6.0.3** | When matching module input/output topologies (linking), the Kernel **MUST** require exact binary equality between the Producer's declared Topic and the Consumer's Binding. |

**Module Requirements:**

| ID           | Requirement                                                                                                                  |
| :----------- | :--------------------------------------------------------------------------------------------------------------------------- |
| **MR-6.0.1** | The Module **SHOULD** ensure all URIs passed to the Kernel are pre-normalized to **IETF RFC 3986** canonical form.           |
| **MR-6.0.2** | The Module **MUST NOT** persist `KernelContextHandles`.                                                                      |
| **MR-6.0.3** | The Module **MUST NOT** pass `KernelContextHandles` to any functions that are not Kernel intrinsics as the first positional. |
| **MR-6.0.4** | WebAssembly Modules **MUST** import all Kernel Interface functions from the module namespace `filament`.                     |

---

# Part III: The Capability Model

## 1. Capability Concepts

Capabilities are identified by Uniform Resource Names (URNs). A Module cannot perform I/O unless the capability is explicitly requested in the Manifest.

### 1.1 Event Directionality

- **Outbound:** Events written _by_ the Module _to_ the Staging Area via `filament_write`.
- **Inbound:** Events written _by_ the Kernel _to_ the Staging Area, read via `filament_read`.

### 1.2 The Symbol Table

Capabilities may authorize the use of direct kernel functions resolved via the Linker.

- **Host Functions:** Symbols exported by the Kernel (e.g., `filament_channel_create`).
- **Module Functions:** Symbols exported by the Module (e.g., `filament_init`).

### 1.3 Security Enforcement

The Kernel **MUST** validate every Outbound event against the Module's granted capabilities. If a Module emits an event for a capability it does not possess, the Kernel **MUST** reject the event with `FILAMENT_ERR_PERM`.

### 1.4 Capability Affinity

Capabilities enforce constraints on the execution model of the consuming Module.

- **Agnostic:** Safe for Instance Pooling. The Module uses the capability via atomic events or transient handles.
- **Pinned:** Requires a persistent execution context. These capabilities typically involve hardware locking, expensive initialization, or DMA mappings that cannot be safely reset between Weaves.

**Kernel Requirements:**

| ID           | Requirement                                                                                                                        |
| :----------- | :--------------------------------------------------------------------------------------------------------------------------------- |
| **KR-1.4.1** | The Kernel **MUST** reject the loading of any Module that declares a **Stateless** lifecycle but requests a **Pinned** capability. |

---

## 2. Capability Catalog

### 2.1 filament.core

The foundational capability set required for the Filament lifecycle. It provides memory management, structured logging, error signaling, and panic handling. This capability is **implicitly granted** to all Modules.

**Affinity:** Agnostic

**Authorized Host Functions:**

- `filament_blob_alloc`
- `filament_blob_map`
- `filament_blob_retain`
- `filament_read`
- `filament_write`

**Events:**

| Topic                 | Direction | Payload Type          | Description                     |
| :-------------------- | :-------- | :-------------------- | :------------------------------ |
| `filament/core/log`   | Outbound  | `FilamentLogRecord`   | Emits a structured log message. |
| `filament/core/panic` | Outbound  | `FilamentPanicRecord` | Signals an unrecoverable error. |

**Kernel Requirements:**

| ID           | Requirement                                                                                                                                                                      |
| :----------- | :------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **KR-2.1.1** | Upon receiving a `filament/core/panic` event, the Kernel **MUST** mark the Process as faulted, discard the Staging Area (rollback), and transition the system to a Safe State.   |
| **KR-2.1.2** | The Kernel **SHOULD** forward events from `filament/core/log` to the configured host telemetry sink.                                                                             |
| **KR-2.1.3** | When handling `filament/core/log`, the Kernel **MUST** validate that the payload matches the `FilamentLogRecord` ABI.                                                            |
| **KR-2.1.4** | Blobs created via `filament_blob_alloc` **MUST** be destroyed at the end of the Weave unless `filament_blob_retain` is called or the Blob ID is referenced in a committed event. |

---

### 2.2 filament.process

Enables Dynamic Supervision. Allows the Module to spawn, monitor, and terminate child processes.

**Affinity:** Agnostic

**Authorized Host Functions:**

- `filament_channel_create`
- `filament_process_spawn`
- `filament_process_terminate`

**Events:**

| Topic                      | Direction | Payload Type             | Description                                      |
| :------------------------- | :-------- | :----------------------- | :----------------------------------------------- |
| `filament/process/status`  | Inbound   | `FilamentProcessStatus`  | Notifications about Child process state changes. |
| `filament/lifecycle/event` | Inbound   | `FilamentLifecycleEvent` | Commands from the Kernel/Parent to stop/reload.  |

**Kernel Requirements:**

| ID           | Requirement                                                                                                                                                                                                                                                                     |
| :----------- | :------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| **KR-2.2.1** | If a Parent Process terminates, the Kernel **MUST** immediately terminate all Child Processes spawned by that Parent (Cascading Failure).                                                                                                                                       |
| **KR-2.2.2** | The Kernel **SHOULD** emit a `filament/lifecycle/event` with `cmd=1` (Stop) to a process before invoking `filament_process_terminate`.                                                                                                                                          |
| **KR-2.2.3** | **Capability Isolation:** The Kernel **MUST** reject a `filament_process_spawn` request if the Child's Manifest requests capabilities that the Parent does not possess, unless the Kernel configuration explicitly permits privilege escalation (e.g., via a `sudo` mechanism). |

---

### 2.3 filament.time

Allows the Module to request wake-up events at specific points in Virtual Time.

**Affinity:** Agnostic

**Types:**

**Type:** `FilamentTimerSet`

**Size:** 16 bytes

| Offset | Field    | Type  | Description                 |
| :----- | :------- | :---- | :-------------------------- |
| 0      | `req_id` | `u64` | Correlation ID.             |
| 8      | `target` | `u64` | Absolute Virtual Time (ns). |

**Type:** `FilamentTimerFire`

**Size:** 24 bytes

| Offset | Field    | Type  | Description                       |
| :----- | :------- | :---- | :-------------------------------- |
| 0      | `req_id` | `u64` | Matches request ID.               |
| 8      | `skew`   | `i64` | `actual_time - target_time` (ns). |
| 16     | `_pad`   | `u64` | Reserved.                         |

**Events:**

| Topic                | Direction | Payload Type        |
| :------------------- | :-------- | :------------------ |
| `filament/time/set`  | Outbound  | `FilamentTimerSet`  |
| `filament/time/fire` | Inbound   | `FilamentTimerFire` |

**Kernel Requirements:**

| ID           | Requirement                                                                                                                                   |
| :----------- | :-------------------------------------------------------------------------------------------------------------------------------------------- |
| **KR-2.3.1** | The Kernel **MUST** populate the `skew` field in `FilamentTimerFire` with the result of `current_virtual_time - requested_target_time`.       |
| **KR-2.3.2** | The Kernel **MUST** ensure that if `target` is in the past relative to the current Virtual Time, the timer fires in the immediate next Weave. |

**Module Requirements:**

| ID           | Constraint                                             |
| :----------- | :----------------------------------------------------- |
| **MR-2.3.1** | `target` **MUST** be a valid 64-bit integer timestamp. |

---

### 2.4 filament.env

Provides read-only access to the Host environment variables.

**Affinity:** Agnostic

**Types:**

**Type:** `FilamentEnvRead`

**Size:** 24 bytes

| Offset | Field    | Type             | Description                             |
| :----- | :------- | :--------------- | :-------------------------------------- |
| 0      | `req_id` | `u64`            | Correlation ID.                         |
| 8      | `key`    | `FilamentString` | The environment variable key to lookup. |

**Type:** `FilamentEnvVal`

**Size:** 32 bytes

| Offset | Field    | Type             | Description                          |
| :----- | :------- | :--------------- | :----------------------------------- |
| 0      | `req_id` | `u64`            | Matches the request ID.              |
| 8      | `val`    | `FilamentString` | The value (empty string if missing). |
| 24     | `_pad`   | `u64`            | Reserved.                            |

**Events:**

| Topic               | Direction | Payload Type      |
| :------------------ | :-------- | :---------------- |
| `filament/env/read` | Outbound  | `FilamentEnvRead` |
| `filament/env/val`  | Inbound   | `FilamentEnvVal`  |

**Kernel Requirements:**

| ID           | Requirement                                                                                                        |
| :----------- | :----------------------------------------------------------------------------------------------------------------- |
| **KR-2.4.1** | The Kernel **MUST NOT** expose variables not explicitly allowlisted in the Process Manifest or Host Configuration. |

---

### 2.5 filament.fs

Provides restricted access to the host filesystem.

**Affinity:** Agnostic

**Types:**

**Type:** `FilamentFsRead`

**Size:** 40 bytes

| Offset | Field    | Type             | Description      |
| :----- | :------- | :--------------- | :--------------- |
| 0      | `req_id` | `u64`            | Correlation ID.  |
| 8      | `path`   | `FilamentString` | Virtual path.    |
| 24     | `offset` | `u64`            | Byte offset.     |
| 32     | `len`    | `u64`            | Number of bytes. |

**Type:** `FilamentFsWrite`

**Size:** 48 bytes

| Offset | Field    | Type             | Description                        |
| :----- | :------- | :--------------- | :--------------------------------- |
| 0      | `req_id` | `u64`            | Correlation ID.                    |
| 8      | `path`   | `FilamentString` | Virtual path.                      |
| 24     | `offset` | `u64`            | Byte offset.                       |
| 32     | `data`   | `FilamentBlob`   | Blob reference containing content. |

**Type:** `FilamentFsList`

**Size:** 48 bytes

| Offset | Field    | Type             | Description       |
| :----- | :------- | :--------------- | :---------------- |
| 0      | `req_id` | `u64`            | Correlation ID.   |
| 8      | `path`   | `FilamentString` | Virtual path.     |
| 24     | `cursor` | `u64`            | Pagination token. |
| 32     | `limit`  | `u32`            | Max entries.      |
| 36     | `_pad`   | `u32`            | Reserved.         |

**Type:** `FilamentFsContent`

**Size:** 48 bytes

| Offset | Field    | Type            | Description                                  |
| :----- | :------- | :-------------- | :------------------------------------------- |
| 0      | `req_id` | `u64`           | Matches request ID.                          |
| 8      | `status` | `i64`           | 0=OK, -2=NotFound, -1=Permission.            |
| 16     | `data`   | `FilamentValue` | `FILAMENT_VAL_BLOB` or `FILAMENT_VAL_BYTES`. |

**Type:** `FilamentDirectoryEntry`

**Size:** 32 bytes

| Offset | Field  | Type             | Description                     |
| :----- | :----- | :--------------- | :------------------------------ |
| 0      | `name` | `FilamentString` | Filename (relative).            |
| 16     | `size` | `u64`            | Size in bytes.                  |
| 24     | `kind` | `u8`             | 0=File, 1=Directory, 2=Symlink. |
| 25     | `_pad` | `u8[7]`          | Reserved.                       |

**Type:** `FilamentFsEntries`

**Size:** 32 bytes

| Offset | Field     | Type            | Description                            |
| :----- | :-------- | :-------------- | :------------------------------------- |
| 0      | `req_id`  | `u64`           | Matches request ID.                    |
| 8      | `status`  | `i64`           | 0=OK, error code otherwise.            |
| 16     | `entries` | `FilamentArray` | Pointer to `FilamentDirectoryEntry[]`. |

**Type:** `FilamentFsAck`

**Size:** 24 bytes

| Offset | Field     | Type  | Description                 |
| :----- | :-------- | :---- | :-------------------------- |
| 0      | `req_id`  | `u64` | Matches request ID.         |
| 8      | `status`  | `i64` | 0=OK, error code otherwise. |
| 16     | `written` | `u64` | Total bytes written.        |

**Events:**

| Topic                 | Direction | Payload Type        |
| :-------------------- | :-------- | :------------------ |
| `filament/fs/read`    | Outbound  | `FilamentFsRead`    |
| `filament/fs/write`   | Outbound  | `FilamentFsWrite`   |
| `filament/fs/list`    | Outbound  | `FilamentFsList`    |
| `filament/fs/content` | Inbound   | `FilamentFsContent` |
| `filament/fs/entries` | Inbound   | `FilamentFsEntries` |
| `filament/fs/ack`     | Inbound   | `FilamentFsAck`     |

**Kernel Requirements:**

| ID           | Requirement                                                                                                         |
| :----------- | :------------------------------------------------------------------------------------------------------------------ |
| **KR-2.3.1** | The Kernel **MUST** validate that `path` is within the allowed directory whitelist defined by the Process Manifest. |
| **KR-2.3.2** | If the file size exceeds `FILAMENT_MIN_BLOB_BYTES`, the Kernel **MUST** return the content as a `blob`.             |
| **KR-2.3.3** | Directory listings **MUST** be sorted by the lexicographical order of their NFC-normalized UTF-8 filenames.         |

**Module Requirements:**

| ID           | Constraint                                                                                                                       |
| :----------- | :------------------------------------------------------------------------------------------------------------------------------- |
| **MR-2.3.1** | In `filament/fs/write`, the `data` field **MUST** reference a valid Blob ID owned by the Module.                                 |
| **MR-2.3.2** | `path` **MUST** be a valid UTF-8 string and **MUST NOT** contain relative parent traversals (`..`) that escape the virtual root. |

---

### 2.6 filament.kv

Provides a persistent Key-Value store.

**Affinity:** Agnostic

**Types:**

**Type:** `FilamentKvSet`

**Size:** 48 bytes

| Offset | Field | Type             | Description    |
| :----- | :---- | :--------------- | :------------- |
| 0      | `key` | `FilamentString` | Lookup Key.    |
| 16     | `val` | `FilamentValue`  | Value payload. |

**Type:** `FilamentKvGet`

**Size:** 16 bytes

| Offset | Field | Type             | Description |
| :----- | :---- | :--------------- | :---------- |
| 0      | `key` | `FilamentString` | Lookup Key. |

**Type:** `FilamentKvResult`

**Size:** 64 bytes

| Offset | Field    | Type             | Description           |
| :----- | :------- | :--------------- | :-------------------- |
| 0      | `key`    | `FilamentString` | Lookup Key.           |
| 16     | `val`    | `FilamentValue`  | Value payload.        |
| 48     | `status` | `i64`            | 0=Found, -2=NotFound. |
| 56     | `_pad`   | `u64`            | Reserved.             |

**Events:**

| Topic                | Direction | Payload Type       |
| :------------------- | :-------- | :----------------- |
| `filament/kv/set`    | Outbound  | `FilamentKvSet`    |
| `filament/kv/get`    | Outbound  | `FilamentKvGet`    |
| `filament/kv/result` | Inbound   | `FilamentKvResult` |

**Kernel Requirements:**

| ID           | Requirement                                                                                                                                     |
| :----------- | :---------------------------------------------------------------------------------------------------------------------------------------------- |
| **KR-2.6.1** | Reads (`get`) **MUST** return the value as it existed at the start of the Weave (Snapshot Isolation).                                           |
| **KR-2.6.2** | Writes (`set`) are buffered and applied atomically at the end of the Weave.                                                                     |
| **KR-2.6.3** | If a Module emits multiple `set` events for the same `key` in a single Weave, the Kernel **MUST** apply the last one emitted (Last-Write-Wins). |

---

### 2.7 filament.net.http

Provides an egress gateway for HTTP requests.

**Affinity:** Agnostic

**Types:**

**Type:** `FilamentHttpRequest`

**Size:** 88 bytes

| Offset | Field     | Type             | Description              |
| :----- | :-------- | :--------------- | :----------------------- |
| 0      | `req_id`  | `u64`            | Correlation ID.          |
| 8      | `method`  | `FilamentString` | HTTP Method (GET, POST). |
| 24     | `url`     | `FilamentString` | Fully qualified URL.     |
| 40     | `headers` | `FilamentArray`  | Map of headers.          |
| 56     | `body`    | `FilamentValue`  | Payload (Blob or Bytes). |

**Type:** `FilamentHttpResponse`

**Size:** 72 bytes

| Offset | Field     | Type            | Description         |
| :----- | :-------- | :-------------- | :------------------ |
| 0      | `req_id`  | `u64`           | Matches request ID. |
| 8      | `status`  | `u32`           | HTTP Status Code.   |
| 12     | `_pad`    | `u32`           | Reserved.           |
| 16     | `headers` | `FilamentArray` | Map of headers.     |
| 32     | `body`    | `FilamentValue` | Payload.            |
| 64     | `latency` | `u64`           | Network time (ns).  |

**Events:**

| Topic                   | Direction | Payload Type           |
| :---------------------- | :-------- | :--------------------- |
| `filament/net/http/req` | Outbound  | `FilamentHttpRequest`  |
| `filament/net/http/res` | Inbound   | `FilamentHttpResponse` |

**Kernel Requirements:**

| ID           | Requirement                                                                                                                                                                                                                          |
| :----------- | :----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **KR-2.7.1** | The Kernel **MUST** perform the network request asynchronously.                                                                                                                                                                      |
| **KR-2.7.2** | The Kernel **SHOULD** use Blob References for response bodies to avoid copying large payloads into the Staging Area.                                                                                                                 |
| **KR-2.7.3** | The Kernel **MUST** validate the URL schema. Requests to non-standard schemes (e.g., `file://`, `gopher://`) or localhost (unless explicitly allowed via config) **MUST** be rejected to prevent Server-Side Request Forgery (SSRF). |

---

### 2.8 filament.llm.tool

Standardizes external tool invocation schemas.

**Affinity:** Agnostic

**Types:**

**Type:** `FilamentToolInvoke`

**Size:** 56 bytes

| Offset | Field     | Type              | Description             |
| :----- | :-------- | :---------------- | :---------------------- |
| 0      | `req_id`  | `u64`             | Correlation ID.         |
| 8      | `tool`    | `FilamentString`  | Tool name.              |
| 24     | `args`    | `FilamentArray`   | Arguments Map.          |
| 40     | `context` | `FilamentAddress` | Optional Trace Context. |
| 48     | `_pad`    | `u64`             | Reserved.               |

**Type:** `FilamentToolResult`

**Size:** 64 bytes

| Offset | Field    | Type             | Description         |
| :----- | :------- | :--------------- | :------------------ |
| 0      | `req_id` | `u64`            | Matches request ID. |
| 8      | `tool`   | `FilamentString` | Tool name.          |
| 24     | `result` | `FilamentArray`  | Output data Map.    |
| 40     | `status` | `i64`            | 0=Success, 1=Error. |
| 48     | `_pad`   | `u8[16]`         | Reserved.           |

**Events:**

| Topic                      | Direction | Payload Type         |
| :------------------------- | :-------- | :------------------- |
| `filament/llm/tool/invoke` | Outbound  | `FilamentToolInvoke` |
| `filament/llm/tool/result` | Inbound   | `FilamentToolResult` |

---

### 2.9 filament.hw.buffer

Provides transactional, buffered access to memory-mapped hardware registers.

**Affinity:** Pinned

**Authorized Host Functions:**

- `filament_hw_buffer_write`

**Kernel Requirements:**

| ID           | Requirement                                                                       |
| :----------- | :-------------------------------------------------------------------------------- |
| **KR-2.9.1** | Writes **MUST** be flushed to physical hardware only during the **Commit Phase**. |
| **KR-2.9.2** | If the Weave fails, the Kernel **MUST** discard the buffer.                       |

---

### 2.10 filament.hw.irq

Allows a Unmanaged Module to subscribe to hardware interrupts.

**Affinity:** Pinned

**Types:**

**Type:** `FilamentIrqEvent`

**Size:** 16 bytes

| Offset | Field       | Type  | Description                 |
| :----- | :---------- | :---- | :-------------------------- |
| 0      | `irq_line`  | `u32` | Hardware Interrupt ID.      |
| 4      | `_pad`      | `u32` | Reserved.                   |
| 8      | `timestamp` | `u64` | Hardware capture time (ns). |

**Events:**

| Topic                  | Direction | Payload Type       |
| :--------------------- | :-------- | :----------------- |
| `filament/hw/irq/fire` | Inbound   | `FilamentIrqEvent` |

---

### 2.11 filament.hw.map

Grants access to map specific physical memory regions (MMIO).

**Affinity:** Pinned

**Authorized Host Functions:**

- Authorizes `filament_blob_map` for specific Resource Keys.

**Kernel Requirements:**

| ID            | Requirement                                                                                                       |
| :------------ | :---------------------------------------------------------------------------------------------------------------- |
| **KR-2.11.1** | The Kernel **MUST** verify the Resource Key requested corresponds to a valid `[resources]` entry in the Manifest. |

---

### 2.12 filament.hw.serial

Provides bidirectional stream access to UART, SPI, or I2C interfaces.

**Affinity:** Pinned

**Types:**

**Type:** `FilamentSerialPacket`

**Size:** 56 bytes

| Offset | Field  | Type             | Description              |
| :----- | :----- | :--------------- | :----------------------- |
| 0      | `port` | `FilamentString` | Device identifier.       |
| 16     | `data` | `FilamentValue`  | Payload (Blob or Bytes). |
| 48     | `_pad` | `u64`            | Reserved.                |

**Events:**

| Topic                      | Direction | Payload Type           |
| :------------------------- | :-------- | :--------------------- |
| `filament/hw/serial/write` | Outbound  | `FilamentSerialPacket` |
| `filament/hw/serial/read`  | Inbound   | `FilamentSerialPacket` |

---

# Appendices

## Appendix A: Glossary [Informative]

This glossary defines the standard nomenclature used throughout the Filament Specification.

| Term             | Definition                                                                                                                                                                              |
| :--------------- | :-------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **Blob**         | A reference-counted, Kernel-managed memory buffer used for zero-copy I/O. Blobs allow Modules to exchange large assets (e.g., video frames) without copying them into the Staging Area. |
| **Capability**   | A strictly typed permission token, identified by a URN (e.g., `filament.net.http`), that grants a Module access to specific Kernel resources or I/O operations.                         |
| **Channel:**     | A persistent, typed ring-buffer used for asynchronous communication between processes.                                                                                                  |
| **Context**      | The execution environment that defines the privileges, loader requirements, and preemption constraints of a Module.                                                                     |
| **Kernel**       | The runtime authority responsible for scheduling, safety enforcement, and resource management. It is the only entity with direct access to the host hardware or OS.                     |
| **Manifest**     | A declarative configuration file that defines the Process topology, resource constraints, and capability requirements.                                                                  |
| **Module**       | The executable binary artifact (Wasm or Native) that implements the Filament Interface. Modules are the atomic units of deployment.                                                     |
| **Pipeline**     | An ordered chain of Modules configured to execute sequentially within a single Process.                                                                                                 |
| **Process**      | The unit of isolation consisting of a Cryptographic Identity, an Immutable Timeline, and an Executable Pipeline.                                                                        |
| **Schema**       | A formal definition of the data structure expected on a specific Event Topic. The Kernel uses Schemas to enforce link-time type safety between Modules.                                 |
| **Staging Area** | The transient, zero-persistence memory interface used to pass data between Modules during a Weave.                                                                                      |
| **Tick**         | A strictly monotonically increasing 64-bit integer representing the logical step index of the Timeline. Unlike Virtual Time, Ticks are unique and provide a total ordering of events.   |
| **Timeline**     | The immutable, strictly ordered log of all committed events representing the Process history.                                                                                           |
| **Virtual Time** | A logical clock injected by the Kernel to ensure deterministic execution. It allows the execution speed to be decoupled from the host OS wall clock.                                    |
| **Weave:**       | The atomic, transactional execution cycle where the Kernel drives inputs through the Pipeline and commits results to the Timeline.                                                      |

## Appendix B: Conformance

A Kernel is considered **Compliant** if it passes the **Kernel Compliance Kit** (KCK). The KCK validates host behavior against the normative requirements defined in this document through the following test suites:

1.  **ABI Layout Verification:** Validates that the Kernel correctly parses binary structures and enforces alignment/padding rules.
2.  **Lifecycle Fuzzing:** Validates that the Kernel correctly handles state transitions (Init -> Weave -> Error) and cleans up resources after a crash.
3.  **Capability Enforcement:** Validates that the Kernel rejects unauthorized I/O attempts and enforces Access Control Lists.
4.  **Determinism Checks:** Validates that Logic Modules produce bitwise-identical outputs given identical inputs and seeds, regardless of wall-clock time.
5.  **Resource Governance:** Validates that the Kernel correctly interrupts Modules that exceed their time or memory budgets.
6.  **Concurrency Safety:** Validates that the Kernel correctly handles Ring Buffer overflows, Wake-on-Close behavior, and memory isolation between Shared and Dedicated processes.
