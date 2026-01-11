use std::fs::{self, File};
use std::io::stdout;
use std::{path::PathBuf, process::ExitCode};

use anyhow::Context;
use clap::{Parser, Subcommand};
use lib_asset::animation::AnimationPack;
use lib_asset::level::LevelDef;
use lib_asset::*;
use postcard::ser_flavors::io::WriteFlavor;

pub fn run() -> ExitCode {
    let cli = Cli::parse();
    let mut resolver = FsResolver::new();
    if let Some(base_dir) = cli.base {
        resolver.set_root(AssetRoot::Base, base_dir);
    }

    let result = match cli.command {
        Commands::CompileCfg { config, out } => compile_impl::<GameCfg>(&resolver, config, out),
        Commands::CheckCfg { configs } => check_impl::<GameCfg>(&resolver, configs),
        Commands::DumpCfg { config } => dump_impl::<GameCfg>(config),
        Commands::CheckAnims { animations } => check_impl::<AnimationPack>(&resolver, animations),
        Commands::CompileAnims { animations, out } => {
            compile_impl::<AnimationPack>(&resolver, animations, out)
        }
        Commands::DumpAnims { animations } => dump_impl::<AnimationPack>(animations),
        Commands::CompileAnimsDir { dir, out } => {
            compile_dir_impl::<AnimationPack>(&resolver, "json", dir, out)
        }
        Commands::ConvertAseprite { aseprite, out } => convert_aseprite(&resolver, aseprite, out),
        Commands::CheckMap { maps } => check_impl::<LevelDef>(&resolver, maps),
        Commands::CompileMap { map, out } => compile_impl::<LevelDef>(&resolver, map, out),
        Commands::DumpMap { map } => dump_impl::<LevelDef>(map),
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

fn convert_aseprite(
    fs_resolver: &FsResolver,
    aseprite: PathBuf,
    out: PathBuf,
) -> anyhow::Result<()> {
    let anims = animation::aseprite_load::load_animations_aseprite(fs_resolver, aseprite, None)?;
    let out = File::create(out).context("open destination")?;
    serde_json::to_writer_pretty(out, &anims).context("writing to dest")
}

fn check_impl<T: DevableAsset>(resolver: &FsResolver, assets: Vec<PathBuf>) -> anyhow::Result<()> {
    for asset in assets {
        println!("Checking {asset:?}");
        T::load_dev(resolver, &asset)?;
    }
    Ok(())
}

fn dump_impl<T: for<'a> serde::Deserialize<'a> + serde::Serialize>(
    asset: PathBuf,
) -> anyhow::Result<()> {
    let data = fs::read(asset)?;
    let data = postcard::from_bytes(&data)?;
    let mut stdout = stdout().lock();
    serde_json::to_writer_pretty::<_, T>(&mut stdout, &data)?;
    Ok(())
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
        buff.set_extension("bin");
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
    postcard::serialize_with_flavor(&val, WriteFlavor::new(out)).context("compiling")?;
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
    /// Compile a game config.
    CompileCfg {
        /// The config to compile
        #[arg(short, long, value_name = "FILE")]
        config: PathBuf,
        /// The output file
        #[arg(short, long, value_name = "FILE")]
        out: PathBuf,
    },
    /// Checks a game config.
    CheckCfg {
        /// The config to check
        #[arg(value_name = "FILE")]
        configs: Vec<PathBuf>,
    },
    /// Dumps config contents.
    DumpCfg {
        /// The config to dump
        #[arg(short, long, value_name = "FILE")]
        config: PathBuf,
    },
    /// Check if an animation package satisfies all
    /// conventions.
    CheckAnims {
        /// The package to check
        #[arg(value_name = "FILE")]
        animations: Vec<PathBuf>,
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
        #[arg(value_name = "FILE")]
        maps: Vec<PathBuf>,
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
