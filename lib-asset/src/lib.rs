mod asset_roots;
mod containers;
mod manager;
mod prefab;

pub use asset_roots::*;
pub use containers::*;
pub use manager::*;
pub use prefab::*;

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
}

impl Default for FsResolver {
    fn default() -> Self {
        FsResolver::new()
    }
}
