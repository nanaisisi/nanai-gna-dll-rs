use std::ffi::c_char;
use std::ffi::c_void;

use crate::error::{GnaError, Result};
use crate::types::{Gna2AccelerationMode, Gna2DeviceVersion, Gna2InstrumentationPoint, Gna2Status};

// Raw C function signatures
pub type FnGna2DeviceGetCount = unsafe extern "C" fn(device_count: *mut u32) -> Gna2Status;
pub type FnGna2DeviceGetVersion =
    unsafe extern "C" fn(device_index: u32, device_version: *mut Gna2DeviceVersion) -> Gna2Status;
pub type FnGna2DeviceOpen = unsafe extern "C" fn(device_index: u32) -> Gna2Status;
pub type FnGna2DeviceClose = unsafe extern "C" fn(device_index: u32) -> Gna2Status;
pub type FnGna2DeviceSetNumberOfThreads =
    unsafe extern "C" fn(device_index: u32, number_of_threads: u32) -> Gna2Status;

pub type FnGna2MemoryAlloc = unsafe extern "C" fn(
    size_requested: u32,
    size_granted: *mut u32,
    memory_address: *mut *mut c_void,
) -> Gna2Status;
pub type FnGna2MemoryAllocForDevice = unsafe extern "C" fn(
    device_index: u32,
    size_requested: u32,
    size_granted: *mut u32,
    memory_address: *mut *mut c_void,
) -> Gna2Status;
pub type FnGna2MemoryFree = unsafe extern "C" fn(memory: *mut c_void) -> Gna2Status;
pub type FnGna2MemorySetTag = unsafe extern "C" fn(memory: *mut c_void, tag: u32) -> Gna2Status;

pub type FnGna2RequestConfigCreate =
    unsafe extern "C" fn(model_id: u32, request_config_id: *mut u32) -> Gna2Status;
pub type FnGna2RequestConfigSetOperandBuffer = unsafe extern "C" fn(
    request_config_id: u32,
    operation_index: u32,
    operand_index: u32,
    address: *mut c_void,
) -> Gna2Status;
pub type FnGna2RequestConfigSetAccelerationMode = unsafe extern "C" fn(
    request_config_id: u32,
    acceleration_mode: Gna2AccelerationMode,
) -> Gna2Status;
pub type FnGna2RequestConfigRelease = unsafe extern "C" fn(request_config_id: u32) -> Gna2Status;
pub type FnGna2RequestEnqueue =
    unsafe extern "C" fn(request_config_id: u32, request_id: *mut u32) -> Gna2Status;
pub type FnGna2RequestWait =
    unsafe extern "C" fn(request_id: u32, timeout_milliseconds: u32) -> Gna2Status;

pub type FnGna2InstrumentationConfigCreate = unsafe extern "C" fn(
    number_of_points: u32,
    selected_points: *const Gna2InstrumentationPoint,
    results: *mut u64,
    instrumentation_config_id: *mut u32,
) -> Gna2Status;
pub type FnGna2InstrumentationConfigAssignToRequestConfig = unsafe extern "C" fn(
    instrumentation_config_id: u32,
    request_config_id: u32,
) -> Gna2Status;
pub type FnGna2InstrumentationConfigRelease =
    unsafe extern "C" fn(instrumentation_config_id: u32) -> Gna2Status;

pub type FnGna2StatusGetMessage =
    unsafe extern "C" fn(status: Gna2Status, buffer: *mut c_char, buffer_size: u32) -> Gna2Status;
pub type FnGna2StatusGetMaxMessageLength = unsafe extern "C" fn() -> u32;

/// Dynamically loaded symbol table from the GNA DLL.
pub struct GnaSymbolTable {
    pub device_get_count: FnGna2DeviceGetCount,
    pub device_get_version: FnGna2DeviceGetVersion,
    pub device_open: FnGna2DeviceOpen,
    pub device_close: FnGna2DeviceClose,
    pub device_set_number_of_threads: Option<FnGna2DeviceSetNumberOfThreads>,

    pub memory_alloc: FnGna2MemoryAlloc,
    pub memory_alloc_for_device: Option<FnGna2MemoryAllocForDevice>,
    pub memory_free: FnGna2MemoryFree,
    pub memory_set_tag: Option<FnGna2MemorySetTag>,

    pub request_config_create: Option<FnGna2RequestConfigCreate>,
    pub request_config_set_operand_buffer: Option<FnGna2RequestConfigSetOperandBuffer>,
    pub request_config_set_acceleration_mode: Option<FnGna2RequestConfigSetAccelerationMode>,
    pub request_config_release: Option<FnGna2RequestConfigRelease>,
    pub request_enqueue: Option<FnGna2RequestEnqueue>,
    pub request_wait: Option<FnGna2RequestWait>,

    pub instrumentation_config_create: Option<FnGna2InstrumentationConfigCreate>,
    pub instrumentation_config_assign_to_request_config:
        Option<FnGna2InstrumentationConfigAssignToRequestConfig>,
    pub instrumentation_config_release: Option<FnGna2InstrumentationConfigRelease>,

    pub status_get_message: Option<FnGna2StatusGetMessage>,
    pub status_get_max_message_length: Option<FnGna2StatusGetMaxMessageLength>,
}

impl GnaSymbolTable {
    /// Load required and optional function symbols from the loaded library.
    pub fn load(lib: &libloading::Library) -> Result<Self> {
        fn load_required<T: Copy>(
            lib: &libloading::Library,
            name: &'static [u8],
            symbol_str: &'static str,
        ) -> Result<T> {
            unsafe {
                match lib.get(name) {
                    Ok(sym) => Ok(*sym),
                    Err(err) => Err(GnaError::SymbolNotFound {
                        symbol: symbol_str,
                        source: err,
                    }),
                }
            }
        }

        fn load_optional<T: Copy>(lib: &libloading::Library, name: &[u8]) -> Option<T> {
            unsafe { lib.get(name).ok().map(|sym| *sym) }
        }

        let device_get_count: FnGna2DeviceGetCount =
            load_required(lib, b"Gna2DeviceGetCount\0", "Gna2DeviceGetCount")?;
        let device_get_version: FnGna2DeviceGetVersion =
            load_required(lib, b"Gna2DeviceGetVersion\0", "Gna2DeviceGetVersion")?;
        let device_open: FnGna2DeviceOpen =
            load_required(lib, b"Gna2DeviceOpen\0", "Gna2DeviceOpen")?;
        let device_close: FnGna2DeviceClose =
            load_required(lib, b"Gna2DeviceClose\0", "Gna2DeviceClose")?;

        let device_set_number_of_threads =
            load_optional(lib, b"Gna2DeviceSetNumberOfThreads\0");

        let memory_alloc: FnGna2MemoryAlloc =
            load_required(lib, b"Gna2MemoryAlloc\0", "Gna2MemoryAlloc")?;
        let memory_alloc_for_device =
            load_optional(lib, b"Gna2MemoryAllocForDevice\0");
        let memory_free: FnGna2MemoryFree =
            load_required(lib, b"Gna2MemoryFree\0", "Gna2MemoryFree")?;
        let memory_set_tag = load_optional(lib, b"Gna2MemorySetTag\0");

        let request_config_create = load_optional(lib, b"Gna2RequestConfigCreate\0");
        let request_config_set_operand_buffer =
            load_optional(lib, b"Gna2RequestConfigSetOperandBuffer\0");
        let request_config_set_acceleration_mode =
            load_optional(lib, b"Gna2RequestConfigSetAccelerationMode\0");
        let request_config_release = load_optional(lib, b"Gna2RequestConfigRelease\0");
        let request_enqueue = load_optional(lib, b"Gna2RequestEnqueue\0");
        let request_wait = load_optional(lib, b"Gna2RequestWait\0");

        let instrumentation_config_create =
            load_optional(lib, b"Gna2InstrumentationConfigCreate\0");
        let instrumentation_config_assign_to_request_config =
            load_optional(lib, b"Gna2InstrumentationConfigAssignToRequestConfig\0");
        let instrumentation_config_release =
            load_optional(lib, b"Gna2InstrumentationConfigRelease\0");

        let status_get_message = load_optional(lib, b"Gna2StatusGetMessage\0");
        let status_get_max_message_length =
            load_optional(lib, b"Gna2StatusGetMaxMessageLength\0");

        Ok(Self {
            device_get_count,
            device_get_version,
            device_open,
            device_close,
            device_set_number_of_threads,
            memory_alloc,
            memory_alloc_for_device,
            memory_free,
            memory_set_tag,
            request_config_create,
            request_config_set_operand_buffer,
            request_config_set_acceleration_mode,
            request_config_release,
            request_enqueue,
            request_wait,
            instrumentation_config_create,
            instrumentation_config_assign_to_request_config,
            instrumentation_config_release,
            status_get_message,
            status_get_max_message_length,
        })
    }
}
