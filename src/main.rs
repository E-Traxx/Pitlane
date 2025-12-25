const BANNER: &str = r"
    ____  ____________    ___    _   ________
   / __ \/  _/_  __/ /   /   |  / | / / ____/
  / /_/ // /  / / / /   / /| | /  |/ / __/
 / ____// /  / / / /___/ ___ |/ /|  / /___
/_/   /___/ /_/ /_____/_/  |_/_/ |_/_____/
";

mod types;

use clap::Parser;
use spinner::{SpinnerBuilder, SpinnerHandle};
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::thread;
use which::which;

use types::Args;

use crate::types::DEFAULT_TARGET;

fn main() {
    println!("{}", BANNER);

    // Progress bar
    //
    // it always starts with configure and build
    // if flashing is not used then it jumps to 100 %
    //
    // else it continues in flashing
    let sp = SpinnerBuilder::new("".into()).start();
    let missing_tools = ensure_tools().map(|_| Vec::new()).unwrap();
    // Fuck ooooooooooooooooooooooffo
    args_parser(sp, missing_tools);
}

fn args_parser(sp: SpinnerHandle, missing_tools: Vec<String>) -> Result<(), String> {
    let args = Args::parse();

    is_arg_valid(&args)?;

    // Generate the buffer for the paths
    let path = PathBuf::from(&args.dir);

    match args.command.as_str() {
        "configure" => 'configure: {
            sp.message("Configuring the Project".into());
            if missing_tools.contains(&"cmake".to_string())
                | missing_tools.contains(&"ninja".to_string())
                | missing_tools.contains(&"arm-none-eabi-gcc".to_string())
            {
                break 'configure;
            };
            configure(&args.preset, &path)?;
        }
        "build" => 'build: {
            if missing_tools.contains(&"cmake".to_string())
                | missing_tools.contains(&"ninja".to_string())
                | missing_tools.contains(&"arm-none-eabi-gcc".to_string())
            {
                break 'build;
            };
            sp.message("Building the Project\n".into());

            build(&args.preset, &path)?;
        }
        "flash" => 'flash: {
            sp.message("Flashing the Project\n".into());
            if missing_tools.contains(&types::DEFAULT_PROGRAMMER.to_string()) {
                println!("Please refer to the documentation for how to install the required tools");
                break 'flash;
            }
            flash(&args.preset, &path)?;
        }
        "devices" => 'devices: {
            sp.message("Listing Available Devices\n".into());
            if missing_tools.contains(&types::DEFAULT_PROGRAMMER.to_string()) {
                println!("Please refer to the documentation for how to install the required tools");
                break 'devices;
            }
            println!("Listing Available Devices");
            check_devices()?;
        }
        _ => {
            println!("Please use a valid command, build, devices, configure, flash");
        }
    };
    Ok(())
}

// TODO: REMOVE
fn check_if_tool_is_missing(missing_tools: Vec<String>, tool: Vec<String>) -> bool {
    for n in tool.iter() {
        if !missing_tools.contains(n) {
            return false;
        }
    }
    true
}

fn is_arg_valid(args: &Args) -> Result<(), String> {
    let command = &args.command;
    println!("Command: {command}");
    if command != "configure" && command != "build" && command != "devices" && command != "flash" {
        Err("Please use a valid command, Build, Build-Logs, Configure, Flash")?;
    }

    let preset = &args.preset;
    if preset != "Debug" && preset != "Release" {
        Err("Please use a valid preset, Debug or Release")?;
    }

    Ok(())
}

// Check if the Tools needed are installed
fn ensure_tools() -> Result<(), Vec<String>> {
    let tools = [
        "cmake",
        "ninja",
        "arm-none-eabi-gcc",
        types::DEFAULT_PROGRAMMER,
    ];
    let mut missing: Vec<String> = vec![];

    for tool in tools.iter() {
        if which(tool).is_err() {
            missing.push((*tool).to_string());
        }
    }

    if missing.len() > 0 {
        println!("Missing tools: {:#?}", missing);

        // WARN: WTF!!!!!!!!!!!!!!!!!!!!
        if missing.contains(&types::DEFAULT_PROGRAMMER.to_string()) {
            println!("Please refer to the documentation for how to install the required tools");
        }

        return Err(missing);
    }

    println!("All tools are present");
    Ok(())
}

// run Cmake
fn configure(preset: &str, path: &PathBuf) -> Result<(), String> {
    // Standard CMake configure and build steps

    // Configure using Debug Preset
    let status = Command::new("cmake")
        .args(["--preset", preset])
        .current_dir(path)
        .status()
        .map_err(|e| format!("Failed to run CMake: {e}"))?;

    if !status.success() {
        return Err(format!(
            "cmake --preset {} exited with {}",
            preset,
            status.code().unwrap_or(-1)
        ));
    }

    Ok(())
}

fn build(preset: &str, path: &PathBuf) -> Result<(), String> {
    // using the Preset, we build the project into Target
    let status = Command::new("cmake")
        .args(["--build", "--preset", preset])
        .current_dir(path)
        .status()
        .map_err(|e| format!("Failed to run CMake: {e}"))?;

    if !status.success() {
        return Err(format!(
            "cmake build exited with {}",
            status.code().unwrap_or(-1)
        ));
    }

    Ok(())
}

// // run STM32_Programmer_CLI
// fn flash(preset: &str, path: &PathBuf) -> Result<(), String> {
//     let elf_path = is_elf_present(path, preset)?;
//
//     println!("Flashing ELF: {}", elf_path.to_string_lossy());
//
//     let command = Command::new(types::DEFAULT_PROGRAMMER)
//         .args(["-c", "port=SWD", "-w", elf_path.to_string_lossy().as_ref()])
//         .args(["-v, -rst"]);
//     let status = &command
//         .status()
//         .map_err(|e| format!("Failed to run STM32 Programmer: {e}"))?;
//     println!("Flash Status: {}", status);
//     if !status.success() {
//         return Err(format!("flash exited with {}", status.code().unwrap_or(-1)));
//     }
//     Ok(())
// }

// run STM32_Programmer_CLI
fn flash(preset: &str, path: &PathBuf) -> Result<(), String> {
    let elf_path = is_elf_present(path, preset)?;

    println!("Flashing ELF: {}", elf_path.display());

    let mut command = Command::new(types::DEFAULT_PROGRAMMER);

    command
        .args(["-c", "port=SWD"])
        .args(["-w", elf_path.to_string_lossy().as_ref()])
        .args(["-v", "-rst"]);

    let status = command
        .status()
        .map_err(|e| format!("Failed to run STM32 Programmer: {e}"))?;

    println!("Flash Status: {}", status);

    if !status.success() {
        return Err(format!("flash exited with {}", status.code().unwrap_or(-1)));
    }

    Ok(())
}

fn is_elf_present(root_dir: &PathBuf, preset: &str) -> Result<PathBuf, String> {
    let elf_path = root_dir
        .join("build")
        .join(preset)
        .join(types::DEFAULT_TARGET);

    if !elf_path.exists() {
        return Err(format!("ELF not present"));
    }
    Ok(elf_path)
}

// Check if devices are connected
fn check_devices() -> Result<(), String> {
    let status = Command::new(types::DEFAULT_PROGRAMMER)
        .args(["-l"])
        .status()
        .map_err(|e| format!("Failed to run STM32 Programmer: {e}"))?;

    if !status.success() {
        return Err(format!("flash exited with {}", status.code().unwrap_or(-1)));
    }

    Ok(())
}
