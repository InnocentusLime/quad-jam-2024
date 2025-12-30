//! This module contains logic for decoding [LevelDef] from a `Tiled`
//! map file.

use std::fs;
use std::path::Path;

use crate::{FsResolver, TextureId};
use anyhow::Context;
use hashbrown::HashMap;
use macroquad::texture::Texture2D;

use super::tiled_props_des::from_properties;
use crate::level::*;

/// Load a level by path through tield. For internal use only.
pub fn load_level(resolver: &FsResolver, path: impl AsRef<Path>) -> anyhow::Result<LevelDef> {
    let mut loader = tiled::Loader::new();
    let map = loader.load_tmx_map(path)?;
    let level = load_level_from_map(resolver, &map)?;

    Ok(level)
}

fn load_level_from_map(resolver: &FsResolver, map: &tiled::Map) -> anyhow::Result<LevelDef> {
    anyhow::ensure!(
        map.version() == REQUIRED_TILED_VERSION,
        "Unsupported tiled map version: {:?}",
        map.version(),
    );
    anyhow::ensure!(
        map.tile_height == TILE_SIDE,
        "Incorrect tile height: expected {TILE_SIDE:}, found {0:}",
        map.tile_height,
    );
    anyhow::ensure!(
        map.tile_width == TILE_SIDE,
        "Incorrect tile width: expected {TILE_SIDE:}, found {0:}",
        map.tile_width,
    );
    anyhow::ensure!(
        map.tilesets().len() == 1,
        "Expected the map to have 1 tileset. Found {0:?}",
        map.tilesets().len(),
    );

    let width = map.width;
    let height = map.height;
    let mut layers_by_name = HashMap::<String, tiled::Layer>::new();
    for layer in map.layers() {
        let name = layer.name.clone();
        anyhow::ensure!(
            name == OBJECT_LAYER || name == WORLD_LAYER,
            "Unknown layer: {name:?}",
        );

        let old = layers_by_name.insert(name.clone(), layer);
        anyhow::ensure!(old.is_none(), "Duplicate layer: {name:?}",);
    }

    let Some(mapdef_layer) = layers_by_name.get(WORLD_LAYER) else {
        anyhow::bail!("World layer {WORLD_LAYER:?} not found");
    };
    let map = load_mapdef_from_layer(resolver, mapdef_layer, width, height)?;

    let Some(entitydefs_layer) = layers_by_name.get(OBJECT_LAYER) else {
        anyhow::bail!("Object layer {OBJECT_LAYER:?} not found");
    };
    let entities = load_entity_defs_from_object_layer(entitydefs_layer)?;

    Ok(LevelDef {
        next_level: None,
        map,
        entities,
    })
}

fn load_mapdef_from_layer(
    resolver: &FsResolver,
    layer: &tiled::Layer,
    map_width: u32,
    map_height: u32,
) -> anyhow::Result<MapDef> {
    let Some(tile_layer) = layer.as_tile_layer() else {
        anyhow::bail!("Expected layer {WORLD_LAYER:?} to be a tile layer");
    };
    let Some(layer_width) = tile_layer.width() else {
        anyhow::bail!("Expected layer {WORLD_LAYER:?} to be a finite tile layer");
    };
    let Some(layer_height) = tile_layer.height() else {
        anyhow::bail!("Expected layer {WORLD_LAYER:?} to be a finite tile layer");
    };
    anyhow::ensure!(
        layer_width == map_width,
        "Incorrect layer {WORLD_LAYER:?} width: expected {map_width}, found {layer_width}",
    );
    anyhow::ensure!(
        layer_height == map_height,
        "Incorrect layer {WORLD_LAYER:?} height: expected {map_height}, found {layer_height}",
    );

    let tileset = &*layer.map().tilesets()[0];
    let mut tiles = HashMap::<_, Tile>::new();
    for (tile_idx, tile_data) in tileset.tiles() {
        let parsed_tile_idx = TileIdx::from_repr(tile_idx).ok_or_else(|| {
            anyhow::anyhow!("Tileset {:?}, tile {tile_idx:}: unknown tile", tileset.name,)
        })?;
        let tile = from_properties(TILE_CLASS, &tile_data.properties)
            .with_context(|| format!("Tileset {:?}, tile {tile_idx}", tileset.name))?;
        tiles.insert(parsed_tile_idx, tile);
    }
    let Some(tileset_atlas) = tileset.image.as_ref() else {
        anyhow::bail!("Image collection based tilesets are not supported");
    };
    let atlas = resolve_atlas(resolver, &tileset.name, &tileset_atlas.source)?;

    let mut tilemap = Vec::with_capacity((layer_width * layer_height) as usize);
    for y in 0..layer_height {
        for x in 0..layer_width {
            let tile_idx = match tile_layer.get_tile(x as i32, y as i32) {
                None => TileIdx::Empty,
                Some(t) => TileIdx::from_repr(t.id()).ok_or_else(|| {
                    anyhow::anyhow!("Tileset {:?}, tile {:}: unknown tile", t.id(), tileset.name,)
                })?,
            };
            tilemap.push(tile_idx);
        }
    }

    Ok(MapDef {
        width: layer_width,
        height: layer_height,
        tiles,
        tilemap,
        atlas,
        atlas_margin: tileset.margin,
        atlas_spacing: tileset.spacing,
    })
}

fn load_entity_defs_from_object_layer(layer: &tiled::Layer) -> anyhow::Result<Vec<EntityDef>> {
    let Some(object_layer) = layer.as_object_layer() else {
        anyhow::bail!("Expected layer {OBJECT_LAYER:?} to be an object layer")
    };

    let mut entities = Vec::new();
    for object in object_layer.objects() {
        anyhow::ensure!(
            !object.user_type.is_empty(),
            "Layer {OBJECT_LAYER:?}, object {}: no class",
            object.id(),
        );
        let info = from_properties(&object.user_type, &object.properties)
            .with_context(|| format!("Layer {OBJECT_LAYER:?}, object {}", object.id()))?;
        let (width, height) = match object.shape {
            tiled::ObjectShape::Rect { width, height } => (width, height),
            _ => anyhow::bail!(
                "Layer {OBJECT_LAYER:?}, object {}: non square object",
                object.id(),
            ),
        };

        entities.push(EntityDef {
            tf: EntityPosition {
                pos: Position {
                    x: object.x + width / 2.0,
                    y: object.y + height / 2.0,
                },
                angle: object.rotation / 180.0 * std::f32::consts::PI,
            },
            info,
        });
    }

    Ok(entities)
}

fn resolve_atlas(
    resolver: &FsResolver,
    tileset: &str,
    atlas_path: impl AsRef<Path>,
) -> anyhow::Result<TextureId> {
    let atlas_path = atlas_path.as_ref();
    let atlas_path = fs::canonicalize(atlas_path)
        .with_context(|| format!("Canonicalizing tileset {tileset:?} atlas {atlas_path:?}"))?;
    resolver
        .inverse_resolve::<Texture2D>(&atlas_path)
        .with_context(|| format!("Inverse resolving {atlas_path:?}"))
}

static REQUIRED_TILED_VERSION: &str = "1.10";
static OBJECT_LAYER: &str = "Actors";
static WORLD_LAYER: &str = "World";
static TILE_CLASS: &str = "Tile";
