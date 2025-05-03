use std::borrow::Cow;

use macroquad::prelude::*;
use shipyard::{Component, Unique};

#[derive(Clone, Copy, Debug, Unique)]
pub struct CameraDef {
    /// Rotation in degrees.
    pub rotation: f32,
    /// Scaling, should be (1.0, 1.0) by default.
    pub zoom: Vec2,
    /// Rotation and zoom origin.
    pub target: Vec2,
    /// Displacement from target.
    pub offset: Vec2,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[repr(transparent)]
pub struct TextureKey(pub &'static str);

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[repr(transparent)]
pub struct FontKey(pub &'static str);

/// Tags the entity to flicker. The flicker works
/// by settings the alpha to 0 occasionally.
/// It is not overridden to 1, however.
///
/// Requires [Tint] to work.
#[derive(Clone, Copy, Component, Debug)]
pub struct Flicker;

/// Tags the entity to have a fixed lifetime. Once
/// the timer stops ticking -- it will get freed.
#[derive(Clone, Copy, Component, Debug)]
pub struct Timed {
    pub time: f32,
    pub start: f32,
}

impl Timed {
    pub fn new(init: f32) -> Self {
        Self {
            time: init,
            start: init,
        }
    }
}

/// Overrides the scaling of an entity.
#[derive(Clone, Copy, Component, Debug)]
pub struct Scale(pub Vec2);

/// Tags the entity to be drawn as a sprite. To see
/// what data to put into the texture, see [crate::render::Render].
///
/// Requires [crate::components::Transform] to work.
///
/// Can read [Tint] and [Scale], but they are optional.
/// * If no tint is specified, the sprite is drawn with a white tint
/// * If no scale is specified, the sprite is drawn with `(1.0, 1.0)` scale
#[derive(Clone, Copy, Component, Debug)]
pub struct Sprite {
    pub origin: Vec2,
    pub texture: TextureKey,
}

/// Draws a circle!
///
/// Requires [crate::components::Transform] to work, however
/// rotation will be ignored.
///
/// Can read [Tint], but it is optional.
/// * If no tint is specified, the circle is drawn with a white tint
#[derive(Clone, Copy, Component, Debug)]
pub struct CircleShape {
    pub radius: f32,
}

/// Draws a rect!
///
/// Requires [crate::components::Transform] to work.
///
/// Can read [Tint], but it is optional.
/// * If no tint is specified, the rect is drawn with a white tint
///
/// Can read [Scale], but it is optional.
#[derive(Clone, Copy, Component, Debug)]
pub struct RectShape {
    pub origin: Vec2,
    pub width: f32,
    pub height: f32,
}

/// Tags the entity to be drawn with a certain color.
#[derive(Clone, Copy, Component, Debug)]
#[repr(transparent)]
pub struct Tint(pub Color);

/// Tags the entity to be procedurally animated. This
/// procedural animation does the following:
/// 1. Shrink the entity along y axis by making the scale go from 1.0 to 0.0
/// 2. Slowly dials down the alpha of its tint from 1.0 to 0.0
///     (the original alpha value in [Tint] is overriden)
///
/// Requires [crate::components::Transform], [Timed] and [Tint] to work.
#[derive(Clone, Copy, Component, Debug)]
pub struct VertShrinkFadeoutAnim;

/// Draws some text with glyphs.
///
/// Requires [crate::components::Transform] to work.
///
/// Can read [Tint], but it is optional. By default the text is
/// drawn as white.
#[derive(Clone, Component, Debug)]
pub struct GlyphText {
    pub font: FontKey,
    pub string: Cow<'static, str>,
    pub font_size: u16,
    pub font_scale: f32,
    pub font_scale_aspect: f32,
}

/// Renders an announcement text with a background.
#[derive(Clone, Copy, Component, Debug)]
pub struct AnnouncementText {
    pub heading: &'static str,
    pub body: Option<&'static str>,
}