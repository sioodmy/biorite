use crate::prelude::*;

use bevy::utils::hashbrown::HashMap;
pub use block_mesh::ndshape::{ConstShape, ConstShape3u32};
pub use block_update::*;
pub use blocks::*;
pub use chunk_updater::*;
pub use collider::*;
pub use generator::*;
pub use pos::*;
pub use request::*;
pub use sender::*;
pub use serde_big_array::BigArray;
pub use server::*;

pub mod block_update;
pub mod blocks;
pub mod chunk_updater;
pub mod collider;
pub mod generator;
pub mod pos;
pub mod request;
pub mod sender;
pub mod server;

pub type CompressedChunk = Vec<u8>;
pub const CHUNK_DIM: u32 = 16;
pub type ChunkShape =
    ConstShape3u32<{ CHUNK_DIM + 2 }, { CHUNK_DIM + 2 }, { CHUNK_DIM + 2 }>;

pub struct ChunkEntry {
    pub chunk: Chunk,
    pub entity: Entity,
}
#[derive(Resource)]
pub struct LoadedChunks(pub HashMap<IVec3, ChunkEntry>);

#[derive(Serialize, Deserialize, Debug, Resource, Copy, Clone)]
pub struct Chunk {
    pub position: IVec3,
    #[serde(with = "BigArray")]
    pub blocks: [BlockType; ChunkShape::SIZE as usize],
    pub modified: bool,
    pub loaded: bool,
}

#[derive(Component)]
pub struct ChunkComponent {
    pub position: IVec3,
    pub loaded: bool,
}

impl Default for Chunk {
    fn default() -> Self {
        Chunk {
            position: IVec3::ZERO,
            // Empty chunk filled with air
            blocks: [BlockType::Air; ChunkShape::SIZE as usize],
            modified: false,
            loaded: false,
        }
    }
}

impl Chunk {
    pub fn compress(&self) -> CompressedChunk {
        let message = bincode::serialize(self).unwrap();
        compress_prepend_size(&message)
    }
    pub fn from_compressed(bytes: &CompressedChunk) -> Self {
        let message = decompress_size_prepended(&bytes).unwrap();
        bincode::deserialize(&message).unwrap()
    }
}
