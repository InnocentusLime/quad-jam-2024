mod game;

use game::MainGame;

fn main() {
    let prefab_factory = lib_game::PrefabFactory::new();

    lib_game::run(lib_game::AppInit {
        initial_state: Box::new(MainGame::new()),
        prefab_factory,
    });
}
