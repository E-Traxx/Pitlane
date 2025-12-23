use clap::Parser;

pub const DEFAULT_TARGET: &str = "Pulsar.elf";
pub const DEFAULT_PRESET: &str = "Debug";
pub const DEFAULT_PROGRAMMER: &str = "STM32_Programmer_CLI";

// Simple parser
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    // for the below ,
    // short = -c
    // long = --command

    // Command to execute {Build, Clean, Configure, Flash}
    #[arg(short, long)]
    pub command: String,

    // root directory of Project
    #[arg(short, long, default_value = ".")]
    pub dir: String,

    // Cmake preset {Debug, release, release_min}
    #[arg(short, long, default_value = DEFAULT_PRESET)]
    pub preset: String,
}
