#[macro_export]
macro_rules! inline_tilemap {
    (@tile w) => { crate::components::TileType::Wall };
    (@tile g) => { crate::components::TileType::Ground };
    (@tile $i:ident) => { $i };
    ($($tile:ident),+) => {
        vec![
            $(inline_tilemap!(@tile $tile)),+
        ]
    };
}
