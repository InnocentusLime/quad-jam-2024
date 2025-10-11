use std::{ffi::OsStr, fmt::Debug, path::{absolute, Path, PathBuf}};

use macroquad::{text::{load_ttf_font, Font}, texture::{load_texture, Texture2D}};
use serde::{Deserialize, Serialize};

fn asset_root() -> PathBuf {
    absolute("./assets").unwrap()
}

fn path_to_id<T: Copy>(files: &[(T, &'static str)], path: &Path) -> anyhow::Result<T> {
    let root = asset_root();
    let got_file = path.strip_prefix(root).unwrap();
    let item = files.iter()
        .find(|(_, file)| got_file.as_os_str() == OsStr::new(file));

    match item {
        Some((id, _)) => Ok(*id),
        None => anyhow::bail!("{path:?} does not correspond to any asset"),
    }
}

fn id_to_path<T: Eq + Debug>(files: &[(T, &'static str)], the_id: T) -> anyhow::Result<PathBuf> {
    let mut root = asset_root();
    let item = files
        .iter()
        .find(|(id, _)| *id == the_id);

    match item {
        Some((_, filename)) => {
            root.push(*filename);
            Ok(root)
        },
        None => anyhow::bail!("{the_id:?} doesn't have a path")
    }
}

macro_rules! declare_assets {
    ($asset_ty:ident {
        $($id_ident:ident($path:literal),)+
    }) => {
        #[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
        pub enum $asset_ty {
            $($id_ident,)+
        }

        impl $asset_ty {
            fn files() -> &'static [($asset_ty, &'static str)] {
                &[$(($asset_ty::$id_ident, $path)),+]                
            }

            pub fn inverse_resolve(path: &std::path::Path) -> anyhow::Result<$asset_ty> {
                path_to_id($asset_ty::files(), path)
            }

            pub fn resolve(self) -> anyhow::Result<std::path::PathBuf> {
                id_to_path($asset_ty::files(), self)
            }
        }
    };
}

declare_assets!(
    TextureId {
        BunnyAtlas("bnuuy.png"),
        WorldAtlas("world.png"),
    }
);

impl TextureId {
    pub async fn load_texture(self) -> anyhow::Result<Texture2D> {
        let path = self.resolve()?;
        let tex = load_texture(&path.as_os_str().to_string_lossy()).await?;
        Ok(tex)
    }
}

declare_assets!(
    FontId {
        Quaver("quaver.ttf"),
    }
);

impl FontId{
    pub async fn load_font(self) -> anyhow::Result<Font> {
        let path = self.resolve()?;
        let font = load_ttf_font(&path.as_os_str().to_string_lossy()).await?;
        Ok(font)
    }
}