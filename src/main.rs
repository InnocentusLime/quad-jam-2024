mod game;

use game::MainGame;
use serde::Deserialize;

#[derive(Debug, Clone, Copy, Deserialize)]
pub struct PlayerTag;

fn main() {
    let mut prefab_factory = lib_game::PrefabFactory::new();
    prefab_factory.register_component::<PlayerTag>("player");

    lib_game::run(lib_game::AppInit {
        initial_state: Box::new(MainGame::new()),
        prefab_factory,
    });
}
