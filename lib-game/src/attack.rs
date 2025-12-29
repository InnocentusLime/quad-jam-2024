use crate::{
    BodyTag, CollisionSolver, GrazeGain, GrazeValue, Transform, col_group, col_query, dump,
};
use hecs::{Bundle, World};
use lib_asset::animation::Team;
use lib_col::{Group, Shape};

pub(crate) fn update_grazing(dt: f32, world: &mut World, col_solver: &CollisionSolver) {
    for (_, (graze_q, graze_gain)) in
        &mut world.query::<(&mut col_query::Grazing, &mut GrazeGain)>()
    {
        for collision in col_solver.collisions_for(graze_q) {
            let Ok(graze_val) = world.get::<&GrazeValue>(*collision) else {
                continue;
            };
            graze_gain.value += graze_val.0 * dt;
            graze_gain.value = graze_gain.value.min(graze_gain.max_value);
        }
        dump!("Graze: {:.2}", graze_gain.value);
    }
}

#[derive(Bundle)]
pub struct AttackBundle {
    pub tf: Transform,
    pub team: Team,
    pub query: col_query::Damage,
    pub graze_hitbox: BodyTag,
    pub graze_value: GrazeValue,
}

impl AttackBundle {
    pub fn new(tf: Transform, team: Team, shape: Shape, graze_value: f32, filter: Group) -> Self {
        AttackBundle {
            tf,
            team,
            query: col_query::Damage::new(shape, col_group::CHARACTERS, filter),
            graze_hitbox: BodyTag {
                groups: col_group::ATTACKS,
                shape,
            },
            graze_value: GrazeValue(graze_value),
        }
    }
}
