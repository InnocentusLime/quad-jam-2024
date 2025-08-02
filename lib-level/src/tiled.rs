//! This module contains logic for decoding [LevelDef] from a `Tiled`
//! map file.

use std::ops::Deref;

use hashbrown::HashMap;
use thiserror::Error;

use crate::level::{LevelDef, MapDef, Tile};
use crate::tiled_props_des::{DeserializerError, from_properties};

const TILE_SIDE: u32 = 16;
static REQUIRED_TILED_VERSION: &'static str = "1.10";
static OBJECT_LAYER: &'static str = "Actors";
static WORLD_LAYER: &'static str = "World";
static TILE_CLASS: &'static str = "Tile";

#[derive(Debug, Error)]
pub enum LoadFromTiledError {
    #[error("Unsupported tiled map version: {0:?}")]
    UnsupportedVersion(String),
    #[error("Incorrect tile height: expected {TILE_SIDE:}, found {0:}")]
    TileHeight(u32),
    #[error("Incorrect tile width: expected {TILE_SIDE:}, found {0:}")]
    TileWidth(u32),
    #[error("Duplicate layer: {0:?}")]
    DuplicateLayer(String),
    #[error("Unknown layer: {0:?}")]
    UnknownLayer(String),
    #[error("World layer ({WORLD_LAYER:?}) not found")]
    WorldLayerAbsent,
    #[error("Expected layer {WORLD_LAYER:?} to be a tile layer")]
    WorldLayerNotTileLayer,
    #[error("Expected world layer to be finite")]
    WorldLayerInfinite,
    #[error("Incorrect layer {WORLD_LAYER:?} width: expected {expected:}, found {found:}")]
    WorldLayerWidth { expected: u32, found: u32 },
    #[error("Incorrect layer {WORLD_LAYER:?} height: expected {expected:}, found {found:}")]
    WorldLayerHeight { expected: u32, found: u32 },
    #[error("Expected the map to have 1 tileset. Found {0:?}")]
    UnexpectedTilesetAmount(usize),
    #[error("Image collection based tilesets are not supported")]
    MapTilesetIrregular,
    #[error(
        "Tileset {tileset:?}, tile {tile_idx:}: expected class to be {TILE_CLASS:?}, found {found:?}"
    )]
    UnknownTileClass {
        tileset: String,
        tile_idx: u32,
        found: String,
    },
    #[error("Tileset {tileset:?}, tile {tile_idx:}: failed to deserialize properties")]
    TileDeserError {
        tileset: String,
        tile_idx: u32,
        #[source]
        reason: DeserializerError,
    },
}

pub fn load_level_from_map(map: &tiled::Map) -> Result<LevelDef, LoadFromTiledError> {
    if map.version() != REQUIRED_TILED_VERSION {
        return Err(LoadFromTiledError::UnsupportedVersion(
            map.version().to_string(),
        ));
    }
    if map.tile_height != TILE_SIDE {
        return Err(LoadFromTiledError::TileHeight(map.tile_height));
    }
    if map.tile_width != TILE_SIDE {
        return Err(LoadFromTiledError::TileHeight(map.tile_width));
    }
    if map.tilesets().len() != 1 {
        return Err(LoadFromTiledError::UnexpectedTilesetAmount(
            map.tilesets().len(),
        ));
    }
    if map.tilesets()[0].image.is_none() {
        return Err(LoadFromTiledError::MapTilesetIrregular);
    }

    let width = map.width;
    let height = map.height;
    let mut layers_by_name = HashMap::<String, tiled::Layer>::new();
    for layer in map.layers() {
        let name = layer.name.clone();
        if name != OBJECT_LAYER && name != WORLD_LAYER {
            return Err(LoadFromTiledError::UnknownLayer(name));
        }

        let old = layers_by_name.insert(name.clone(), layer);
        if old.is_some() {
            return Err(LoadFromTiledError::DuplicateLayer(name));
        }
    }

    let Some(mapdef_layer) = layers_by_name.get(WORLD_LAYER) else {
        return Err(LoadFromTiledError::WorldLayerAbsent);
    };
    let mapdef = load_mapdef_from_layer(mapdef_layer, width, height)?;

    todo!()
}

fn load_mapdef_from_layer(
    layer: &tiled::Layer,
    map_width: u32,
    map_height: u32,
) -> Result<MapDef, LoadFromTiledError> {
    let Some(tile_layer) = layer.as_tile_layer() else {
        return Err(LoadFromTiledError::WorldLayerNotTileLayer);
    };
    let Some(layer_width) = tile_layer.width() else {
        return Err(LoadFromTiledError::WorldLayerInfinite);
    };
    let Some(layer_height) = tile_layer.height() else {
        return Err(LoadFromTiledError::WorldLayerInfinite);
    };
    if layer_width != map_width {
        return Err(LoadFromTiledError::WorldLayerWidth {
            expected: map_width,
            found: layer_width,
        });
    }
    if layer_height != map_height {
        return Err(LoadFromTiledError::WorldLayerHeight {
            expected: map_height,
            found: layer_height,
        });
    }

    let tileset = layer.map().tilesets()[0].deref();
    let mut tiles = HashMap::<u32, Tile>::new();
    for (tile_idx, tile_data) in tileset.tiles() {
        let class = tile_data.user_type.as_deref().unwrap_or("");
        if class != TILE_CLASS {
            return Err(LoadFromTiledError::UnknownTileClass {
                tileset: tileset.name.clone(),
                tile_idx,
                found: class.to_string(),
            });
        }
        let tile = from_properties(TILE_CLASS, &tile_data.properties).map_err(|reason| {
            LoadFromTiledError::TileDeserError {
                tileset: tileset.name.clone(),
                tile_idx,
                reason,
            }
        })?;

        tiles.insert(tile_idx, tile);
    }

    let mut tilemap = Vec::<u32>::with_capacity((layer_width * layer_height) as usize);
    for y in 0..layer_height {
        for x in 0..layer_width {
            let tile_instance = tile_layer.get_tile(x as i32, y as i32).unwrap();
            tilemap.push(tile_instance.id());
        }
    }

    Ok(MapDef {
        width: layer_width,
        height: layer_height,
        tiles,
        tilemap,
    })
}
