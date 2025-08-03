#![cfg(not(target_family = "wasm"))]

use std::error::Error as StdError;
use std::fs;
use std::{path::PathBuf, process::ExitCode};

use clap::{Parser, Subcommand};

fn main() -> ExitCode {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::CheckMap { map } => check_map(map),
        Commands::CompileMap { map, out } => compile_map(map, out),
        Commands::DumpMap { map } => dump_map(map),
        Commands::CompileDir { dir, out } => compile_dir(dir, out),
    };

    match result {
        Ok(_) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("{e}");
            ExitCode::FAILURE
        }
    }
}

fn check_map(map: PathBuf) -> Result<(), Box<dyn StdError>> {
    println!("Checking {map:?}");

    lib_level::tiled_load::load_level(map)?;
    Ok(())
}

fn compile_map(map: PathBuf, out: PathBuf) -> Result<(), Box<dyn StdError>> {
    println!("Compiling {map:?} into {out:?}");

    let level = lib_level::tiled_load::load_level(map)?;
    let out = fs::File::create(out)?;
    lib_level::binary_io::compile::write_level(&level, out)
}

fn dump_map(map: PathBuf) -> Result<(), Box<dyn StdError>> {
    let level_data = fs::read(map)?;
    let level = lib_level::binary_io::load_from_memory(&level_data)?;
    println!("{:?}", level);
    Ok(())
}

fn compile_dir(dir: PathBuf, out: PathBuf) -> Result<(), Box<dyn StdError>> {
    let dir = fs::read_dir(dir)?;
    for file in dir {
        let file = file?.path();
        let name = file.file_name().expect("File in DirEntry has no name");
        let Some(extension) = file.extension() else {
            continue;
        };
        if extension != "tmx" {
            continue;
        }

        let mut buff = out.clone();
        buff.push(name);
        buff.set_extension("bin");
        compile_map(file, buff)?;
    }
    Ok(())
}

/// A tool for working with the game's maps.
#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Check if a map satisfies all conventions
    CheckMap {
        /// The map to check
        #[arg(short, long, value_name = "FILE")]
        map: PathBuf,
    },
    /// Convert a map into binary format
    CompileMap {
        /// The map to compile
        #[arg(short, long, value_name = "FILE")]
        map: PathBuf,
        /// The output file
        #[arg(short, long, value_name = "FILE")]
        out: PathBuf,
    },
    /// Debug-dump a binary map
    DumpMap {
        /// The map to dump
        #[arg(short, long, value_name = "FILE")]
        map: PathBuf,
    },
    /// Build all maps in specified directory and put
    /// the compiled maps in the other. Each map called
    /// "name.tmx" will be turned into "name.bin".
    CompileDir {
        /// The directory to read the maps from
        #[arg(short, long, value_name = "DIR")]
        dir: PathBuf,
        /// The directory to put the results into
        #[arg(short, long, value_name = "DIR")]
        out: PathBuf,
    },
}
