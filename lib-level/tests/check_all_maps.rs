use std::fs;

#[test]
fn test_all_maps() {
    let dir = fs::read_dir("../tiled-project").unwrap();
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
        lib_level::tiled_load::load_level(&file).unwrap();
    }
}

#[test]
fn test_all_maps_sanity() {
    let dir = fs::read_dir("../tiled-project").unwrap();
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
        let level = lib_level::tiled_load::load_level(&file).unwrap();

        let mut out = file.clone();
        out.set_extension("bin");
        let out_file = fs::File::create(&out).unwrap();
        lib_level::binary_io::compile::write_level(&level, out_file).unwrap();

        let level_data = fs::read(out).unwrap();
        let restored_level = lib_level::binary_io::load_from_memory(&level_data).unwrap();

        assert_eq!(level, restored_level);
    }
}
