use anyhow::Result;
use clap::{Parser, Subcommand};
use cps_deps::cps::parse_and_print_cps;
use cps_deps::generate_from_pkg_config::{generate_all_from_pkg_config, generate_from_pkg_config};
use std::path::PathBuf;

/// Common Package Specification (CPS) deps
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Generate cps files from pkg-config files found on your system
    GenerateAll {
        #[arg(value_name = "OUTDIR")]
        outdir: PathBuf,
    },
    /// Generate a cps file from a pkg config file
    Generate {
        #[arg(value_name = "PC_FILE")]
        pc: PathBuf,
        #[arg(value_name = "CPS_FILE")]
        cps: PathBuf,
    },
    /// Parse a CPS file and display the result
    ParseCps {
        #[arg(value_name = "FILE")]
        filepath: PathBuf,
    },
}

fn main() -> Result<()> {
    let args = Args::parse();

    match &args.command {
        Commands::GenerateAll { outdir } => generate_all_from_pkg_config(outdir),
        Commands::Generate { pc, cps } => generate_from_pkg_config(pc, cps),
        Commands::ParseCps { filepath } => parse_and_print_cps(filepath),
    }
}
