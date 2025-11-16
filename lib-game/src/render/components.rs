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

pub struct AnnouncementText {
    pub heading: &'static str,
    pub body: Option<&'static str>,
}
