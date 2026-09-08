use std::env;
use std::path::PathBuf;

use nanai_gna_dll_rs::{GnaDevice, GnaLibrary};

fn print_usage(program: &str) {
    println!("Usage: {} [OPTIONS]", program);
    println!();
    println!("Options:");
    println!("  --dll <PATH>               Load DLL directly from specified file or directory path");
    println!("  --env <VAR_NAME>           Load DLL from specified environment variable");
    println!("  --stress [ITERATIONS]      Run stress load test (default: 1000 iterations)");
    println!("  --duration <SECS>          Run stress load test for specified duration in seconds");
    println!("  --concurrency <N>          Queue depth / in-flight requests for stress test (default: 1)");
    println!("  --help, -h                 Show this help message");
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
    let mut stress_test = false;
    let mut stress_iterations: Option<usize> = None;
    let mut stress_duration_secs: Option<u64> = None;
    let mut stress_concurrency: usize = 1;

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
            "--stress" => {
                stress_test = true;
                // Check if next arg is a number of iterations
                if let Some(val) = args.next() {
                    if let Ok(n) = val.parse::<usize>() {
                        stress_iterations = Some(n);
                    } else if val.starts_with('-') {
                        // It was another option, continue parsing with it
                        // Since we already advanced, handle it manually
                        // Alternatively, default to 1000 and handle the flag
                    } else {
                        stress_iterations = Some(1000);
                    }
                } else {
                    stress_iterations = Some(1000);
                }
            }
            "--duration" => {
                stress_test = true;
                if let Some(val) = args.next() {
                    match val.parse::<u64>() {
                        Ok(sec) => stress_duration_secs = Some(sec),
                        Err(_) => {
                            eprintln!("Error: --duration requires integer seconds.");
                            return;
                        }
                    }
                } else {
                    eprintln!("Error: --duration requires integer seconds.");
                    return;
                }
            }
            "--concurrency" => {
                if let Some(val) = args.next() {
                    match val.parse::<usize>() {
                        Ok(c) => stress_concurrency = c.max(1),
                        Err(_) => {
                            eprintln!("Error: --concurrency requires a positive integer.");
                            return;
                        }
                    }
                } else {
                    eprintln!("Error: --concurrency requires an integer value.");
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

                        println!("\n[2] Building and running a custom Dense (Fully Connected Affine) model...");
                        run_model_demo(&device, stress_test, stress_iterations, stress_duration_secs, stress_concurrency);

                        println!("\n[3] Dropping GnaDevice (device #0) now...");
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

fn round_up_to_64(size: usize) -> usize {
    (size + 63) & !63
}

fn round_up(val: usize, align: usize) -> usize {
    (val + align - 1) & !(align - 1)
}

fn run_model_demo(
    device: &GnaDevice,
    stress_test: bool,
    stress_iterations: Option<usize>,
    stress_duration_secs: Option<u64>,
    stress_concurrency: usize,
) {
    use std::time::Duration;
    use nanai_gna_dll_rs::{
        Gna2AccelerationMode, Gna2DataType, Gna2Tensor, GnaBuffer, GnaLoadTestConfig,
        GnaLoadTester, GnaModelBuilder, GnaRequestConfig,
    };

    const W: usize = 16;
    const H: usize = 8;
    const B: usize = 4;

    let weights: [i16; H * W] = [
        -6, -2, -1, -1, -2, 9, 6, 5, 2, 4, -1, 5, -2, -4, 0, 9,
        -8, 8, -4, 6, 5, 3, -7, -9, 7, 0, -4, -1, 1, 7, 6, -6,
        2, -8, 6, 5, -1, -2, 7, 5, -1, 4, 8, 7, -9, -1, 7, 1,
        0, -2, 1, 0, 6, -6, 7, 4, -6, 0, 3, -2, 1, 8, -6, -2,
        -6, -3, 4, -2, -8, -6, 6, 5, 6, -9, -5, -2, -5, -8, -6, -2,
        -7, 0, 6, -3, -1, -6, 4, 1, -4, -5, -3, 7, 9, -9, 9, 9,
        0, -2, 6, -3, 5, -2, -1, -3, -5, 7, 6, 6, -8, 0, -4, 9,
        2, 7, -8, -7, 8, -6, -6, 1, 7, -4, -4, 9, -6, -6, 5, -7,
    ];

    let inputs: [i16; W * B] = [
        -5, 9, -7, 4,
        5, -4, -7, 4,
        0, 7, 1, -7,
        1, 6, 7, 9,
        2, -4, 9, 8,
        -5, -1, 2, 9,
        -8, -8, 8, 1,
        -7, 2, -1, -1,
        -9, -5, -8, 5,
        0, -1, 3, 9,
        0, 8, 1, -2,
        -9, 8, 0, -7,
        -9, -8, -1, -4,
        -3, -7, -2, 3,
        -8, 0, 1, 3,
        -4, -6, -8, -2,
    ];

    let biases: [i32; H] = [5, 4, -2, 5, -7, -5, 4, -1];

    let buf_size_weights = round_up_to_64(std::mem::size_of_val(&weights));
    let buf_size_inputs = round_up_to_64(std::mem::size_of_val(&inputs));
    let buf_size_biases = round_up_to_64(std::mem::size_of_val(&biases));
    let buf_size_outputs = round_up_to_64(H * B * 4);

    let rw_buffer_size = round_up(buf_size_inputs + buf_size_outputs, 0x1000);
    let bytes_requested = rw_buffer_size + buf_size_weights + buf_size_biases;

    println!("  Allocating contiguous pinned memory for model: {} bytes", bytes_requested);
    let mem = match GnaBuffer::new(device.library(), bytes_requested) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("  Failed to allocate model memory: {}", e);
            return;
        }
    };

    let base_ptr = mem.as_raw_ptr() as *mut u8;
    let inputs_ptr = base_ptr as *mut i16;
    let outputs_ptr = unsafe { base_ptr.add(buf_size_inputs) } as *mut i32;
    let weights_ptr = unsafe { base_ptr.add(rw_buffer_size) } as *mut i16;
    let biases_ptr = unsafe { base_ptr.add(rw_buffer_size + buf_size_weights) } as *mut i32;

    unsafe {
        std::ptr::copy_nonoverlapping(inputs.as_ptr(), inputs_ptr, inputs.len());
        std::ptr::copy_nonoverlapping(weights.as_ptr(), weights_ptr, weights.len());
        std::ptr::copy_nonoverlapping(biases.as_ptr(), biases_ptr, biases.len());
        std::ptr::write_bytes(outputs_ptr, 0, H * B);
    }

    let input_tensor = Gna2Tensor::d2(W as u32, B as u32, Gna2DataType::Int16, inputs_ptr as _);
    let output_tensor = Gna2Tensor::d2(H as u32, B as u32, Gna2DataType::Int32, outputs_ptr as _);
    let weight_tensor = Gna2Tensor::d2(H as u32, W as u32, Gna2DataType::Int16, weights_ptr as _);
    let bias_tensor = Gna2Tensor::d1(H as u32, Gna2DataType::Int32, biases_ptr as _);

    let model = match GnaModelBuilder::new()
        .add_fully_connected_affine(
            input_tensor,
            output_tensor,
            weight_tensor,
            bias_tensor,
            None,
        )
        .build(device)
    {
        Ok(m) => {
            println!("  Model compiled successfully! Model ID: {}", m.id());
            m
        }
        Err(e) => {
            eprintln!("  Failed to compile model: {}", e);
            return;
        }
    };

    // Configure request
    let mut request_config = match GnaRequestConfig::create(device.library(), model.id()) {
        Ok(rc) => rc,
        Err(e) => {
            eprintln!("  Failed to create request config: {}", e);
            return;
        }
    };

    if let Err(e) = unsafe { request_config.set_operand_buffer(0, 0, inputs_ptr as _) } {
        eprintln!("  Failed to set input buffer: {}", e);
        return;
    }
    if let Err(e) = unsafe { request_config.set_operand_buffer(0, 1, outputs_ptr as _) } {
        eprintln!("  Failed to set output buffer: {}", e);
        return;
    }

    let _ = request_config.set_acceleration_mode(Gna2AccelerationMode::Auto);

    if stress_test {
        println!("\n  >>> Running Stress / Load Test <<<");
        let mut load_config = GnaLoadTestConfig::new()
            .with_concurrency(stress_concurrency);

        if let Some(sec) = stress_duration_secs {
            println!("  Load test mode: Time-based ({} seconds), Concurrency: {}", sec, stress_concurrency);
            load_config = load_config.with_duration(Duration::from_secs(sec));
            if stress_iterations.is_some() {
                load_config.iterations = stress_iterations;
            } else {
                load_config.iterations = None;
            }
        } else {
            let iters = stress_iterations.unwrap_or(1000);
            println!("  Load test mode: Iterations-based ({} runs), Concurrency: {}", iters, stress_concurrency);
            load_config = load_config.with_iterations(iters);
        }

        let tester = GnaLoadTester::new(load_config);
        match tester.run(&mut request_config) {
            Ok(report) => {
                println!();
                report.print_summary();
            }
            Err(err) => eprintln!("  Stress test failed: {}", err),
        }
        return;
    }

    if let Err(e) = request_config.enable_performance_counter() {
        println!("  (Performance counter not available or disabled: {})", e);
    }

    println!("  Enqueuing inference request...");
    let req_id = match request_config.enqueue() {
        Ok(id) => id,
        Err(e) => {
            eprintln!("  Enqueue failed: {}", e);
            return;
        }
    };

    println!("  Waiting for inference completion (request ID: {})...", req_id);
    match request_config.wait(req_id, 1000) {
        Ok(()) => println!("  Inference request completed successfully!"),
        Err(e) => {
            eprintln!("  Inference failed / timed out: {}", e);
            return;
        }
    }

    // Report performance statistics if available
    if let Ok(stats) = request_config.get_performance_stats() {
        println!("  Inference Performance Metrics:");
        println!("    Total Cycles:   {}", stats.total_cycles);
        println!("    Stall Cycles:   {}", stats.stall_cycles);
        println!("    Active Cycles:  {}", stats.active_cycles);
        println!("    HW Utilization: {:.2}%", stats.hw_usage_percentage());
        if let Some(t) = stats.execution_time {
            println!("    Execution Time: {} cycles/us", t);
        }
    }

    let results = unsafe { std::slice::from_raw_parts(outputs_ptr, H * B) };
    println!("  Inference outputs ({} rows x {} cols):", H, B);
    for row in 0..H {
        print!("   ");
        for col in 0..B {
            print!("\t{}", results[row * B + col]);
        }
        println!();
    }
}
