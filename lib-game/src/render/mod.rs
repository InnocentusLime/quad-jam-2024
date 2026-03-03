mod components;

use std::rc::Rc;

use crate::{Sprite, dump};
pub use components::*;
use glam::*;
use hecs::World;
use lib_asset::{AssetKey, INVALID_ASSET};
use mimiq::util::{ShapeBatcher, SpriteBatcher};
use mimiq::{BLACK, Clear, GlContext, Texture2D};

use crate::{Resources, Transform};

pub struct Render {
    pub curr_texture: AssetKey,
    pub sprite_batcher: SpriteBatcher,

    pub gizmos: ShapeBatcher,
}

impl Render {
    pub fn new(resources: &Resources) -> Self {
        Self {
            curr_texture: INVALID_ASSET,
            sprite_batcher: SpriteBatcher::new_from_size(&resources.gl_ctx, 1_000),
            gizmos: ShapeBatcher::new_from_size(&resources.gl_ctx, 20_000, 20_000),
        }
    }

    pub fn new_frame(&mut self) { /* NO-OP */
    }

    pub fn render(&mut self, resources: &Resources, render_world: bool) {
        resources
            .gl_ctx
            .default_pass(Clear::depth_color(BLACK), |width, height| {
                let view_projection =
                    Mat4::orthographic_rh_gl(0.0, width as f32, height as f32, 0.0, 0.0, 100.0);
                if render_world {
                    self.draw_sprites(resources, view_projection);
                }
                self.gizmos.basic_draw(
                    &resources.gl_ctx,
                    view_projection,
                    &resources.basic_pipeline,
                );
            });
    }

    fn draw_sprites(&mut self, resources: &Resources, view_projection: Mat4) {
        // TODO: need sprite length
        // dump!("sprites drawn: {}", self.sprite_buffer.len());

        // TODO: need possibility to sort
        // self.sprite_batcher.sort_by(|s1, s2| {
        //     let y_s1 = s1.tf.pos.y + s1.sort_offset;
        //     let y_s2 = s2.tf.pos.y + s2.sort_offset;
        //     u32::cmp(&s1.layer, &s2.layer).then(f32::total_cmp(&y_s1, &y_s2))
        // });

        let Some(texture) = resources.textures.get(self.curr_texture) else {
            // warn!("No texture {:?}", sprite.texture);
            return;
        };

        self.sprite_batcher.draw(
            &resources.gl_ctx,
            view_projection,
            &resources.sprite_pipeline,
            texture,
        );
    }

    pub fn buffer_sprites(&mut self, world: &mut World) {
        for (_, (tf, sprite)) in world.query_mut::<(&Transform, &Sprite)>() {
            let transform = Affine2::from_angle_translation(tf.angle, tf.pos);

            self.curr_texture = sprite.texture;
            self.sprite_batcher.add_sprite(mimiq::util::Sprite {
                tex_rect_pos: sprite.tex_rect_pos,
                tex_rect_size: sprite.tex_rect_size,
                color: sprite.color,
                transform,
            });
        }
    }
}

// #[derive(Debug, Clone, Copy)]
// pub struct SpriteData {
//     pub layer: u32,
//     pub tf: Transform,
//     pub texture: AssetKey,
//     pub rect: Rect,
//     pub color: Color,
//     pub sort_offset: f32,
// }
