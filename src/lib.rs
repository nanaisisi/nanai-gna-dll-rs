pub mod device;
pub mod error;
pub mod inference;
pub mod loader;
pub mod memory;
pub mod symbol_table;
pub mod types;

// Re-export core items at root
pub use device::GnaDevice;
pub use error::{GnaError, Result};
pub use inference::GnaRequestConfig;
pub use loader::GnaLibrary;
pub use memory::GnaBuffer;
pub use types::{
    Gna2AccelerationMode, Gna2DeviceVersion, Gna2InstrumentationPoint, Gna2Status,
    GNA2_STATUS_SUCCESS,
};
