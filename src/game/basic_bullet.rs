use super::prelude::*;

pub fn init(builder: &mut EntityBuilder, pos: Vec2, look_angle: f32, resources: &Resources) {
    let (projectile, attack) =  ProjectileBundle::new_enemy(
        pos, 
        resources.cfg.basic_bullet.shape, 
        Vec2::from_angle(look_angle), 
        resources.cfg.basic_bullet.graze_value, 
        resources.cfg.basic_bullet.speed,
    );
    builder.add_bundle(projectile);
    builder.add_bundle(attack);
    builder.add(Sprite {
        layer: 0,
        texture: resources.textures.resolve("atlas/world.png").unwrap(),
        rect: Rect { x: TILE_SIDE_F32, y: 0.0, w: TILE_SIDE_F32, h: TILE_SIDE_F32 },
        color: WHITE,
        sort_offset: 0.0,
        local_offset: Vec2::splat(-TILE_SIDE_F32 / 2.0),
    });
}
