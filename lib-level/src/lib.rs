pub mod binary_io;
mod level;
#[cfg(not(target_family = "wasm"))]
pub mod tiled_load;

pub use level::*;
use lib_asset::{FsResolver, declare_assets};

declare_assets!(
    LevelId(level_filename, level_path) {
        TestRoom("test_room.bin"),
    }
);

impl LevelId {
    #[cfg(not(target_family = "wasm"))]
    pub async fn load_level(self, resolver: &FsResolver) -> anyhow::Result<LevelDef> {
        use std::path::PathBuf;

        let mut filename = PathBuf::from(self.get_filename());
        filename.set_extension("tmx");

        tiled_load::load_level(resolver, resolver.tiled_path(filename))
    }

    #[cfg(target_family = "wasm")]
    pub async fn load_level(self, resolver: &FsResolver) -> anyhow::Result<LevelDef> {
        use macroquad::prelude::*;
        let path = self.resolve(resolver);
        let data = load_file(path.to_str().unwrap()).await?;
        binary_io::load_from_memory(&data)
    }
}
