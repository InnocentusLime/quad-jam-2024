#[macro_export]
macro_rules! wrap_method {
    ($p:path as $new:ident(
        this:$thisty:ty |
            $($carg:ident: $cargty:ty),* |
            $($parg:ident: $pargty:ty),*
    )) => {
        pub fn $new(
            world: &shipyard::World,
            $($parg: $pargty),*
        ) {
            use shipyard::*;

            #[allow(unused_parens)]
            let ($(mut $carg),*) = world.borrow::<($($cargty),*)>().unwrap();

            $p(
                &mut *world.borrow::<UniqueViewMut<$thisty>>().unwrap(),
                $(&mut $carg),*,
                $($parg),*
            )
        }
    };
}

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