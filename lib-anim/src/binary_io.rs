use hashbrown::HashMap;

use crate::{AnimationId, animation::Animation};

/// Loads an animation pack from memory, treating the bytes as the binary level
/// format encoding. For internal use only.
pub fn load_from_memory(data: &[u8]) -> anyhow::Result<HashMap<AnimationId, Animation>> {
    postcard::from_bytes(data).map_err(Into::into)
}

/// Animation pack compilation routine. For internal use only.
#[cfg(not(target_family = "wasm"))]
pub mod compile {
    use super::{Animation, AnimationId, HashMap};
    use postcard::{ser_flavors::io::WriteFlavor, serialize_with_flavor};

    use std::io::Write;

    /// Write the animation pack to `out` in binary format.
    pub fn write_animation_pack(
        anims: &HashMap<AnimationId, Animation>,
        out: impl Write,
    ) -> anyhow::Result<()> {
        serialize_with_flavor(anims, WriteFlavor::new(out))?;
        Ok(())
    }
}
