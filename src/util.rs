#[macro_export]
macro_rules! method_as_system {
    ($p:path as $new:ident($this:ident:$thisty:ty, $($arg:ident: $argty:ty),*)) => {
        pub fn $new(mut $this:shipyard::UniqueViewMut<$thisty>, $($arg: $argty),*) {
            $p(&mut *$this, $($arg),*);
        }
    };
}

#[macro_export]
macro_rules! wrap_method {
    ($p:path as $new:ident(
        this:$thisty:ty |
            $($carg:ident: $cargty:ty),* |
            $($parg:ident: $pargty:ty),*
    )) => {
        pub fn $new(
            world: &mut shipyard::World,
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