use std::{
    fmt::Debug,
    path::{Path, PathBuf},
};

use anyhow::Context;
use macroquad::{
    text::{Font, load_ttf_font},
    texture::{Texture2D, load_texture},
};

#[doc(hidden)]
macro_rules! impl_resource_methods {
    (
        $field:ident,
        $dir_def:ident = $dir_path:literal,
        $get_dir:ident,
        $set_dir:ident,
        $get_filename:ident,
        $get_path:ident
    ) => {
        fn $dir_def() -> PathBuf {
            PathBuf::from($dir_path)
        }

        #[cfg(not(target_family = "wasm"))]
        fn $get_dir(&self) -> impl AsRef<Path> {
            let mut path = PathBuf::from("./");
            path.push(&self.$field);
            match std::fs::canonicalize(&path) {
                Ok(x) => x,
                Err(e) => panic!("Failed to resolve {path:?}: {e}"),
            }
        }

        #[cfg(target_family = "wasm")]
        fn $get_dir(&self) -> impl AsRef<Path> {
            &self.$field
        }

        pub fn $set_dir(&mut self, dir: impl AsRef<Path>) -> anyhow::Result<()> {
            self.$field = PathBuf::from(dir.as_ref());
            Ok(())
        }

        pub fn $get_filename<'a>(&self, path: &'a Path) -> anyhow::Result<&'a Path> {
            path.strip_prefix(self.$get_dir())
                .with_context(|| format!("Resolving against {:?}", self.$field))
        }

        pub fn $get_path(&self, filename: impl AsRef<Path>) -> PathBuf {
            let filename = filename.as_ref();
            PathBuf::from_iter([self.$get_dir().as_ref(), filename])
        }
    };
}

pub struct FsResolver {
    assets_dir: PathBuf,
    aseprite_dir: PathBuf,
    animations_pack_dir: PathBuf,
}

impl FsResolver {
    pub fn new() -> Self {
        FsResolver {
            assets_dir: Self::assets_dir_default(),
            aseprite_dir: Self::aseprite_dir_default(),
            animations_pack_dir: Self::animations_pack_dir_default(),
        }
    }

    impl_resource_methods!(
        assets_dir,
        assets_dir_default = "assets",
        get_assets_dir,
        set_assets_dir,
        asset_filename,
        asset_path
    );

    impl_resource_methods!(
        animations_pack_dir,
        animations_pack_dir_default = "animations",
        get_animations_pack_dir,
        set_animations_pack_dir,
        animation_pack_filename,
        animation_pack_path
    );

    impl_resource_methods!(
        aseprite_dir,
        aseprite_dir_default = "project-aseprite",
        get_aseprite_dir,
        set_aserpite_dir,
        aseprite_filename,
        aseprite_path
    );
}

#[macro_export]
macro_rules! declare_assets {
    ($asset_ty:ident($resolver_inv_call:ident, $resolver_call:ident) {
        $($id_ident:ident($path:literal),)+
    }) => {
        #[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize, PartialEq, Eq, Hash)]
        pub enum $asset_ty {
            $($id_ident,)+
        }

        impl $asset_ty {
            pub fn filenames() -> &'static [($asset_ty, &'static str)] {
                &[$(($asset_ty::$id_ident, $path)),+]
            }

            pub fn get_filename(self) -> &'static str {
                match self {
                    $( $asset_ty::$id_ident => $path ),+
                }
            }

            pub fn inverse_resolve(resolver: &$crate::FsResolver, path: &std::path::Path) -> anyhow::Result<$asset_ty> {
                let got_file = resolver.$resolver_inv_call(path)?;
                let item = Self::filenames().iter()
                    .find(|(_, file)| got_file.as_os_str() == std::ffi::OsStr::new(file));

                match item {
                    Some((id, _)) => Ok(*id),
                    None => anyhow::bail!("{path:?} does not correspond to any asset"),
                }
            }

            pub fn resolve(self, resolver: &$crate::FsResolver) -> std::path::PathBuf {
                resolver.$resolver_call(self.get_filename())
            }
        }
    };
}

declare_assets!(
    TextureId(asset_filename, asset_path) {
        BunnyAtlas("bnuuy.png"),
        WorldAtlas("world.png"),
    }
);

impl TextureId {
    pub async fn load_texture(self, resolver: &FsResolver) -> anyhow::Result<Texture2D> {
        let path = self.resolve(resolver);
        let tex = load_texture(&path.as_os_str().to_string_lossy()).await?;
        Ok(tex)
    }
}

declare_assets!(
    FontId(asset_filename, asset_path) {
        Quaver("quaver.ttf"),
    }
);

impl FontId {
    pub async fn load_font(self, resolver: &FsResolver) -> anyhow::Result<Font> {
        let path = self.resolve(resolver);
        let font = load_ttf_font(&path.as_os_str().to_string_lossy()).await?;
        Ok(font)
    }
}
