mod components;

use crate::{Sprite, dump};
pub use components::*;
use hecs::World;
use lib_asset::AssetKey;
use macroquad::prelude::*;

use crate::{Resources, Transform};

pub struct Render {
    pub announcement_text: Option<AnnouncementText>,
    pub sprite_buffer: Vec<SpriteData>,
}

impl Render {
    pub fn new() -> Self {
        Self {
            announcement_text: None,
            sprite_buffer: Vec::new(),
        }
    }

    pub fn new_frame(&mut self) {
        self.announcement_text = None;
        self.sprite_buffer.clear();
    }

    pub fn render(&mut self, resources: &Resources, render_world: bool, _dt: f32) {
        clear_background(Color {
            r: 0.0,
            g: 0.0,
            b: 0.02,
            a: 1.0,
        });

        if render_world {
            self.draw_sprites(resources);
        }
    }

    pub fn debug_render<F>(&mut self, code: F)
    where
        F: FnOnce(),
    {
        code();
    }

    fn draw_sprites(&mut self, resources: &Resources) {
        dump!("sprites drawn: {}", self.sprite_buffer.len());

        self.sprite_buffer.sort_by(|s1, s2| {
            let y_s1 = s1.tf.pos.y + s1.sort_offset;
            let y_s2 = s2.tf.pos.y + s2.sort_offset;
            u32::cmp(&s1.layer, &s2.layer).then(f32::total_cmp(&y_s1, &y_s2))
        });

        for sprite in self.sprite_buffer.iter() {
            let Some(texture) = resources.textures.get(sprite.texture) else {
                warn!("No texture {:?}", sprite.texture);
                continue;
            };
            draw_texture_ex(
                texture,
                sprite.tf.pos.x,
                sprite.tf.pos.y,
                sprite.color,
                DrawTextureParams {
                    dest_size: None,
                    source: Some(sprite.rect),
                    rotation: sprite.tf.angle,
                    flip_x: false,
                    flip_y: false,
                    pivot: Some(sprite.tf.pos),
                },
            );
        }
    }

    pub fn buffer_sprites(&mut self, world: &mut World) {
        for (_, (tf, sprite)) in world.query_mut::<(&Transform, &Sprite)>() {
            self.sprite_buffer.push(SpriteData {
                layer: sprite.layer,
                tf: Transform {
                    pos: tf.pos + sprite.local_offset,
                    angle: tf.angle,
                },
                texture: sprite.texture,
                rect: sprite.rect,
                color: sprite.color,
                sort_offset: sprite.sort_offset,
            });
        }
    }
}

impl Default for Render {
    fn default() -> Self {
        Render::new()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct SpriteData {
    pub layer: u32,
    pub tf: Transform,
    pub texture: AssetKey,
    pub rect: Rect,
    pub color: Color,
    pub sort_offset: f32,
}
