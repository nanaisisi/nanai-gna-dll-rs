use std::ffi::c_void;

use crate::error::{GnaError, Result};
use crate::loader::GnaLibrary;
use crate::types::{Gna2AccelerationMode, GNA2_STATUS_SUCCESS};

/// High-level wrapper for GNA Request Configuration.
pub struct GnaRequestConfig {
    library: GnaLibrary,
    config_id: u32,
}

impl GnaRequestConfig {
    /// Create a new request configuration for the given model ID.
    pub fn create(library: &GnaLibrary, model_id: u32) -> Result<Self> {
        let create_fn = library
            .symbols()
            .request_config_create
            .ok_or_else(|| GnaError::Other("Gna2RequestConfigCreate not supported".into()))?;

        let mut config_id: u32 = 0;
        let status = unsafe { create_fn(model_id, &mut config_id) };
        if status != GNA2_STATUS_SUCCESS {
            return Err(GnaError::from_status(status));
        }

        Ok(Self {
            library: library.clone(),
            config_id,
        })
    }

    /// Set an operand buffer for this configuration.
    ///
    /// # Safety
    /// `buffer_ptr` must be a valid pointer to allocated memory of the expected size for the given operation operand,
    /// and it must remain valid and not be accessed concurrently in an invalid manner during inference execution.
    pub unsafe fn set_operand_buffer(
        &mut self,
        operation_index: u32,
        operand_index: u32,
        buffer_ptr: *mut c_void,
    ) -> Result<()> {
        let set_buffer_fn = self
            .library
            .symbols()
            .request_config_set_operand_buffer
            .ok_or_else(|| {
                GnaError::Other("Gna2RequestConfigSetOperandBuffer not supported".into())
            })?;

        let status = unsafe {
            set_buffer_fn(
                self.config_id,
                operation_index,
                operand_index,
                buffer_ptr,
            )
        };
        if status != GNA2_STATUS_SUCCESS {
            return Err(GnaError::from_status(status));
        }
        Ok(())
    }

    /// Enable active outputs list for an operation operand.
    pub fn enable_active_list(&mut self, operation_index: u32, indices: &[u32]) -> Result<()> {
        let enable_list_fn = self
            .library
            .symbols()
            .request_config_enable_active_list
            .ok_or_else(|| {
                GnaError::Other("Gna2RequestConfigEnableActiveList not supported".into())
            })?;

        let status = unsafe {
            enable_list_fn(
                self.config_id,
                operation_index,
                indices.len() as u32,
                indices.as_ptr(),
            )
        };
        if status != GNA2_STATUS_SUCCESS {
            return Err(GnaError::from_status(status));
        }
        Ok(())
    }

    /// Enable hardware consistency for the request configuration.
    pub fn enable_hardware_consistency(
        &mut self,
        device_version: crate::types::Gna2DeviceVersion,
    ) -> Result<()> {
        let consistency_fn = self
            .library
            .symbols()
            .request_config_enable_hardware_consistency
            .ok_or_else(|| {
                GnaError::Other("Gna2RequestConfigEnableHardwareConsistency not supported".into())
            })?;

        let status = unsafe { consistency_fn(self.config_id, device_version) };
        if status != GNA2_STATUS_SUCCESS {
            return Err(GnaError::from_status(status));
        }
        Ok(())
    }

    /// Set acceleration mode.
    pub fn set_acceleration_mode(&mut self, mode: Gna2AccelerationMode) -> Result<()> {
        let set_mode_fn = self
            .library
            .symbols()
            .request_config_set_acceleration_mode
            .ok_or_else(|| {
                GnaError::Other("Gna2RequestConfigSetAccelerationMode not supported".into())
            })?;

        let status = unsafe { set_mode_fn(self.config_id, mode) };
        if status != GNA2_STATUS_SUCCESS {
            return Err(GnaError::from_status(status));
        }
        Ok(())
    }

    /// Enqueue an asynchronous request using this configuration and return request ID.
    pub fn enqueue(&self) -> Result<u32> {
        let enqueue_fn = self
            .library
            .symbols()
            .request_enqueue
            .ok_or_else(|| GnaError::Other("Gna2RequestEnqueue not supported".into()))?;

        let mut request_id: u32 = 0;
        let status = unsafe { enqueue_fn(self.config_id, &mut request_id) };
        if status != GNA2_STATUS_SUCCESS {
            return Err(GnaError::from_status(status));
        }
        Ok(request_id)
    }

    /// Wait for request completion with a timeout in milliseconds.
    pub fn wait_request(&self, request_id: u32, timeout_ms: u32) -> Result<()> {
        let wait_fn = self
            .library
            .symbols()
            .request_wait
            .ok_or_else(|| GnaError::Other("Gna2RequestWait not supported".into()))?;

        let status = unsafe { wait_fn(request_id, timeout_ms) };
        if status != GNA2_STATUS_SUCCESS {
            return Err(GnaError::from_status(status));
        }
        Ok(())
    }

    /// Get configuration ID.
    pub fn id(&self) -> u32 {
        self.config_id
    }
}

impl Drop for GnaRequestConfig {
    fn drop(&mut self) {
        if let Some(release_fn) = self.library.symbols().request_config_release {
            unsafe {
                let _ = release_fn(self.config_id);
            }
        }
    }
}
