#![no_std]

pub mod sys;

pub type FilamentResult<T> = core::result::Result<T, FilamentError>;

pub enum FilamentError {
    TryFrom {
        from: &'static str,
        into: &'static str,
    },
    System(SystemError),
}

#[repr(i64)]
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SystemError {
    Unknown = sys::FILAMENT_ERR_UNKNOWN,
    Perm = sys::FILAMENT_ERR_PERM,
    NotFound = sys::FILAMENT_ERR_NOT_FOUND,
    Io = sys::FILAMENT_ERR_IO,
    Oom = sys::FILAMENT_ERR_OOM,
    Invalid = sys::FILAMENT_ERR_INVALID,
    Timeout = sys::FILAMENT_ERR_TIMEOUT,
    Type = sys::FILAMENT_ERR_TYPE,
}

impl TryFrom<i64> for SystemError {
    type Error = FilamentError;

    fn try_from(code: i64) -> FilamentResult<Self> {
        match code {
            sys::FILAMENT_ERR_UNKNOWN => Ok(SystemError::Unknown),
            sys::FILAMENT_ERR_PERM => Ok(SystemError::Perm),
            sys::FILAMENT_ERR_NOT_FOUND => Ok(SystemError::NotFound),
            sys::FILAMENT_ERR_IO => Ok(SystemError::Io),
            sys::FILAMENT_ERR_OOM => Ok(SystemError::Oom),
            sys::FILAMENT_ERR_INVALID => Ok(SystemError::Invalid),
            sys::FILAMENT_ERR_TIMEOUT => Ok(SystemError::Timeout),
            sys::FILAMENT_ERR_TYPE => Ok(SystemError::Type),
            _ => Err(FilamentError::TryFrom {
                from: "i64",
                into: "SystemError",
            }),
        }
    }
}

pub struct FilamentStr<'a> {
    inner: sys::FilamentString,
    _marker: core::marker::PhantomData<&'a u8>,
}

pub struct FilamentBlob<'a> {
    inner: sys::FilamentBlob,
    _marker: core::marker::PhantomData<&'a u8>,
}

pub struct FilamentMap<'a> {
    inner: sys::FilamentArray,
    _marker: core::marker::PhantomData<&'a u8>,
}

pub struct FilamentList<'a> {
    inner: sys::FilamentArray,
    _marker: core::marker::PhantomData<&'a u8>,
}

pub struct FilamentBytes<'a> {
    inner: sys::FilamentArray,
    _marker: core::marker::PhantomData<&'a u8>,
}

pub enum FilamentValue<'a> {
    Unit,
    Bool(bool),
    I64(i64),
    U64(u64),
    F64(f64),
    Str(FilamentStr<'a>),
    Blob(FilamentBlob<'a>),
    Map(FilamentMap<'a>),
    List(FilamentList<'a>),
    Bytes(FilamentBytes<'a>),
}
