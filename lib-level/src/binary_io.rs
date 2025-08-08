use crate::{LoadLevelError, level::LevelDef};

use std::error::Error as StdError;

/// Loads a level from memory, treating the bytes as the binary level
/// format encoding. For internal use only.
pub fn load_from_memory(data: &[u8]) -> Result<LevelDef, LoadLevelError> {
    postcard::from_bytes(data)
        .map_err(|e| Box::new(e) as Box<dyn StdError>)
        .map_err(LoadLevelError::Loading)
}

/// Level compilation routine. For internal use only.
#[cfg(not(target_family = "wasm"))]
pub mod compile {
    use super::LevelDef;
    use postcard::{ser_flavors::io::WriteFlavor, serialize_with_flavor};

    use std::error::Error as StdError;
    use std::io::Write;

    /// Write the level to `out` in binary format.
    pub fn write_level(level: &LevelDef, out: impl Write) -> Result<(), Box<dyn StdError>> {
        serialize_with_flavor(level, WriteFlavor::new(out))
            .map(|_| ())
            .map_err(|e| Box::new(e) as Box<dyn StdError>)
    }
}
