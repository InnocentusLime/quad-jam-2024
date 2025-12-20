mod asset_roots;
mod assets;

pub mod animation;
pub mod level;

pub use asset_roots::*;
pub use assets::*;

use anyhow::Context;
use hashbrown::HashMap;
use strum::VariantArray;

use std::path::{Path, PathBuf};

pub struct FsResolver {
    roots: HashMap<AssetRoot, PathBuf>,
}

impl FsResolver {
    pub fn new() -> Self {
        let roots = AssetRoot::VARIANTS
            .iter()
            .map(|x| (*x, x.default_path().into()))
            .collect();
        FsResolver { roots }
    }

    pub fn inverse_resolve<A: Asset>(&self, path: &Path) -> anyhow::Result<A::AssetId> {
        let got_file = self.get_filename(A::ROOT, path)?;
        let item = <A::AssetId as VariantArray>::VARIANTS
            .iter()
            .map(|id| (id, A::filename(*id)))
            .find(|(_, file)| got_file.as_os_str() == std::ffi::OsStr::new(file));

        match item {
            Some((id, _)) => Ok(*id),
            None => anyhow::bail!("{path:?} does not correspond to any asset"),
        }
    }

    pub async fn load<A: Asset>(&self, id: A::AssetId) -> anyhow::Result<A> {
        A::load(self, &self.get_path(A::ROOT, A::filename(id))).await
    }

    pub fn set_root(&mut self, id: AssetRoot, dir: impl AsRef<Path>) {
        self.roots.insert(id, dir.as_ref().to_path_buf());
    }

    #[cfg(not(target_family = "wasm"))]
    fn get_dir(&self, root: AssetRoot) -> impl AsRef<Path> {
        let mut path = PathBuf::from("./");
        path.push(&self.roots[&root]);
        match std::fs::canonicalize(&path) {
            Ok(x) => x,
            Err(e) => panic!("Failed to resolve {path:?}: {e}"),
        }
    }

    #[cfg(target_family = "wasm")]
    fn get_dir(&self, root: AssetRoot) -> impl AsRef<Path> {
        &self.roots[&root]
    }

    fn get_filename<'a>(&self, root: AssetRoot, path: &'a Path) -> anyhow::Result<&'a Path> {
        let dir = self.get_dir(root);
        let dir = dir.as_ref();
        path.strip_prefix(dir)
            .with_context(|| format!("Resolving against {dir:?}"))
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

pub trait Asset: Sized {
    type AssetId: Copy + strum::VariantArray;
    const ROOT: AssetRoot;

    fn load(
        resolver: &FsResolver,
        path: &Path,
    ) -> impl Future<Output = anyhow::Result<Self>> + Send;

    fn filename(id: Self::AssetId) -> &'static str;
}
