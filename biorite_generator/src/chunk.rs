use crate::blocks::*;
use bevy::{prelude::*, utils::hashbrown::HashMap};
use block_mesh::ndshape::{ConstShape, ConstShape3u32};
use lz4_flex::{compress_prepend_size, decompress_size_prepended};
use serde::{Deserialize, Serialize};

use serde_big_array::BigArray;

pub type CompressedChunk = Vec<u8>;
pub const CHUNK_DIM: u32 = 32;
pub type ChunkShape = ConstShape3u32<CHUNK_DIM, CHUNK_DIM, CHUNK_DIM>;
pub type ChunkBoundary =
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
        let message = decompress_size_prepended(bytes).unwrap();
        bincode::deserialize(&message).unwrap()
    }
}
