use crate::prelude::*;
pub use block_mesh::ndshape::{ConstShape, ConstShape3u32};
pub use serde_big_array::BigArray;
pub use zstd_util::*;

pub mod blocks;
pub use blocks::*;
pub const ZSTD_CHUNK_LVL: i32 = 22;

pub type CompressedChunk = Vec<u8>;
pub type ChunkShape = ConstShape3u32<18, 18, 18>;

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
        let bytes = bincode::serialize(&self).unwrap();
        let mut zstd = ZstdContext::new(ZSTD_CHUNK_LVL, None);
        zstd.compress(&bytes)
            .expect("Failed to compress chunk packet")
    }
    pub fn from_compressed(bytes: CompressedChunk) -> Self {
        let mut zstd = ZstdContext::new(ZSTD_CHUNK_LVL, None);
        let decompressed = zstd.decompress(&bytes).unwrap();
        bincode::deserialize(&decompressed).unwrap()
    }
}
