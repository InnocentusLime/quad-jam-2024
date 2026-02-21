mod asset_roots;
mod containers;

pub mod animation_manifest;
pub mod gamecfg;
pub mod level;

pub use asset_roots::*;
pub use containers::*;
pub use gamecfg::*;

use hashbrown::HashMap;

use std::path::{Path, PathBuf};

pub struct FsResolver {
    roots: HashMap<AssetRoot, PathBuf>,
}

impl FsResolver {
    pub fn new() -> Self {
        let mut roots = HashMap::new();
        roots.insert(AssetRoot::Base, AssetRoot::Base.default_path().into());
        roots.insert(AssetRoot::Assets, AssetRoot::Assets.default_path().into());
        roots.insert(
            AssetRoot::AsepriteProjectRoot,
            AssetRoot::AsepriteProjectRoot.default_path().into(),
        );
        roots.insert(
            AssetRoot::TiledProjectRoot,
            AssetRoot::TiledProjectRoot.default_path().into(),
        );
        FsResolver { roots }
    }

    pub fn set_root(&mut self, id: AssetRoot, dir: impl AsRef<Path>) {
        self.roots.insert(id, dir.as_ref().to_path_buf());
    }

    fn get_dir(&self, root: AssetRoot) -> impl AsRef<Path> {
        let mut path = PathBuf::new();
        path.push(&self.roots[&AssetRoot::Base]);
        if root != AssetRoot::Base {
            path.push(&self.roots[&root]);
        }
        #[cfg(not(target_family = "wasm"))]
        match std::fs::canonicalize(&path) {
            Ok(x) => x,
            Err(e) => panic!("Failed to resolve {path:?}: {e}"),
        }
        #[cfg(target_family = "wasm")]
        path
    }

    pub fn get_path(&self, root: AssetRoot, filename: impl AsRef<Path>) -> PathBuf {
        PathBuf::from_iter([self.get_dir(root).as_ref(), filename.as_ref()])
    }

    #[cfg(feature = "dev-env")]
    pub(crate) fn get_filename(&self, root: AssetRoot, path: &Path) -> anyhow::Result<PathBuf> {
        use anyhow::Context;

        let path = match std::fs::canonicalize(&path) {
            Ok(x) => x,
            Err(e) => return Err(e).context(format!("canonicalizing {path:?}")),
        };

        let dir = self.get_dir(root);
        let dir = dir.as_ref();
        path.strip_prefix(dir)
            .with_context(|| format!("Resolving against {dir:?}"))
            .map(|x| x.to_path_buf())
    }
}

impl Default for FsResolver {
    fn default() -> Self {
        FsResolver::new()
    }
}
