mod components;

use hashbrown::HashMap;
use lib_dbg::dump;

pub use components::*;
use hecs::{Entity, World};
use lib_level::{LevelDef, TILE_SIDE, TileIdx};
use macroquad::prelude::*;

use crate::Transform;

const FONT_SCALE: f32 = 1.0;
const MAIN_FONT_SIZE: u16 = 32;
const HINT_FONT_SIZE: u16 = 16;
const VERTICAL_ORIENT_HORIZONTAL_PADDING: f32 = 16.0;
pub static ORIENTATION_TEXT: &'static str = "Wrong Orientation";
pub static ORIENTATION_HINT: &'static str = "Please re-orient your device\ninto landscape";

struct TextureVal {
    texture: Texture2D,
}

/// Render does rendering stuff. When it comes to the world
/// drawing, all the data is taken from its own world -- "export world".
///
/// It also provides a simple asset storage for quick access
/// for the rendering code callers.
pub struct Render {
    pub ui_font: FontKey,
    pub world: World,

    tilemap_atlas: TextureKey,
    tilemap_tiles: Vec<Rect>,
    tilemap_data: Vec<TileIdx>,
    tilemap_width: usize,
    tilemap_height: usize,

    sprite_buffer: Vec<SpriteData>,

    to_delete: Vec<Entity>,
    time: f32,

    textures: HashMap<TextureKey, TextureVal>,
    fonts: HashMap<FontKey, Font>,
}

impl Render {
    pub fn new() -> Self {
        let world = World::new();

        Self {
            ui_font: FontKey("undefined"),
            tilemap_atlas: TextureKey("undefined"),
            tilemap_tiles: Vec::new(),
            tilemap_data: Vec::new(),
            tilemap_width: 0,
            tilemap_height: 0,
            sprite_buffer: Vec::new(),
            world,
            to_delete: Vec::new(),
            time: 0.0,
            textures: HashMap::new(),
            fonts: HashMap::new(),
        }
    }

    pub fn add_texture(&mut self, key: TextureKey, texture: &Texture2D) {
        self.textures.insert(
            key,
            TextureVal {
                texture: texture.clone(),
            },
        );
    }

    pub fn add_font(&mut self, key: FontKey, font: &Font) {
        self.fonts.insert(key, font.clone());
    }

    pub fn get_font(&self, key: FontKey) -> Option<&Font> {
        self.fonts.get(&key)
    }

    /// * `atlas`: the atlas texture key
    /// * `atlas_margin`: space around the whole tileset
    /// * `atlas_spacing`: space between tiles
    pub fn set_atlas(&mut self, atlas: TextureKey, atlas_margin: u32, atlas_spacing: u32) {
        let Some(atlas_texture) = self.textures.get(&atlas) else {
            warn!("No such texture: {atlas:?}");
            return;
        };

        let atlas_width = atlas_texture.texture.width() as u32;
        let atlas_height = atlas_texture.texture.height() as u32;
        let (atlas_tiles_x, atlas_tiles_y) =
            get_tile_count_in_atlas(atlas_width, atlas_height, atlas_margin, atlas_spacing);

        self.tilemap_atlas = atlas;
        self.tilemap_tiles.clear();
        self.tilemap_tiles
            .reserve((atlas_tiles_x * atlas_tiles_y) as usize);
        for x in 0..atlas_tiles_x {
            for y in 0..atlas_tiles_y {
                let tex_x = (TILE_SIDE + atlas_spacing) * x + atlas_margin;
                let tex_y = (TILE_SIDE + atlas_spacing) * y + atlas_margin;
                self.tilemap_tiles.push(Rect {
                    x: tex_x as f32,
                    y: tex_y as f32,
                    w: TILE_SIDE as f32,
                    h: TILE_SIDE as f32,
                });
            }
        }
    }

    pub fn set_tilemap(&mut self, level: &LevelDef) {
        self.tilemap_data.clear();
        self.tilemap_data
            .reserve((level.map.width * level.map.height) as usize);
        self.tilemap_data.extend(level.map.tilemap.iter().copied());
        self.tilemap_width = level.map.width as usize;
        self.tilemap_height = level.map.height as usize;
    }

    pub fn new_frame(&mut self) {
        dump!("Render entities: {}", self.world.iter().count());

        self.sprite_buffer.clear();

        let it = self
            .world
            .iter()
            .filter(|ent| {
                let Some(timed) = ent.get::<&Timed>() else {
                    return true;
                };

                Self::should_delete_timed(&timed)
            })
            .map(|x| x.entity());
        self.to_delete.extend(it);

        for e in self.to_delete.drain(..) {
            let _ = self.world.despawn(e);
        }
    }

    pub fn render(&mut self, camera: &dyn Camera, dry_run: bool, dt: f32) {
        clear_background(Color {
            r: 0.0,
            g: 0.0,
            b: 0.02,
            a: 1.0,
        });

        self.draw_world(camera, dt, dry_run);

        self.setup_ui_camera();
        self.draw_announcement_text();
    }

    pub fn debug_render<F>(&mut self, camera: &dyn Camera, code: F)
    where
        F: FnOnce(),
    {
        set_camera(camera);
        code();
    }

    fn draw_world(&mut self, camera: &dyn Camera, dt: f32, dry_run: bool) {
        self.update_time(dt);
        self.update_flickers();
        self.anim_vert_shrink_fadeout();

        if dry_run {
            return;
        }

        set_camera(camera);

        self.put_tilemap_into_sprite_buffer();
        self.draw_sprites();

        self.draw_circles();
        self.draw_rects();
        self.draw_texts();
    }

    fn setup_ui_camera(&mut self) {
        match self.get_font(self.ui_font) {
            None => {
                warn!("No such font: {:?}", self.ui_font);
                set_default_camera();
            }
            Some(font) => set_camera(&Self::get_ui_cam(font)),
        }
    }

    fn put_tilemap_into_sprite_buffer(&mut self) {
        self.sprite_buffer
            .reserve(self.tilemap_height * self.tilemap_width);
        for (idx, &tile_idx) in self.tilemap_data.iter().enumerate() {
            let tile_x = (idx % self.tilemap_width) as u32;
            let tile_y = (idx / self.tilemap_width) as u32;
            let tile_rect = self.tilemap_tiles[tile_idx as usize];

            self.sprite_buffer.push(SpriteData {
                layer: 0,
                tf: Transform {
                    pos: vec2((tile_x * TILE_SIDE) as f32, (tile_y * TILE_SIDE) as f32),
                    angle: 0.0,
                },
                texture: self.tilemap_atlas,
                rect: tile_rect,
                origin: vec2(0.0, 0.0),
                color: WHITE,
                sort_offset: 0.0,
            });
        }
    }

    fn draw_announcement_text(&mut self) {
        for (_, announce) in self.world.query_mut::<&AnnouncementText>() {
            let Some(font) = self.fonts.get(&self.ui_font) else {
                warn!("No such font: {:?}", self.ui_font);
                continue;
            };

            let view_rect = Self::ui_view_rect(font);

            draw_rectangle(
                view_rect.x,
                view_rect.y,
                view_rect.w,
                view_rect.h,
                Color {
                    r: 0.0,
                    g: 0.0,
                    b: 0.12,
                    a: 0.5,
                },
            );

            let center = get_text_center(
                announce.heading,
                Some(font),
                MAIN_FONT_SIZE,
                FONT_SCALE,
                0.0,
            );
            draw_text_ex(
                announce.heading,
                view_rect.left() + view_rect.w / 2.0 - center.x,
                view_rect.top() + view_rect.h / 2.0 - center.y,
                TextParams {
                    font: Some(font),
                    font_size: MAIN_FONT_SIZE,
                    color: Color::from_hex(0xDDFBFF),
                    font_scale: FONT_SCALE,
                    ..Default::default()
                },
            );

            let Some(hint) = announce.body else {
                continue;
            };
            let center = get_text_center(
                Self::find_longest_line(hint),
                Some(font),
                HINT_FONT_SIZE,
                FONT_SCALE,
                0.0,
            );
            draw_multiline_text_ex(
                hint,
                view_rect.left() + view_rect.w / 2.0 - center.x,
                view_rect.top() + view_rect.h / 2.0 - center.y + (MAIN_FONT_SIZE as f32) * 1.5,
                None,
                TextParams {
                    font: Some(font),
                    font_size: HINT_FONT_SIZE,
                    color: Color::from_hex(0xDDFBFF),
                    font_scale: FONT_SCALE,
                    ..Default::default()
                },
            );
        }
    }

    fn draw_rects(&mut self) {
        for (_, (rect, tint, tf, scale)) in
            self.world
                .query_mut::<(&RectShape, Option<&Tint>, &Transform, Option<&Scale>)>()
        {
            let scale = scale.map(|x| x.0).unwrap_or(vec2(1.0, 1.0));
            let tint = tint.map(|x| x.0).unwrap_or(WHITE);
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
        }
    }

    fn draw_circles(&mut self) {
        for (_, (circle, tint, tf)) in self
            .world
            .query_mut::<(&CircleShape, Option<&Tint>, &Transform)>()
        {
            let tint = tint.map(|x| x.0).unwrap_or(WHITE);

            draw_circle(tf.pos.x, tf.pos.y, circle.radius, tint);
        }
    }

    fn draw_sprites(&mut self) {
        self.sprite_buffer.sort_by(|s1, s2| {
            let y_s1 = s1.tf.pos.y + s1.sort_offset;
            let y_s2 = s2.tf.pos.y + s2.sort_offset;
            u32::cmp(&s1.layer, &s2.layer).then(f32::total_cmp(&y_s1, &y_s2))
        });

        for sprite in self.sprite_buffer.iter() {
            let Some(TextureVal { texture, .. }) = self.textures.get(&sprite.texture) else {
                warn!("No texture {:?}", sprite.texture.0);
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
                    pivot: Some(sprite.origin),
                },
            );
        }
    }

    fn draw_texts(&mut self) {
        for (_, (text, tf, tint)) in self
            .world
            .query_mut::<(&GlyphText, &Transform, Option<&Tint>)>()
        {
            let tint = tint.map(|x| x.0).unwrap_or(WHITE);
            let Some(font) = self.fonts.get(&text.font) else {
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
        }
    }

    fn anim_vert_shrink_fadeout(&mut self) {
        for (_, (timed, tint, scale)) in self.world.query_mut::<(&Timed, &mut Tint, &mut Scale)>() {
            let k = timed.time / timed.start;

            tint.0.a = (2.0 * k).clamp(0.0, 1.0);
            scale.0.y = k.powf(4.0);
        }
    }

    fn update_time(&mut self, dt: f32) {
        self.time += dt;

        for (_, timed) in self.world.query_mut::<&mut Timed>() {
            timed.time -= dt;
        }
    }

    fn update_flickers(&mut self) {
        let flicker_vis = (self.time * 1000.0) as u32 % 2 == 0;
        if flicker_vis {
            return;
        }

        for (_, (_flicker, tint)) in self.world.query_mut::<(&Flicker, &mut Tint)>() {
            tint.0.a = 0.0;
        }
    }

    fn should_delete_timed(timed: &Timed) -> bool {
        timed.time <= 0.0
    }

    fn find_longest_line(text: &str) -> &str {
        text.split('\n').max_by_key(|x| x.len()).unwrap_or("")
    }

    fn ui_view_rect(font: &Font) -> Rect {
        // Special case for misoriented mobile devices
        if screen_height() > screen_width() {
            let measure = measure_text(ORIENTATION_TEXT, Some(font), MAIN_FONT_SIZE, FONT_SCALE);
            let view_width = measure.width + 2.0 * VERTICAL_ORIENT_HORIZONTAL_PADDING;

            return Rect {
                x: -VERTICAL_ORIENT_HORIZONTAL_PADDING,
                y: 0.0,
                w: view_width,
                h: view_width * (screen_height() / screen_width()),
            };
        }

        let view_height = (MAIN_FONT_SIZE as f32) * 12.0;
        Rect {
            x: 0.0,
            y: 0.0,
            w: view_height * (screen_width() / screen_height()),
            h: view_height,
        }
    }

    fn get_ui_cam(font: &Font) -> Camera2D {
        let mut cam = Camera2D::from_display_rect(Self::ui_view_rect(font));
        cam.zoom.y *= -1.0;

        cam
    }
}

#[derive(Debug, Clone, Copy)]
struct SpriteData {
    layer: u32,
    tf: Transform,
    texture: TextureKey,
    rect: Rect,
    origin: Vec2,
    color: Color,
    sort_offset: f32,
}

fn get_tile_count_in_atlas(
    mut atlas_width: u32,
    mut atlas_height: u32,
    atlas_margin: u32,
    atlas_spacing: u32,
) -> (u32, u32) {
    assert!(atlas_height >= TILE_SIDE + 2 * atlas_margin);
    assert!(atlas_width >= TILE_SIDE + 2 * atlas_margin);

    // Remove margins and the leading tile
    atlas_width -= TILE_SIDE + 2 * atlas_margin;
    atlas_height -= TILE_SIDE + 2 * atlas_margin;

    let tiles_x = atlas_width / (TILE_SIDE + atlas_spacing);
    let tiles_y = atlas_height / (TILE_SIDE + atlas_spacing);

    // Add the leading tiles back
    (tiles_x + 1, tiles_y + 1)
}

#[cfg(test)]
mod tests {
    use super::{TILE_SIDE, get_tile_count_in_atlas};

    const SAMPLE_COUNT: usize = 1000;

    #[derive(Clone, Copy)]
    struct TileCountTestSample {
        tiles_x: u32,
        tiles_y: u32,
        margin: u32,
        spacing: u32,
    }

    impl TileCountTestSample {
        fn width(&self) -> u32 {
            self.tiles_x * (TILE_SIDE + self.spacing) + 2 * self.margin
        }

        fn height(&self) -> u32 {
            self.tiles_y * (TILE_SIDE + self.spacing) + 2 * self.margin
        }
    }

    #[test]
    fn test_get_tile_count_in_atlas() {
        for _ in 0..SAMPLE_COUNT {
            let sample = TileCountTestSample {
                tiles_x: rand::random_range(1..100),
                tiles_y: rand::random_range(1..100),
                margin: rand::random_range(0..10),
                spacing: rand::random_range(0..3),
            };
            let (other_tiles_x, other_tiles_y) = get_tile_count_in_atlas(
                sample.width(),
                sample.height(),
                sample.margin,
                sample.spacing,
            );
            assert_eq!(
                (other_tiles_x, other_tiles_y),
                (sample.tiles_x, sample.tiles_y),
                "margin={} spacing={}",
                sample.margin,
                sample.spacing,
            );
        }
    }
}
