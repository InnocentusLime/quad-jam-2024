#![cfg(not(target_family = "wasm"))]

use std::fs::{self, File};
use std::{path::PathBuf, process::ExitCode};

use anyhow::Context;
use clap::{Parser, Subcommand};
use lib_asset::FsResolver;

fn main() -> ExitCode {
    let cli = Cli::parse();
    let mut resolver = FsResolver::new();
    if let Some(asset_dir) = cli.assets {
        resolver.set_assets_dir(asset_dir).unwrap();
    }

    let result = match cli.command {
        Commands::CheckAnims { animations } => check_animations(animations),
        Commands::CompileAnims { animations, out } => compile_animations(animations, out),
        Commands::DumpAnims { animations } => dump_animations(animations),
        Commands::CompileDir { dir, out } => compile_dir(dir, out),
        Commands::ConvertAseprite { aseprite, out } => convert_aseprite(&resolver, aseprite, out),
    };

    match result {
        Ok(_) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("{e:#}");
            ExitCode::FAILURE
        }
    }
}

fn check_animations(animations: PathBuf) -> anyhow::Result<()> {
    println!("Checking {animations:?}");

    lib_anim::aseprite_load::load_animations_project(animations)?;
    Ok(())
}

fn compile_animations(animations: PathBuf, out: PathBuf) -> anyhow::Result<()> {
    println!("Compiling {animations:?} into {out:?}");

    let anims =
        lib_anim::aseprite_load::load_animations_project(animations).context("loading package")?;
    let out = fs::File::create(out).context("opening the output")?;
    lib_anim::binary_io::compile::write_animation_pack(&anims, out).context("writing the package")
}

fn dump_animations(animations: PathBuf) -> anyhow::Result<()> {
    let anims_data = fs::read(animations)?;
    let anims = lib_anim::binary_io::load_from_memory(&anims_data)?;
    println!("{:?}", anims);
    Ok(())
}

fn compile_dir(dir: PathBuf, out: PathBuf) -> anyhow::Result<()> {
    let dir = fs::read_dir(dir)?;
    for file in dir {
        let file = file?.path();
        let name = file.file_name().expect("File in DirEntry has no name");
        let Some(extension) = file.extension() else {
            continue;
        };
        if extension != "json" {
            continue;
        }

        let mut buff = out.clone();
        buff.push(name);
        buff.set_extension("bin");
        compile_animations(file, buff)?;
    }
    Ok(())
}

fn convert_aseprite(
    fs_resolver: &FsResolver,
    aseprite: PathBuf,
    out: PathBuf,
) -> anyhow::Result<()> {
    let anims = lib_anim::aseprite_load::load_animations_aseprite(fs_resolver, aseprite, None)?;
    let out = File::create(out).context("open destination")?;
    serde_json::to_writer_pretty(out, &anims).context("writing to dest")
}

/// A tool for working with the game's maps.
#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// The path to the assets directory. By default,
    /// the current working directory is used.
    #[arg(long, value_name = "DIR")]
    assets: Option<PathBuf>,
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Check if an animation package satisfies all
    /// conventions.
    CheckAnims {
        /// The package to check
        #[arg(short, long, value_name = "FILE")]
        animations: PathBuf,
    },
    /// Convert an animation package into binary format.
    CompileAnims {
        /// The animation package to compile
        #[arg(short, long, value_name = "FILE")]
        animations: PathBuf,
        /// The output file
        #[arg(short, long, value_name = "FILE")]
        out: PathBuf,
    },
    /// Debug-dump a binary animation package
    DumpAnims {
        /// The animation package to dump
        #[arg(short, long, value_name = "FILE")]
        animations: PathBuf,
    },
    /// Build all animation packages in specified directory
    /// and put the compiled animations in the other. Each
    /// package called "name.json" will be turned into "name.bin".
    CompileDir {
        /// The directory to read the animation packages from
        #[arg(short, long, value_name = "DIR")]
        dir: PathBuf,
        /// The directory to put the results into
        #[arg(short, long, value_name = "DIR")]
        out: PathBuf,
    },
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
}
