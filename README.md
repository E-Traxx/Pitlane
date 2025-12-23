# pitlane_rs

pitlane_rs is a small Rust command line tool that helps build and flash the PULSAR firmware. It checks that the usual firmware tools are installed, runs the CMake presets for you, and calls STM32_Programmer_CLI to flash the `Pulsar.elf` image.

## Requirements
- Rust toolchain (edition 2024) to compile this utility
- CMake presets available in the project directory
- Tools on PATH: `cmake`, `ninja`, `arm-none-eabi-gcc`, `STM32_Programmer_CLI`

The program prints a list of missing tools and stops if any of them are not found.

## Build
```bash
cargo build --release
```
Binary output: `target/release/pitlane_rs`.

During development you can run `cargo run -- --command <name> ...`.

## Usage
```
pitlane_rs --command <configure|build|flash|devices> [--dir <path>] [--preset <Debug|Release>]
```

- `--command` (or `-c`) selects the action.
- `--dir` (or `-d`) points to the project root. Default is the current directory.
- `--preset` (or `-p`) selects the CMake preset. Default is `Debug`.

### Typical flow
```bash
# Configure the build tree
cargo run -- --command configure --preset Debug --dir /path/to/project

# Build the firmware
cargo run -- --command build --preset Debug --dir /path/to/project

# Flash build/Debug/Pulsar.elf over SWD
cargo run -- --command flash --preset Debug --dir /path/to/project

# List devices from STM32_Programmer_CLI
cargo run -- --command devices
```

Flashing assumes that `build/<preset>/Pulsar.elf` exists. If it does not, run configure and build first or adjust the preset so the tool looks in the correct folder.

## Troubleshooting
- Missing tool message: install the listed tools and try again.
- Unknown preset: check `CMakePresets.json` in your project root.
- ELF not found: confirm the build produced `Pulsar.elf` under `build/<preset>`.

@ Ash (21akame03)
https://github.com/21Akame03
