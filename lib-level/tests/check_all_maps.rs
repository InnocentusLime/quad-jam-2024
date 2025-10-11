use std::fs;

use lib_asset::FsResolver;

#[test]
fn test_all_maps() {
    let mut resolver = FsResolver::new();
    resolver.set_assets_dir("../assets").unwrap();

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
        lib_level::tiled_load::load_level(&resolver, &file).unwrap();
    }
}

#[test]
fn test_all_maps_sanity() {
    let mut resolver = FsResolver::new();
    resolver.set_assets_dir("../assets").unwrap();

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
        let level = lib_level::tiled_load::load_level(&resolver, &file).unwrap();

        let mut out = file.clone();
        out.set_extension("bin");
        let out_file = fs::File::create(&out).unwrap();
        lib_level::binary_io::compile::write_level(&level, out_file).unwrap();

        let level_data = fs::read(out).unwrap();
        let restored_level = lib_level::binary_io::load_from_memory(&level_data).unwrap();

        assert_eq!(level, restored_level);
    }
}
