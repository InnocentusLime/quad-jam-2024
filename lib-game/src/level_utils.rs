use lib_asset::{Asset, LevelId, level::LevelDef};
use strum::VariantArray;

pub fn resolve_level(s: &str) -> Option<LevelId> {
    let s = s.trim();
    for level in LevelId::VARIANTS {
        let level_name: &'static str = level.into();
        if level_name == s {
            return Some(*level);
        }
        if LevelDef::filename(*level) == s {
            return Some(*level);
        }
    }

    None
}
