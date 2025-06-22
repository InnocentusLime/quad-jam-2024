mod game;

use game::Project;
use macroquad::prelude::*;

fn window_conf() -> Conf {
    Conf {
        window_title: "Project Swarm".to_owned(),
        high_dpi: true,
        window_width: 1600,
        window_height: 900,
        fullscreen: false,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        lib_game::sys::panic_screen(&format!("Driver panicked:\n{}", info));
        hook(info);
    }));

    set_max_level(STATIC_MAX_LEVEL);

    let mut app = lib_game::App::new(&window_conf()).await.unwrap();
    let mut project = Project::new(&mut app).await;

    app.run(&mut project).await;
}
