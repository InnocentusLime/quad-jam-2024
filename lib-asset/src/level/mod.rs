#[cfg(feature = "dev-env")]
pub mod tiled_load;
#[cfg(feature = "dev-env")]
mod tiled_props_des;

use crate::TextureId;
use glam::Vec2;
use hashbrown::HashMap;
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

pub const TILE_SIDE: u32 = 16;

/// The root of level's definition. This type contains all information
/// required for loadin a level.
#[derive(Default, Debug, Serialize, Deserialize, PartialEq)]
pub struct LevelDef {
    /// Next level to load after it is complete.
    pub next_level: Option<String>,
    /// The level's map definition. A map is a bunch
    /// of tiles with custom data.
    pub map: MapDef,
    /// The characters placed on the map.
    pub characters: Vec<CharacterDef>,
}

/// Entity data. Currently, all entities are represented as squares.
#[derive(Default, Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
pub struct CharacterDef {
    /// Rotation angle in degrees
    pub look_angle: f32,
    /// Position in level units
    pub pos: Vec2,
    /// Entity's manifest
    #[serde(flatten)]
    pub info: CharacterInfo,
}

/// The enum containing all possible entity types for a level.
/// Your tiled project must have custom class types with their
/// names matching the variants of that type. For instance,
/// for variant [EntityInfo::Player] there must be a class
/// called `Player`, that has all corresponding fields.
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum CharacterInfo {
    Player {},
    Goal {},
    Damager {},
    Stabber {},
    BasicBullet {},
    Shooter {},
}

impl Default for CharacterInfo {
    fn default() -> Self {
        CharacterInfo::BasicBullet {}
    }
}

/// The definition of a map. Contains the tilemap
/// tiles and tile data.
#[derive(Default, Debug, Serialize, Deserialize, PartialEq)]
pub struct MapDef {
    /// Map width
    pub width: u32,
    /// Map height
    pub height: u32,
    /// Tile manifests
    pub tiles: HashMap<TileIdx, Tile>,
    /// Map's tiles in row-major order.
    /// Each index refers to an entry in `tiles`.
    pub tilemap: Vec<TileIdx>,
    /// The asset id of the atlas
    pub atlas: TextureId,
    /// Atlas margin
    pub atlas_margin: u32,
    /// Atlas spacing between tiles
    pub atlas_spacing: u32,
}

/// Tile data. Your tiled project should have a custom class
/// called `Tile`. It must have the `ty` field of type `TileTy`.
#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Tile {
    #[serde(default)]
    pub ty: TileTy,
    #[serde(default = "description_unused")]
    pub description: String,
}

fn description_unused() -> String {
    "UNUSED".to_string()
}

/// Tile type. Your tiled project should have a custom enum
/// called `TileTy` with the variants corresponding to this
/// type's variants. The default value must be `Ground`.
#[derive(Default, Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TileTy {
    #[default]
    Ground,
    Wall,
}

#[derive(
    Debug,
    Default,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    Serialize_repr,
    Deserialize_repr,
    strum::VariantNames,
    strum::FromRepr,
)]
#[repr(u32)]
pub enum TileIdx {
    #[default]
    Empty = 0,
    Unused0 = 1,
    Unused1 = 2,
    Unused2 = 3,
    Unused3 = 4,
    Unused4 = 5,
    Unused5 = 6,
    Unused6 = 7,
    Unused7 = 8,
    Unused8 = 9,
    Unused9 = 10,
    Unused10 = 11,
    Unused11 = 12,
    Unused12 = 13,
    GrassDense = 14,
    GrassSparse = 15,
    Unused13 = 16,
    BrickWallTop = 17,
    BrickWallBot = 18,
    BrickWallRight = 19,
    BrickWallLeft = 20,
    BrickWallBLCorner = 21,
    BrickWallBRCorner = 22,
    BrickWallBLCornerBot = 23,
    BrickWallBRCornerBot = 24,
    BrickWallRTCorner = 25,
    BrickWallLTCorner = 26,
    Unused14 = 27,
    Unused15 = 28,
    Unused16 = 29,
}
