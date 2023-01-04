use crate::prelude::*;
pub use block_mesh::ndshape::{ConstShape, ConstShape3u32};
pub use blocks::*;
pub use generator::*;
pub use pos::*;
pub use request::*;
pub use sender::*;
pub use serde_big_array::BigArray;
pub use server::*;

pub mod blocks;
pub mod generator;
pub mod pos;
pub mod request;
pub mod sender;
pub mod server;

pub type CompressedChunk = Vec<u8>;
pub const CHUNK_DIM: u32 = 16;
pub type ChunkShape = ConstShape3u32<{ CHUNK_DIM + 2 }, { CHUNK_DIM + 2 }, { CHUNK_DIM + 2 }>;

use lz4::block::decompress;

#[derive(Serialize, Deserialize, Debug, Resource)]
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
            blocks: [BlockType(0); ChunkShape::SIZE as usize],
            modified: false,
            loaded: false,
        }
    }
}

impl Chunk {
    pub fn compress(&self) -> CompressedChunk {
        let message = bincode::serialize(self).unwrap();
        compress(&message, Some(CompressionMode::HIGHCOMPRESSION(12)), true).unwrap()
    }
    pub fn from_compressed(bytes: &CompressedChunk) -> Self {
        let message = decompress(bytes, None).unwrap();
        bincode::deserialize(&message).unwrap()
    }
}
