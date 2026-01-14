// --------------------------------------------------------------------------
// AUTO-GENERATED FILE. DO NOT EDIT.
// Source: spec/strands.md
// --------------------------------------------------------------------------

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
