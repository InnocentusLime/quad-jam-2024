//! This module contains all the plain-data structures for
//! representing the level. There are several conventions for that module.
//!
//! ## Simplicity
//! The data must be as simple as possible, so it can be decoded from any
//! possible format: JSON, binary, tmx, etc.
//!
//! ## Defaults
//! Some editors do not include a field if it is set to its default. All
//! fields that come from such sources must be marked with `#[serde(default)]`
//!
//! ## Plain-Data
//! All fields are public and do not represent any complex data structure:
//! * If you have a list, use a `Vec<T>`
//! * If you have a map, use `HashMap<S, T>`
//!
//! ## Zero-dependency
//! The only okay dependency is `serde`. Existing data structures may be
//! duplicated here.
//!
//! ## Readability First
//! * All fields must be named, tuple structs are not allowed
//! * All data-containing enums must be externally tagged
//! * If there are shorter, easier to grasp variant names than the ones
//!   used in code -- use `serde(rename)`
//! * Less nesting! If the code really need some nested types,
//!   consider using `#[serde(flatten)]`
//!
//! ## Convergence
//! All level data should leave in a single type.
//!
//! ## Object Manifests are Constructors
//! Setting all potential fields of an object is counter-productive for
//! level design. We want to work with templates. A manifest for an object
//! should 1-to-1 correspond to its appropriate `spawn()` function in the
//! game code minus the `&mut World` argument.

use hashbrown::HashMap;
use lib_asset::TextureId;
use serde::{Deserialize, Serialize};

pub const TILE_SIDE: u32 = 16;

/// The root of level's definition. This type contains all information
/// required for loadin a level.
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct LevelDef {
    /// Next level to load after it is complete.
    pub next_level: Option<String>,
    /// The level's map definition. A map is a bunch
    /// of tiles with custom data.
    pub map: MapDef,
    /// The entities placed on the map.
    pub entities: Vec<EntityDef>,
}

/// Entity data. Currently, all entities are represented as squares.
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
pub struct EntityDef {
    /// Entity's transform
    pub tf: EntityPosition,
    /// Entity's width
    pub width: f32,
    /// Entity's height
    pub height: f32,
    /// Entity's manifest
    pub info: EntityInfo,
}

/// The enum containing all possible entity types for a level.
/// Your tiled project must have custom class types with their
/// names matching the variants of that type. For instance,
/// for variant [EntityInfo::Player] there must be a class
/// called `Player`, that has all corresponding fields.
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
pub enum EntityInfo {
    Player {},
    Goal {},
    Damager {},
}

/// The definition of a map. Contains the tilemap
/// tiles and tile data.
#[derive(Debug, Serialize, Deserialize, PartialEq)]
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
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct Tile {
    #[serde(default)]
    pub ty: TileTy,
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
    Serialize,
    Deserialize,
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

/// A library-agnostic transform representation.
#[derive(Default, Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
pub struct EntityPosition {
    /// Rotation angle in degrees
    pub angle: f32,
    /// Position in level units
    pub pos: Position,
}

/// A library-agnostic position representation.
#[derive(Default, Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
pub struct Position {
    pub x: f32,
    pub y: f32,
}
