#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AssetRoot {
    Base,
    Assets,
}

impl AssetRoot {
    pub fn default_path(self) -> &'static str {
        match self {
            #[cfg(not(target_family = "wasm"))]
            AssetRoot::Base => ".",
            #[cfg(target_family = "wasm")]
            AssetRoot::Base => "",
            AssetRoot::Assets => "assets",
        }
    }
}
