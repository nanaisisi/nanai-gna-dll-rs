use std::env;
use std::path::PathBuf;

use nanai_gna_dll_rs::{GnaDevice, GnaLibrary};

fn main() {
    println!("==================================================");
    println!("  nanai-gna-dll-rs: GNA Dynamic DLL Loader Demo   ");
    println!("==================================================");

    let mut args = env::args().skip(1);
    let dll_path = if let Some(flag) = args.next() {
        if flag == "--dll" {
            args.next().map(PathBuf::from)
        } else {
            Some(PathBuf::from(flag))
        }
    } else {
        None
    };

    let library = match dll_path {
        Some(path) => {
            println!("Attempting to load DLL from specified path: {}", path.display());
            GnaLibrary::load_from_path(path)
        }
        None => {
            println!("Attempting to load default DLL (gna.dll) from environment / system path...");
            GnaLibrary::load_default()
        }
    };

    let library = match library {
        Ok(lib) => {
            println!("Successfully loaded DLL: {}", lib.path().display());
            lib
        }
        Err(err) => {
            eprintln!("\nDLL could not be loaded: {}", err);
            eprintln!("\nHint: You can specify a DLL path via command line argument:");
            eprintln!("  cargo run -- --dll path/to/gna.dll");
            eprintln!("Or set the GNA_LIB_PATH environment variable.");
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
