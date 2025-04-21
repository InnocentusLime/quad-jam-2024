
#[macro_export]
macro_rules! inline_tilemap {
    (@tile w) => { crate::TileType::Wall };
    (@tile g) => { crate::TileType::Ground };
    (@tile $i:ident) => { $i };
    ($($tile:ident),+) => {
        vec![
            $(inline_tilemap!(@tile $tile)),+
        ]
    };
}