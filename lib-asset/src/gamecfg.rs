macro_rules! game_cfg {
    (
        $( $section_name:ident : $section_ty:ident {
            $( $field_name:ident : $field_ty:ty ),+ $(,)?
        } ),*
        $(,)?
    ) => {
        #[derive(Debug, Clone, Copy, Default, serde::Deserialize, serde::Serialize)]
        pub struct GameCfg {
            $( pub $section_name : sections::$section_ty ),+
        }

        impl GameCfg {
            #[cfg(feature = "dbg")]
            pub fn set_field(&mut self, section: &str, field: &str, val: &str) -> anyhow::Result<()> {
                match section {
                    $(stringify!($section_name) => match field {
                        $(stringify!($field_name) => self.$section_name.$field_name = serde_json::from_str(val)?,)+
                        _ => anyhow::bail!("unknown field in section {section:?}: {field:?}"),
                    })+
                    _ => anyhow::bail!("unknown section: {section:?}"),
                };
                Ok(())
            }

            #[cfg(feature = "dbg")]
            pub fn get_field(&self, section: &str, field: &str) -> anyhow::Result<String> {
                let val = match section {
                    $(stringify!($section_name) => match field {
                        $(stringify!($field_name) => serde_json::to_string(&self.$section_name.$field_name)?,)+
                        _ => anyhow::bail!("unknown field in section {section:?}: {field:?}"),
                    })+
                    _ => anyhow::bail!("unknown section: {section:?}"),
                };
                Ok(val)
            }
        }

        pub mod sections {
            $(
                #[derive(Debug, Clone, Copy, Default, serde::Deserialize, serde::Serialize)]
                pub struct $section_ty {
                    $( pub $field_name : $field_ty ),+
                }
            )+
        }
    };
}

game_cfg! {
    player: Player {
        speed: f32,
        dash_speed: f32,
        max_hp: i32,
        hit_cooldown: f32,
        shape: lib_col::Shape,
        max_stamina: f32,
        attack_cost: f32,
        dash_cost: f32,
        graze_shape: lib_col::Shape,
    },
    basic_bullet: BasicBullet {
        speed: f32,
        graze_value: f32,
        shape: lib_col::Shape,
    },
    shooter: Shooter {
        max_hp: i32,
        hit_cooldown: f32,
        shape: lib_col::Shape,
    },
    stabber: Stabber {
        max_hp: i32,
        hit_cooldown: f32,
        shape: lib_col::Shape,
        speed: f32,
        attack_range: f32,
    },
}
