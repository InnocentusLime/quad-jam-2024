mod game;

use game::MainGame;
use serde::Deserialize;

#[derive(Debug, Clone, Copy, Deserialize)]
pub struct PlayerTag;

#[derive(Debug, Clone, Copy, Deserialize)]
pub struct BulletTag;

fn main() {
    let mut prefab_factory = lib_game::PrefabFactory::new();
    prefab_factory.register_component::<PlayerTag>("player");
    prefab_factory.register_component::<BulletTag>("bullet");

    lib_game::run(lib_game::AppInit {
        initial_state: Box::new(MainGame::new()),
        prefab_factory,
    });
}
