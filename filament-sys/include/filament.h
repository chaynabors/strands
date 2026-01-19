// --------------------------------------------------------------------------
// AUTO-GENERATED FILE. DO NOT EDIT.
// Source: spec/filament-0.1.0.md
// --------------------------------------------------------------------------

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
