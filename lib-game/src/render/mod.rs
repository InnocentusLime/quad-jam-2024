mod components;

use quad_dbg::dump;
use std::collections::HashMap;

pub use components::*;
use macroquad::prelude::*;
use shipyard::{EntitiesView, EntityId, Get, IntoIter, UniqueView, View, ViewMut, World};

use crate::Transform;

struct TextureVal {
    texture: Texture2D,
    texture_rect: Rect,
}

/// Render does rendering stuff. When it comes to the world
/// drawing, all the data is taken from its own world -- "export world".
///
/// It also provides a simple asset storage for quick access
/// for the rendering code callers.
pub struct Render {
    pub world: World,

    camera: Camera2D,
    to_delete: Vec<EntityId>,
    time: f32,

    // FIXME: put a faster hash here
    textures: HashMap<TextureKey, TextureVal>,
    fonts: HashMap<FontKey, Font>,
}

impl Render {
    pub fn new() -> Self {
        Self {
            world: World::new(),
            camera: Camera2D::default(),
            to_delete: Vec::new(),
            time: 0.0,
            textures: HashMap::new(),
            fonts: HashMap::new(),
        }
    }

    pub fn add_texture(
        &mut self,
        key: TextureKey,
        texture: &Texture2D,
        texture_rect: Option<Rect>,
    ) {
        let texture_rect = texture_rect
            .unwrap_or(Rect {
                x: 0.0,
                y: 0.0,
                w: texture.width(),
                h: texture.height(),
            });

        self.textures.insert(
            key,
            TextureVal {
                texture_rect,
                texture: texture.clone(),
            },
        );
    }

    pub fn add_font(
        &mut self,
        key: FontKey,
        font: &Font
    ) {
        self.fonts.insert(
            key,
            font.clone(),
        );
    }

    pub fn new_frame(&mut self) {
        let _borrow_scope = {
            let ents = self.world.borrow::<EntitiesView>().unwrap();
            let timed = self.world.borrow::<View<Timed>>().unwrap();

            dump!("Render entities: {}", ents.iter().count());

            self.to_delete.extend(
                ents.iter()
                    .filter(|e| Self::should_delete_timed(&timed, *e))
            );
        };

        for e in self.to_delete.drain(..) {
            self.world.delete_entity(e);
        }
    }

    pub fn render(&mut self, dry_run: bool, dt: f32) {
        clear_background(Color {
            r: 0.0,
            g: 0.0,
            b: 0.02,
            a: 1.0,
        });

        self.setup_world_camera();
        self.draw_world(dt, dry_run);
    }

    pub fn debug_render<F>(&mut self, code: F)
    where
        F: FnOnce(),
    {
        self.setup_world_camera();
        code();
    }

    fn draw_world(&mut self, dt: f32, dry_run: bool) {
        self.update_time(dt);
        self.update_flickers();
        self.anim_vert_shrink_fadeout();

        if dry_run { return; }

        // FIXME: draw order is broken
        self.draw_sprites();
        self.draw_circles();
        self.draw_rects();
        self.draw_texts();
    }

    fn setup_world_camera(&mut self) {
        self.world.run(|cam_def: UniqueView<CameraDef>| {
            self.camera.offset = cam_def.offset;
            self.camera.rotation = cam_def.rotation;
            self.camera.zoom = cam_def.zoom;
            self.camera.target = cam_def.target;
        });

        set_camera(&self.camera);
    }

    fn draw_rects(&mut self) {
        self.world.run(|
            rect: View<RectShape>,
            tint: View<Tint>,
            tf: View<Transform>,
            scale: View<Scale>,
        | for (entity, (rect, tf)) in (&rect, &tf).iter().with_id() {
            let scale = scale.get(entity)
                .map(|x|x.0)
                .unwrap_or(vec2(1.0, 1.0));
            let tint = tint.get(entity)
                .map(|x| x.0)
                .unwrap_or(WHITE);
            draw_rectangle_ex(
                tf.pos.x,
                tf.pos.y,
                rect.width * scale.x,
                rect.height * scale.y,
                DrawRectangleParams {
                    offset: rect.origin,
                    rotation: tf.angle,
                    color: tint,
                },
            );
        })
    }

    fn draw_circles(&mut self) {
        self.world.run(|
            circle: View<CircleShape>,
            tint: View<Tint>,
            tf: View<Transform>,
        | for (entity, (circle, tf)) in (&circle, &tf).iter().with_id() {
            let tint = tint.get(entity)
                .map(|x| x.0)
                .unwrap_or(WHITE);

            draw_circle(
                tf.pos.x,
                tf.pos.y,
                circle.radius,
                tint,
            );
        })
    }

    fn draw_sprites(&mut self) {
        self.world.run(|
            sprite: View<Sprite>,
            tint: View<Tint>,
            tf: View<Transform>,
            scale: View<Scale>,
        | for (entity, (sprite, tf)) in (&sprite, &tf).iter().with_id() {
            let scale = scale.get(entity)
                .map(|x|x.0)
                .unwrap_or(vec2(1.0, 1.0));
            let tint = tint.get(entity)
                .map(|x| x.0)
                .unwrap_or(WHITE);
            let Some(TextureVal { texture, texture_rect: rect }) = self.textures.get(&sprite.texture)
            else {
                warn!("No texture {:?}", sprite.texture.0);
                continue;
            };

            draw_texture_ex(
                texture,
                tf.pos.x,
                tf.pos.y,
                tint,
                DrawTextureParams {
                    dest_size: Some(rect.size() * scale),
                    source: Some(*rect),
                    rotation: tf.angle,
                    flip_x: false,
                    flip_y: false,
                    pivot: Some(sprite.origin),
                },
            );
        })
    }

    fn draw_texts(&mut self) {
        self.world.run(|
            text: View<GlyphText>,
            tf: View<Transform>,
            tint: View<Tint>,
        | for (entity, (text, tf)) in (&text, &tf).iter().with_id() {
            let tint = tint.get(entity)
                .map(|x| x.0)
                .unwrap_or(WHITE);
            let Some(font) = self.fonts.get(&text.font)
            else {
                warn!("No font {:?}", text.font.0);
                continue;
            };

            draw_text_ex(
                &text.string,
                tf.pos.x,
                tf.pos.y,
                TextParams {
                    font: Some(font),
                    font_size: text.font_size,
                    font_scale: text.font_scale,
                    font_scale_aspect: text.font_scale_aspect,
                    rotation: tf.angle,
                    color: tint,
                },
            );
        })
    }

    fn anim_vert_shrink_fadeout(&mut self) {
        self.world.run(|
            timed: View<Timed>,
            mut tint: ViewMut<Tint>,
            mut scale: ViewMut<Scale>,
            tag: View<VertShrinkFadeoutAnim>,
        | for (timed, tint, scale, _) in (&timed, &mut tint, &mut scale, &tag).iter() {
            let k = timed.time / timed.start;

            tint.0.a = (2.0 * k).clamp(0.0, 1.0);
            scale.0.y = k.powf(4.0);
        });
    }

    fn update_time(&mut self, dt: f32) {
        self.time += dt;

        self.world.run(|mut timed: ViewMut<Timed>| for timed in (&mut timed).iter() {
            timed.time -= dt;
        })
    }

    fn update_flickers(&mut self) {
        let flicker_vis = (self.time * 1000.0) as u32 % 2 == 0;
        if flicker_vis { return; }

        self.world.run(|flicker: View<Flicker>, mut tint: ViewMut<Tint>| for (_, tint) in (&flicker, &mut tint).iter() {
            tint.0.a = 0.0;
        });
    }

    fn should_delete_timed(timed: &View<Timed>, entity: EntityId) -> bool {
        let Ok(timed) = timed.get(entity)
            else { return true };

        timed.time <= 0.0
    }
}