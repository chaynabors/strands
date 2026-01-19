// --------------------------------------------------------------------------
// AUTO-GENERATED FILE. DO NOT EDIT.
// Source: spec/filament-0.1.0.md
// --------------------------------------------------------------------------

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
