use crate::level::LevelDef;

/// Loads a level from memory, treating the bytes as the binary level
/// format encoding. For internal use only.
pub fn load_from_memory(data: &[u8]) -> anyhow::Result<LevelDef> {
    postcard::from_bytes(data).map_err(Into::into)
}

/// Level compilation routine. For internal use only.
#[cfg(not(target_family = "wasm"))]
pub mod compile {
    use super::LevelDef;
    use postcard::{ser_flavors::io::WriteFlavor, serialize_with_flavor};

    use std::io::Write;

    /// Write the level to `out` in binary format.
    pub fn write_level(level: &LevelDef, out: impl Write) -> anyhow::Result<()> {
        serialize_with_flavor(level, WriteFlavor::new(out))?;
        Ok(())
    }
}
