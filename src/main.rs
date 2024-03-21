use clap::Parser;
use cps_deps::cps::Package;
use std::{error::Error, fs::File, io::BufReader};

#[derive(Parser)]
struct Args {
    cps_filepath: std::path::PathBuf,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    println!("reading cps file: {:?}", args.cps_filepath);

    let file = File::open(args.cps_filepath)?;
    let reader = BufReader::new(file);
    let package: Package = serde_json::from_reader(reader)?;

    dbg!(package);

    Ok(())
}
