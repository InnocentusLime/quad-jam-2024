#![cfg(feature = "dev-env")]

use std::fs;

use lib_asset::*;
use postcard::ser_flavors::io::WriteFlavor;

#[test]
fn test_all_maps() {
    let mut resolver = FsResolver::new();
    resolver.set_root(AssetRoot::Default, "../assets");

    let dir = fs::read_dir("../project-tiled").unwrap();
    for file in dir {
        let file = file.unwrap();
        let file = file.path();

        let Some(ext) = file.extension() else {
            continue;
        };
        if ext != "tmx" {
            continue;
        };

        println!("Checking {file:?}");
        level::tiled_load::load_level(&resolver, &file).unwrap();
    }
}

#[test]
fn test_all_maps_sanity() {
    let mut resolver = FsResolver::new();
    resolver.set_root(AssetRoot::Default, "../assets");

    let dir = fs::read_dir("../project-tiled").unwrap();
    for file in dir {
        let file = file.unwrap();
        let file = file.path();

        let Some(ext) = file.extension() else {
            continue;
        };
        if ext != "tmx" {
            continue;
        };

        println!("Checking {file:?}");
        let level = level::tiled_load::load_level(&resolver, &file).unwrap();

        let mut out = file.clone();
        out.set_extension("bin");
        let out_file = fs::File::create(&out).unwrap();
        postcard::serialize_with_flavor(&level, WriteFlavor::new(out_file)).unwrap();

        let level_data = fs::read(out).unwrap();
        let restored_level: level::LevelDef = postcard::from_bytes(&level_data).unwrap();

        assert_eq!(level, restored_level);
    }
}
