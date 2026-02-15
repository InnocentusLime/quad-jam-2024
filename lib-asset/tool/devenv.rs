use std::fs::{self, File};
use std::{path::PathBuf, process::ExitCode};

use anyhow::Context;
use clap::{Parser, Subcommand};
use lib_asset::level::LevelDef;
use lib_asset::*;

pub fn run() -> ExitCode {
    let cli = Cli::parse();
    let mut resolver = FsResolver::new();
    if let Some(base_dir) = cli.base {
        resolver.set_root(AssetRoot::Base, base_dir);
    }

    let result = match cli.command {
        Commands::ConvertAseprite { aseprite, out } => convert_aseprite(aseprite, out),
        Commands::CompileMap { map, out } => compile_impl::<LevelDef>(&resolver, map, out),
        Commands::CompileMapsDir { dir, out } => {
            compile_dir_impl::<LevelDef>(&resolver, "tmx", dir, out)
        }
    };

    match result {
        Ok(_) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("{e:#}");
            ExitCode::FAILURE
        }
    }
}

fn convert_aseprite(aseprite: PathBuf, out: PathBuf) -> anyhow::Result<()> {
    let anims = animation_manifest::aseprite_load::load_animations_aseprite(aseprite, None)?;
    let out = File::create(out).context("open destination")?;
    serde_json::to_writer_pretty(out, &anims).context("writing to dest")
}

fn compile_dir_impl<T: DevableAsset + serde::Serialize>(
    resolver: &FsResolver,
    file_extension: &str,
    dir: PathBuf,
    out: PathBuf,
) -> anyhow::Result<()> {
    let dir = fs::read_dir(dir)?;
    for file in dir {
        let file = file?.path();
        let name = file.file_name().expect("File in DirEntry has no name");
        let Some(extension) = file.extension() else {
            continue;
        };
        if extension != file_extension {
            continue;
        }

        let mut buff = out.clone();
        buff.push(name);
        buff.set_extension("json");
        compile_impl::<T>(resolver, file, buff)?;
    }
    Ok(())
}

fn compile_impl<T: DevableAsset + serde::Serialize>(
    resolver: &FsResolver,
    path: PathBuf,
    out: PathBuf,
) -> anyhow::Result<()> {
    println!("Compiling {path:?} into {out:?}");

    let val = T::load_dev(resolver, &path).context("loading")?;
    let out = fs::File::create(out).context("opening the output")?;
    serde_json::to_writer_pretty(out, &val).context("compiling")?;
    Ok(())
}

/// Asset compiler for lib-asset.
#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// The path to the project directory. By default,
    /// the current working directory is used.
    #[arg(long, value_name = "DIR")]
    base: Option<PathBuf>,
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Convert an aseprite animation pack into an animation
    /// pack for the editor and the game.
    ConvertAseprite {
        /// The aseprite package
        #[arg(short, long, value_name = "FILE")]
        aseprite: PathBuf,
        /// The animation package destination-file
        #[arg(short, long, value_name = "FILE")]
        out: PathBuf,
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
    /// Build all maps in specified directory and put
    /// the compiled maps in the other. Each map called
    /// "name.tmx" will be turned into "name.bin".
    CompileMapsDir {
        /// The directory to read the maps from
        #[arg(short, long, value_name = "DIR")]
        dir: PathBuf,
        /// The directory to put the results into
        #[arg(short, long, value_name = "DIR")]
        out: PathBuf,
    },
}
