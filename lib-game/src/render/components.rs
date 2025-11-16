use std::borrow::Cow;

use lib_asset::FontId;
use macroquad::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[repr(transparent)]
pub struct FontKey(pub &'static str);

/// Tags the entity to be drawn with a certain color.
#[derive(Clone, Copy, Debug)]
#[repr(transparent)]
pub struct Tint(pub Color);

/// Tags the entity to be procedurally animated. This
/// procedural animation does the following:
/// 1. Shrink the entity along y axis by making the scale go from 1.0 to 0.0
/// 2. Slowly dials down the alpha of its tint from 1.0 to 0.0
///     (the original alpha value in [Tint] is overriden)
///
/// Requires [crate::components::Transform], [Timed] and [Tint] to work.
#[derive(Clone, Copy, Debug)]
pub struct VertShrinkFadeoutAnim;

/// Draws some text with glyphs.
///
/// Requires [crate::components::Transform] to work.
///
/// Can read [Tint], but it is optional. By default the text is
/// drawn as white.
#[derive(Clone, Debug)]
pub struct GlyphText {
    pub font: FontId,
    pub string: Cow<'static, str>,
    pub font_size: u16,
    pub font_scale: f32,
    pub font_scale_aspect: f32,
}

/// Renders an announcement text with a background.
#[derive(Clone, Copy, Debug)]
pub struct AnnouncementText {
    pub heading: &'static str,
    pub body: Option<&'static str>,
}
