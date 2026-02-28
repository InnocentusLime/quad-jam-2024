mod game;

use game::MainGame;
use mimiq::*;

fn window_conf() -> Conf {
    Conf {
        ..Default::default()
    }
}

fn main() {
    mimiq::run::<lib_game::App>(window_conf());
}
