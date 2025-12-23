use std::fs::{self, File};
use std::{path::PathBuf, process::ExitCode};

use anyhow::Context;
use clap::{Parser, Subcommand};
use hashbrown::HashMap;
use lib_asset::*;
use postcard::ser_flavors::io::WriteFlavor;

pub fn run() -> ExitCode {
    let cli = Cli::parse();
    let mut resolver = FsResolver::new();
    if let Some(asset_dir) = cli.assets {
        resolver.set_root(AssetRoot::Default, asset_dir);
    }

    let result = match cli.command {
        Commands::CheckAnims { animations } => check_animations(animations),
        Commands::CompileAnims { animations, out } => compile_animations(animations, out),
        Commands::DumpAnims { animations } => dump_animations(animations),
        Commands::CompileAnimsDir { dir, out } => compile_anims_dir(dir, out),
        Commands::ConvertAseprite { aseprite, out } => convert_aseprite(&resolver, aseprite, out),
        Commands::CheckMap { map } => check_map(&resolver, map),
        Commands::CompileMap { map, out } => compile_map(&resolver, map, out),
        Commands::DumpMap { map } => dump_map(map),
        Commands::CompileMapsDir { dir, out } => compile_maps_dir(&resolver, dir, out),
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

    animation::aseprite_load::load_animations_project(animations)?;
    Ok(())
}

fn compile_animations(animations: PathBuf, out: PathBuf) -> anyhow::Result<()> {
    println!("Compiling {animations:?} into {out:?}");

    let anims =
        animation::aseprite_load::load_animations_project(animations).context("loading package")?;
    let out = fs::File::create(out).context("opening the output")?;

    postcard::serialize_with_flavor(&anims, WriteFlavor::new(out))
        .context("writing the package")?;
    Ok(())
}

fn dump_animations(animations: PathBuf) -> anyhow::Result<()> {
    let anims_data = fs::read(animations)?;

    let anims: HashMap<animation::AnimationId, animation::Animation>;
    anims = postcard::from_bytes(&anims_data)?;

    println!("{:?}", anims);
    Ok(())
}

fn compile_anims_dir(dir: PathBuf, out: PathBuf) -> anyhow::Result<()> {
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
    let anims = animation::aseprite_load::load_animations_aseprite(fs_resolver, aseprite, None)?;
    let out = File::create(out).context("open destination")?;
    serde_json::to_writer_pretty(out, &anims).context("writing to dest")
}

fn check_map(resolver: &FsResolver, map: PathBuf) -> anyhow::Result<()> {
    println!("Checking {map:?}");

    level::tiled_load::load_level(resolver, map)?;
    Ok(())
}

fn compile_map(resolver: &FsResolver, map: PathBuf, out: PathBuf) -> anyhow::Result<()> {
    println!("Compiling {map:?} into {out:?}");

    let level = level::tiled_load::load_level(resolver, map).context("loading map")?;
    let out = fs::File::create(out).context("opening the output")?;

    postcard::serialize_with_flavor(&level, WriteFlavor::new(out)).context("writing the level")?;
    Ok(())
}

fn dump_map(map: PathBuf) -> anyhow::Result<()> {
    let level_data = fs::read(map)?;
    let level: level::LevelDef;
    level = postcard::from_bytes(&level_data)?;
    println!("{:?}", level);
    Ok(())
}

fn compile_maps_dir(resolver: &FsResolver, dir: PathBuf, out: PathBuf) -> anyhow::Result<()> {
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
        compile_map(resolver, file, buff)?;
    }
    Ok(())
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
    CompileAnimsDir {
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
    CompileMapsDir {
        /// The directory to read the maps from
        #[arg(short, long, value_name = "DIR")]
        dir: PathBuf,
        /// The directory to put the results into
        #[arg(short, long, value_name = "DIR")]
        out: PathBuf,
    },
}
