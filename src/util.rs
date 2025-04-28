
#[macro_export]
macro_rules! inline_tilemap {
    (@tile w) => { crate::game::TileType::Wall };
    (@tile g) => { crate::game::TileType::Ground };
    (@tile $i:ident) => { $i };
    ($($tile:ident),+) => {
        vec![
            $(inline_tilemap!(@tile $tile)),+
        ]
    };
}