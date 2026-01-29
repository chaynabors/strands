pub const FILAMENT_NULL: u64 = 0;
pub const FILAMENT_MAGIC: u32 = 0x9D2F8A41;

pub const FILAMENT_MAX_RECURSION_DEPTH: u32 = 64;
pub const FILAMENT_MAX_URI_LEN: u32 = 2048;
pub const FILAMENT_MIN_BLOB_BYTES: u64 = 128;
pub const FILAMENT_MIN_BUS_BYTES: u64 = 65536;

pub const FILAMENT_PARK: i64 = 0;
pub const FILAMENT_YIELD: i64 = 1;

pub const FILAMENT_ERR_UNKNOWN: i64 = -1;
pub const FILAMENT_ERR_PERM: i64 = -2;
pub const FILAMENT_ERR_NOT_FOUND: i64 = -3;
pub const FILAMENT_ERR_IO: i64 = -4;
pub const FILAMENT_ERR_OOM: i64 = -5;
pub const FILAMENT_ERR_INVALID: i64 = -6;
pub const FILAMENT_ERR_TIMEOUT: i64 = -7;
pub const FILAMENT_ERR_TYPE: i64 = -8;

pub const FILAMENT_VAL_UNIT: u32 = 0;
pub const FILAMENT_VAL_BOOL: u32 = 1;
pub const FILAMENT_VAL_I64: u32 = 2;
pub const FILAMENT_VAL_U64: u32 = 3;
pub const FILAMENT_VAL_F64: u32 = 4;
pub const FILAMENT_VAL_STR: u32 = 5;
pub const FILAMENT_VAL_BLOB: u32 = 6;
pub const FILAMENT_VAL_MAP: u32 = 7;
pub const FILAMENT_VAL_LIST: u32 = 8;
pub const FILAMENT_VAL_BYTES: u32 = 9;

pub const FILAMENT_IO_RAW: u32 = 1 << 0;
pub const FILAMENT_IO_VAL: u32 = 1 << 1;
pub const FILAMENT_IO_DMA: u32 = 1 << 2;
pub const FILAMENT_IO_DMA_OPTIONAL: u32 = 1 << 3;

pub const FILAMENT_FMT_BINARY: u32 = 0;
pub const FILAMENT_FMT_JSON: u32 = 1;
pub const FILAMENT_FMT_PROTO: u32 = 2;
pub const FILAMENT_FMT_TEXT: u32 = 3;

pub const FILAMENT_SCHED_SHARED: u8 = 0;
pub const FILAMENT_SCHED_DEDICATED: u8 = 1;

pub const FILAMENT_CONTEXT_LOGIC: u8 = 0;
pub const FILAMENT_CONTEXT_MANAGED: u8 = 1;
pub const FILAMENT_CONTEXT_UNMANAGED: u8 = 2;

pub const FILAMENT_WAKE_INIT: u32 = 1 << 0;
pub const FILAMENT_WAKE_IO: u32 = 1 << 1;
pub const FILAMENT_WAKE_TIMER: u32 = 1 << 2;
pub const FILAMENT_WAKE_YIELD: u32 = 1 << 3;
pub const FILAMENT_WAKE_LIFECYCLE: u32 = 1 << 4;

pub const FILAMENT_MMAP_READ: u32 = 1 << 0;
pub const FILAMENT_MMAP_WRITE: u32 = 1 << 1;
pub const FILAMENT_MMAP_EXEC: u32 = 1 << 2;

macro_rules! static_assert_layout {
    ($ty:ty, size: $size:expr, align: $align:expr) => {
        const _: () = {
            assert!(
                core::mem::size_of::<$ty>() == $size,
                concat!("Size mismatch for ", stringify!($ty))
            );
            assert!(
                core::mem::align_of::<$ty>() == $align,
                concat!("Alignment mismatch for ", stringify!($ty))
            );
        };
    };
}

#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
pub struct FilamentAddress(u64);

#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
pub struct FilamentContextHandle(u64);

#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
pub struct FilamentBlobHandle(u64);

#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
pub struct FilamentCursorHandle(u64);

#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
pub struct FilamentProcessHandle(u64);

#[repr(C, align(8))]
#[derive(Debug, Clone, Copy)]
pub struct FilamentString {
    pub ptr: FilamentAddress,
    pub len: u64,
}
static_assert_layout!(FilamentString, size: 16, align: 8);

#[repr(C, align(8))]
#[derive(Debug, Clone, Copy)]
pub struct FilamentBlob {
    pub handle: FilamentBlobHandle,
    pub ptr: FilamentAddress,
    pub size: u64,
}
static_assert_layout!(FilamentBlob, size: 24, align: 8);

#[repr(C, align(8))]
#[derive(Debug, Clone, Copy)]
pub struct FilamentArray {
    pub ptr: FilamentAddress,
    pub len: u64,
}
static_assert_layout!(FilamentArray, size: 16, align: 8);

#[repr(C, align(8))]
#[derive(Debug)]
pub struct FilamentPair {
    pub key: FilamentString,
    pub value: FilamentValue,
}
static_assert_layout!(FilamentPair, size: 48, align: 8);

#[repr(C, align(8))]
pub struct FilamentValue {
    pub tag: u32,
    pub flags: u32,
    pub data: FilamentValueData,
}
static_assert_layout!(FilamentValue, size: 32, align: 8);

impl core::fmt::Debug for FilamentValue {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self.tag {
            FILAMENT_VAL_UNIT => write!(f, "Unit"),
            FILAMENT_VAL_BOOL => write!(f, "Bool"),
            FILAMENT_VAL_I64 => write!(f, "I64"),
            FILAMENT_VAL_U64 => write!(f, "U64"),
            FILAMENT_VAL_F64 => write!(f, "F64"),
            FILAMENT_VAL_STR => write!(f, "Str"),
            FILAMENT_VAL_BLOB => write!(f, "Blob"),
            FILAMENT_VAL_MAP => write!(f, "Map"),
            FILAMENT_VAL_LIST => write!(f, "List"),
            FILAMENT_VAL_BYTES => write!(f, "Bytes"),
            _ => write!(f, "UnknownValue(tag={})", self.tag),
        }
    }
}

#[repr(C, align(8))]
pub union FilamentValueData {
    pub as_u64: u64,
    pub as_i64: i64,
    pub as_f64: f64,
    pub as_bool: u8,
    pub as_str: FilamentString,
    pub as_blob: FilamentBlob,
    pub as_map: FilamentArray,
    pub as_list: FilamentArray,
    pub as_bytes: FilamentArray,
}
static_assert_layout!(FilamentValueData, size: 24, align: 8);

#[repr(C, align(8))]
#[derive(Debug)]
pub struct FilamentTraceContext {
    pub trace_id_hi: u64,
    pub trace_id_lo: u64,
    pub span_id: u64,
    pub flags: u8,
    pub _pad: [u8; 7],
}
static_assert_layout!(FilamentTraceContext, size: 32, align: 8);

#[repr(C, align(8))]
#[derive(Debug)]
pub struct FilamentEventHeader {
    pub total_len: u32,
    pub flags: u32,
    pub id: u64,
    pub timestamp: u64,
    pub schema_id: u64,
    pub auth_agent: u64,
    pub auth_user: u64,
    pub trace: FilamentTraceContext,
    pub topic_len: u32,
    pub data_len: u32,
    pub encoding: u32,
    pub _pad: [u8; 36],
}
static_assert_layout!(FilamentEventHeader, size: 128, align: 8);

#[repr(C, align(8))]
#[derive(Debug)]
pub struct FilamentResourceLimits {
    pub mem_max: u64,
    pub time_limit: u64,
    pub priority: u8,
    pub policy: u8,
    pub _pad: [u8; 6],
}
static_assert_layout!(FilamentResourceLimits, size: 24, align: 8);

#[repr(C, align(8))]
#[derive(Debug)]
pub struct FilamentHostInfo {
    pub limits: FilamentResourceLimits,
    pub bus_size: u64,
    pub formats: u32,
    pub cores: u32,
    pub _pad: [u8; 8],
}
static_assert_layout!(FilamentHostInfo, size: 48, align: 8);

#[repr(C, align(8))]
#[derive(Debug)]
pub struct FilamentModuleInfo {
    pub magic: u32,
    pub abi_ver: u32,
    pub mod_type: u32,
    pub _pad: u32,
    pub mem_req: u64,
    pub name: FilamentString,
    pub version: FilamentString,
}
static_assert_layout!(FilamentModuleInfo, size: 56, align: 8);

#[repr(C, align(8))]
#[derive(Debug)]
pub struct FilamentConfig {
    pub count: u64,
    pub entries: FilamentAddress,
}
static_assert_layout!(FilamentConfig, size: 16, align: 8);

#[repr(C, align(8))]
#[derive(Debug)]
pub struct FilamentChannelDefinition {
    pub schema: FilamentString,
    pub capacity: u64,
    pub msg_size: u64,
    pub direction: u32,
    pub root_type: u32,
}
static_assert_layout!(FilamentChannelDefinition, size: 40, align: 8);

#[repr(C, align(8))]
#[derive(Debug)]
pub struct FilamentModuleDefinition {
    pub alias: FilamentString,
    pub source: FilamentString,
    pub digest: FilamentString,
    pub config: FilamentAddress,
    pub context: u32,
    pub _pad: u32,
}
static_assert_layout!(FilamentModuleDefinition, size: 64, align: 8);

#[repr(C, align(8))]
#[derive(Debug)]
pub struct FilamentProcessStatus {
    pub handle: FilamentProcessHandle,
    pub code: i64,
    pub state: u32,
    pub _pad: u32,
}
static_assert_layout!(FilamentProcessStatus, size: 24, align: 8);

#[repr(C, align(8))]
#[derive(Debug)]
pub struct FilamentProcessLifecycleEvent {
    pub timeout: u64,
    pub cmd: u32,
    pub _pad: u32,
}
static_assert_layout!(FilamentProcessLifecycleEvent, size: 16, align: 8);

#[repr(C, align(8))]
#[derive(Debug)]
pub struct FilamentReadArgs {
    pub topic: FilamentString,
    pub start: u64,
    pub out_ptr: FilamentAddress,
    pub out_cap: u64,
}
static_assert_layout!(FilamentReadArgs, size: 40, align: 8);

#[repr(C, align(8))]
#[derive(Debug)]
pub struct FilamentWriteArgs {
    pub topic: FilamentString,
    pub data: FilamentAddress,
    pub len: u64,
    pub flags: u32,
    pub _pad: u32,
}
static_assert_layout!(FilamentWriteArgs, size: 40, align: 8);

#[repr(C, align(8))]
#[derive(Debug)]
pub struct FilamentBlobAllocArgs {
    pub out_ref: FilamentAddress,
    pub size: u64,
    pub flags: u32,
    pub _pad: u32,
}
static_assert_layout!(FilamentBlobAllocArgs, size: 24, align: 8);

#[repr(C, align(8))]
#[derive(Debug)]
pub struct FilamentBlobMapArgs {
    pub out_ref: FilamentAddress,
    pub handle: FilamentBlobHandle,
    pub flags: u32,
    pub _pad: u32,
}
static_assert_layout!(FilamentBlobMapArgs, size: 24, align: 8);

#[repr(C, align(8))]
#[derive(Debug)]
pub struct FilamentBlobRetainArgs {
    pub handle: FilamentBlobHandle,
}
static_assert_layout!(FilamentBlobRetainArgs, size: 8, align: 8);

#[repr(C, align(8))]
#[derive(Debug)]
pub struct FilamentChannelCreateArgs {
    pub def: FilamentChannelDefinition,
    pub out_ptr: FilamentAddress,
    pub out_cap: u64,
}
static_assert_layout!(FilamentChannelCreateArgs, size: 56, align: 8);

#[repr(C, align(8))]
#[derive(Debug)]
pub struct FilamentProcessSpawnArgs {
    pub modules: FilamentArray,
    pub bindings: FilamentArray,
    pub limits: FilamentResourceLimits,
    pub _pad: [u8; 8],
}
static_assert_layout!(FilamentProcessSpawnArgs, size: 64, align: 8);

#[repr(C, align(8))]
#[derive(Debug)]
pub struct FilamentProcessTerminateArgs {
    pub handle: FilamentProcessHandle,
}
static_assert_layout!(FilamentProcessTerminateArgs, size: 8, align: 8);

#[repr(C, align(8))]
#[derive(Debug)]
pub struct FilamentTimelineOpenArgs {
    pub topic: FilamentString,
    pub start: u64,
    pub end: u64,
    pub limit: u64,
    pub desc: u8,
    pub _pad: [u8; 7],
}
static_assert_layout!(FilamentTimelineOpenArgs, size: 48, align: 8);

#[repr(C, align(8))]
#[derive(Debug)]
pub struct FilamentTimelineNextArgs {
    pub handle: FilamentCursorHandle,
    pub out_ptr: FilamentAddress,
    pub buf_cap: u64,
}
static_assert_layout!(FilamentTimelineNextArgs, size: 24, align: 8);

#[repr(C, align(8))]
#[derive(Debug)]
pub struct FilamentTimelineCloseArgs {
    pub handle: FilamentCursorHandle,
}
static_assert_layout!(FilamentTimelineCloseArgs, size: 8, align: 8);

#[repr(C, align(8))]
#[derive(Debug)]
pub struct FilamentInitArgs {
    pub host: FilamentAddress,
    pub config: FilamentAddress,
    pub _pad: [u8; 16],
}
static_assert_layout!(FilamentInitArgs, size: 32, align: 8);

#[repr(C, align(8))]
#[derive(Debug)]
pub struct FilamentWeaveArgs {
    pub time_limit: u64,
    pub res_used: u64,
    pub res_max: u64,
    pub mem_max: u64,
    pub rand_seed: u64,
    pub virt_time: u64,
    pub trace: FilamentTraceContext,
    pub delta_ns: u64,
    pub tick: u64,
    pub wake_flags: u32,
    pub _pad: u32,
    pub user_data: u64,
    pub _pad2: [u8; 16],
}
static_assert_layout!(FilamentWeaveArgs, size: 128, align: 8);

#[repr(C, align(8))]
#[derive(Debug)]
pub struct FilamentLogRecord {
    pub level: u32,
    pub _pad: u32,
    pub msg: FilamentString,
    pub context: FilamentAddress,
}
static_assert_layout!(FilamentLogRecord, size: 32, align: 8);

#[repr(C, align(8))]
#[derive(Debug)]
pub struct FilamentPanicRecord {
    pub code: i64,
    pub reason: FilamentString,
}
static_assert_layout!(FilamentPanicRecord, size: 24, align: 8);

// Kernel prototype definitions
pub type FilamentReadFn =
    unsafe extern "C" fn(ctx: FilamentContextHandle, args: *const FilamentReadArgs) -> i64;
pub type FilamentWriteFn =
    unsafe extern "C" fn(ctx: FilamentContextHandle, args: *const FilamentWriteArgs) -> i64;
pub type FilamentBlobAllocFn =
    unsafe extern "C" fn(ctx: FilamentContextHandle, args: *const FilamentBlobAllocArgs) -> i64;
pub type FilamentBlobMapFn =
    unsafe extern "C" fn(ctx: FilamentContextHandle, args: *const FilamentBlobMapArgs) -> i64;
pub type FilamentBlobRetainFn =
    unsafe extern "C" fn(ctx: FilamentContextHandle, args: *const FilamentBlobRetainArgs) -> i64;
pub type FilamentTimelineOpenFn =
    unsafe extern "C" fn(ctx: FilamentContextHandle, args: *const FilamentTimelineOpenArgs) -> i64;
pub type FilamentTimelineNextFn =
    unsafe extern "C" fn(ctx: FilamentContextHandle, args: *const FilamentTimelineNextArgs) -> i64;
pub type FilamentTimelineCloseFn =
    unsafe extern "C" fn(ctx: FilamentContextHandle, args: *const FilamentTimelineCloseArgs) -> i64;
pub type FilamentChannelCreateFn =
    unsafe extern "C" fn(ctx: FilamentContextHandle, args: *const FilamentChannelCreateArgs) -> i64;
pub type FilamentProcessSpawnFn =
    unsafe extern "C" fn(ctx: FilamentContextHandle, args: *const FilamentProcessSpawnArgs) -> i64;
pub type FilamentProcessTerminateFn = unsafe extern "C" fn(
    ctx: FilamentContextHandle,
    args: *const FilamentProcessTerminateArgs,
) -> i64;

// Module prototype definitions
pub type FilamentGetInfoFn = unsafe extern "C" fn(kernel_version: u32, capabilities: u64) -> u64;
pub type FilamentReserveFn =
    unsafe extern "C" fn(size: u64, alignment: u64, flags: u32) -> FilamentAddress;
pub type FilamentInitFn =
    unsafe extern "C" fn(ctx: FilamentContextHandle, args: *const FilamentInitArgs) -> i64;
pub type FilamentWeaveFn =
    unsafe extern "C" fn(ctx: FilamentContextHandle, args: *const FilamentWeaveArgs) -> i64;
