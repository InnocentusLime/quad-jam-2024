#[derive(
    Debug,
    Clone,
    Copy,
    serde::Serialize,
    serde::Deserialize,
    PartialEq,
    Eq,
    Hash,
    strum::IntoStaticStr,
    strum::VariantArray,
)]
pub enum AssetRoot {
    Base,
    Assets,
    Animations,
    Levels,
    AsepriteProjectRoot,
    AnimationsProjectRoot,
    TiledProjectRoot,
}

impl AssetRoot {
    pub fn default_path(self) -> &'static str {
        match self {
            #[cfg(not(target_family = "wasm"))]
            AssetRoot::Base => ".",
            #[cfg(target_family = "wasm")]
            AssetRoot::Base => "",
            AssetRoot::Assets => "assets",
            AssetRoot::Animations => "animations",
            AssetRoot::Levels => "levels",
            AssetRoot::AsepriteProjectRoot => "project-aseprite",
            AssetRoot::AnimationsProjectRoot => "project-animations",
            AssetRoot::TiledProjectRoot => "project-tiled",
        }
    }
}
