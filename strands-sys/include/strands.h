// --------------------------------------------------------------------------
// AUTO-GENERATED FILE. DO NOT EDIT.
// Source: spec/strands.md
// --------------------------------------------------------------------------

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
