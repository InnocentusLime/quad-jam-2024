use std::path::{Path, PathBuf};

use hashbrown::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct AssetKey(usize);

pub const INVALID_ASSET: AssetKey = AssetKey(usize::MAX);

impl Default for AssetKey {
    fn default() -> Self {
        INVALID_ASSET
    }
}

pub struct AssetContainer<T> {
    storage: slab::Slab<T>,
    path_lookup: HashMap<PathBuf, usize>,
}

impl<T> AssetContainer<T> {
    pub fn new() -> Self {
        Self {
            storage: slab::Slab::new(),
            path_lookup: HashMap::new(),
        }
    }

    pub fn get(&self, key: AssetKey) -> Option<&T> {
        self.storage.get(key.0)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&Path, AssetKey)> {
        self.path_lookup
            .iter()
            .map(|(k, v)| (k.as_ref(), AssetKey(*v)))
    }

    pub fn inverse_resolve(&'_ self, search_key: AssetKey) -> &'_ Path {
        self.path_lookup
            .iter()
            .find(|(_, key)| **key == search_key.0)
            .map(|(path, _)| path)
            .expect("used dead AssetKey")
    }

    pub fn resolve(&self, path: impl AsRef<Path>) -> Option<AssetKey> {
        self.resolve_(path.as_ref())
    }

    fn resolve_(&self, path: &Path) -> Option<AssetKey> {
        self.path_lookup.get(path).copied().map(AssetKey)
    }

    pub fn insert(&mut self, source: impl AsRef<Path>, asset: T) -> AssetKey {
        self.insert_(source.as_ref().to_path_buf(), asset)
    }

    fn insert_(&mut self, source: PathBuf, asset: T) -> AssetKey {
        match self.path_lookup.get(&source).copied() {
            Some(entry) => {
                self.storage[entry] = asset;
                AssetKey(entry)
            }
            None => {
                let entry = self.storage.insert(asset);
                self.path_lookup.insert(source, entry);
                AssetKey(entry)
            }
        }
    }
}
