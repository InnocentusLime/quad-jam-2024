mod main_game;
mod components;
mod prelude;

use main_game::MainGame;

fn main() {
    let mut prefab_factory = lib_game::PrefabFactory::new();
    components::register_components(&mut prefab_factory);

    lib_game::run(lib_game::AppInit {
        initial_state: Box::new(MainGame::new()),
        prefab_factory,
    });
}
