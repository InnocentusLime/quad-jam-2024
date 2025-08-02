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
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct LevelDef {
    pub next_level: Option<String>,
    pub map: MapDef,
    pub entities: Vec<EntityDef>,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct EntityDef {
    pub tf: Transform,
    pub width: f32,
    pub height: f32,
    pub info: EntityInfo,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum EntityInfo {
    Player {},
    Goal {},
    Damager {},
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MapDef {
    /// Map width
    pub width: u32,
    /// Map height
    pub height: u32,
    /// Tile manifests
    pub tiles: HashMap<u32, Tile>,
    /// Map tiled in row-major order
    pub tilemap: Vec<u32>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Tile {
    #[serde(default)]
    pub ty: TileTy,
}

#[derive(Default, Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TileTy {
    #[default]
    Ground,
    Wall,
}

#[derive(Default, Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Transform {
    /// Rotation angle in degrees
    pub angle: f32,
    /// Position in level units
    pub pos: Position,
}

#[derive(Default, Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Position {
    pub x: f32,
    pub y: f32,
}
