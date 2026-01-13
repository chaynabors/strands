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