use anyhow::Result;
use clap::{Parser, Subcommand};
use cps_deps::cps::parse_and_print_cps;
use cps_deps::generate_from_pkg_config::generate_from_pkg_config;
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
    Generate {
        #[arg(value_name = "OUTDIR")]
        outdir: PathBuf,
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
        Commands::Generate { outdir } => generate_from_pkg_config(outdir),
        Commands::ParseCps { filepath } => parse_and_print_cps(filepath),
    }
}
