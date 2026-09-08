use std::env;
use std::path::PathBuf;

use nanai_gna_dll_rs::{GnaDevice, GnaLibrary};

fn print_usage(program: &str) {
    println!("Usage: {} [OPTIONS]", program);
    println!();
    println!("Options:");
    println!("  --dll <PATH>       Load DLL directly from specified file or directory path");
    println!("  --env <VAR_NAME>   Load DLL from specified environment variable");
    println!("  --help, -h         Show this help message");
    println!();
    println!("If no option is specified, standard search order is used:");
    println!("  1. GNA_LIB_PATH environment variable");
    println!("  2. GNA_LIB_DIR environment variable");
    println!("  3. Current directory ({})", GnaLibrary::default_dll_name());
    println!("  4. System library search path");
}

fn main() {
    println!("==================================================");
    println!("  nanai-gna-dll-rs: GNA Dynamic DLL Loader Demo   ");
    println!("==================================================");

    let mut args = env::args();
    let program = args.next().unwrap_or_else(|| "nanai-gna-dll-rs".into());

    let mut dll_path: Option<PathBuf> = None;
    let mut env_var: Option<String> = None;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--dll" => {
                if let Some(val) = args.next() {
                    dll_path = Some(PathBuf::from(val));
                } else {
                    eprintln!("Error: --dll requires a path argument.");
                    return;
                }
            }
            "--env" => {
                if let Some(val) = args.next() {
                    env_var = Some(val);
                } else {
                    eprintln!("Error: --env requires an environment variable name.");
                    return;
                }
            }
            "--help" | "-h" => {
                print_usage(&program);
                return;
            }
            other if other.starts_with('-') => {
                eprintln!("Unknown option: {}", other);
                print_usage(&program);
                return;
            }
            positional => {
                // Allow passing the DLL path directly as a positional argument
                dll_path = Some(PathBuf::from(positional));
            }
        }
    }

    let library = if let Some(path) = dll_path {
        println!("Attempting to load DLL from specified path: {}", path.display());
        GnaLibrary::load_from_path(path)
    } else if let Some(var) = env_var {
        println!("Attempting to load DLL from environment variable: {}", var);
        GnaLibrary::load_from_env(&var)
    } else {
        println!("Attempting to load default DLL ({}) from environment / system path...", GnaLibrary::default_dll_name());
        GnaLibrary::load_default()
    };

    let library = match library {
        Ok(lib) => {
            println!("Successfully loaded DLL: {}", lib.path().display());
            lib
        }
        Err(err) => {
            eprintln!("\nDLL could not be loaded: {}", err);
            eprintln!("\nHint:");
            eprintln!("  1. Specify a DLL path: cargo run -- --dll path/to/{}", GnaLibrary::default_dll_name());
            eprintln!("  2. Or specify an environment variable: cargo run -- --env MY_GNA_PATH");
            eprintln!("  3. Or set standard GNA_LIB_PATH / GNA_LIB_DIR.");
            return;
        }
    };

    // Query available devices
    match GnaDevice::get_count(&library) {
        Ok(count) => {
            println!("Available GNA devices: {}", count);
            for i in 0..count {
                match GnaDevice::get_version(&library, i) {
                    Ok(ver) => println!("  Device #{}: version 0x{:02x} ({})", i, ver.0, ver.as_str()),
                    Err(e) => eprintln!("  Device #{}: failed to query version: {}", i, e),
                }
            }

            if count > 0 {
                println!("\nOpening first device (index 0)...");
                match GnaDevice::open(&library, 0) {
                    Ok(device) => {
                        println!("Successfully opened device #0! Version: {}", device.version().as_str());

                        // Allocate buffer inside a block to test GnaBuffer drop first
                        {
                            println!("\n[1] Allocating a 1024-byte buffer...");
                            match device.allocate_buffer(1024) {
                                Ok(mut buf) => {
                                    println!("Successfully allocated buffer of {} bytes!", buf.len());
                                    buf[0..4].copy_from_slice(&[0x12, 0x34, 0x56, 0x78]);
                                    println!("First 4 bytes set to: {:02x?}", &buf[0..4]);
                                    println!("Dropping GnaBuffer now...");
                                }
                                Err(e) => eprintln!("Failed to allocate buffer: {}", e),
                            }
                            println!("GnaBuffer dropped.");
                        }

                        println!("\n[2] Dropping GnaDevice (device #0) now...");
                    }
                    Err(e) => eprintln!("Failed to open device #0: {}", e),
                }
                println!("GnaDevice dropped.");
            }
        }
        Err(err) => eprintln!("Failed to query device count: {}", err),
    }

    println!("\nCompleted demo successfully.");
}
