use std::env;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::error::{GnaError, Result};
use crate::symbol_table::GnaSymbolTable;

/// Represents a dynamically loaded GNA runtime library.
///
/// Wrapping `libloading::Library` in an `Arc` ensures that loaded symbols
/// remain valid as long as any device or buffer referencing this library is alive.
#[derive(Clone)]
pub struct GnaLibrary {
    _lib: Arc<libloading::Library>,
    symbols: Arc<GnaSymbolTable>,
    path: PathBuf,
}

impl GnaLibrary {
    /// Attempt to load the GNA dynamic library from a specific path.
    pub fn load_from_path<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path_ref = path.as_ref();
        let lib = unsafe { libloading::Library::new(path_ref) }
            .map_err(GnaError::LibraryLoadError)?;

        let symbols = GnaSymbolTable::load(&lib)?;

        Ok(Self {
            _lib: Arc::new(lib),
            symbols: Arc::new(symbols),
            path: path_ref.to_path_buf(),
        })
    }

    /// Load the GNA library by searching standard and configured locations:
    /// 1. Environment variable `GNA_LIB_PATH`
    /// 2. Directory specified by `GNA_LIB_DIR` / `gna.dll` (or `libgna.so`)
    /// 3. Current working directory
    /// 4. System DLL search path
    pub fn load_default() -> Result<Self> {
        // 1. Direct file path from env
        if let Ok(path_str) = env::var("GNA_LIB_PATH") {
            let path = PathBuf::from(path_str);
            if path.exists() {
                return Self::load_from_path(path);
            }
        }

        // 2. Directory from env
        let default_dll_name = if cfg!(target_os = "windows") {
            "gna.dll"
        } else if cfg!(target_os = "macos") {
            "libgna.dylib"
        } else {
            "libgna.so"
        };

        if let Ok(dir_str) = env::var("GNA_LIB_DIR") {
            let path = PathBuf::from(dir_str).join(default_dll_name);
            if path.exists() {
                return Self::load_from_path(path);
            }
        }

        // 3. Current working directory
        let local_path = PathBuf::from(default_dll_name);
        if local_path.exists() {
            return Self::load_from_path(local_path);
        }

        // 4. Default system search path
        Self::load_from_path(OsStr::new(default_dll_name))
    }

    /// Get the path of the loaded library.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Access the resolved symbol table.
    pub fn symbols(&self) -> &GnaSymbolTable {
        &self.symbols
    }
}
