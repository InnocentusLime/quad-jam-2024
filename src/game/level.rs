use macroquad::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum EntityDef {
    Player(Vec2),
    Goal(Vec2),
    Damager(Vec2),
}

#[derive(Debug, Serialize, Deserialize)]
pub enum TileDef {
    #[serde(rename = "W")]
    Wall,
    #[serde(rename = "G")]
    Ground,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MapDef {
    pub width: usize,
    pub height: usize,
    pub tiles: Vec<TileDef>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LevelDef {
    pub next_level: Option<String>,
    pub map: MapDef,
    pub entities: Vec<EntityDef>,
}
