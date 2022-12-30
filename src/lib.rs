pub use bevy::prelude::*;
pub use bevy_renet::{renet::*, *};
use block_mesh::{
    ndshape::{ConstShape, ConstShape3u32},
    MergeVoxel, Voxel, VoxelVisibility,
};
use serde::{Deserialize, Serialize};
use serde_big_array::BigArray;
use zstd_util::*;

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize)]

pub enum ClientMessage {
    Ping,
    RequestChunkData(IVec3),
}

// Render prototype
#[derive(PartialEq, Eq, Clone, Copy, Debug, Serialize, Deserialize)]
pub struct BlockType(u16);

impl Default for BlockType {
    fn default() -> Self {
        // Air
        BlockType(0)
    }
}

#[allow(unreachable_patterns)]

impl Voxel for BlockType {
    fn get_visibility(&self) -> VoxelVisibility {
        if self.0 == 0 {
            return VoxelVisibility::Empty;
        }

        VoxelVisibility::Opaque
    }
}

impl MergeVoxel for BlockType {
    type MergeValue = Self;

    fn merge_value(&self) -> Self::MergeValue {
        *self
    }
}

pub type CompressedChunk = Vec<u8>;
pub type ChunkShape = ConstShape3u32<18, 18, 18>;

// Used for chunk compression
pub const ZSTD_LVL: i32 = 22;

#[derive(Serialize, Deserialize, Debug, Resource)]
pub struct Chunk {
    pub position: IVec3,
    #[serde(with = "BigArray")]
    pub blocks: [BlockType; ChunkShape::SIZE as usize],
    pub modified: bool,
    pub loaded: bool,
}

#[derive(Debug, Resource, Default)]
pub struct ChunkRenderQueue(pub Vec<Chunk>);

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
        let mut zstd = ZstdContext::new(ZSTD_LVL, None);
        zstd.compress(&bytes)
            .expect("Failed to compress chunk packet")
    }
    pub fn from_compressed(bytes: CompressedChunk) -> Self {
        let mut zstd = ZstdContext::new(ZSTD_LVL, None);
        let decompressed = zstd.decompress(&bytes).unwrap();
        bincode::deserialize(&decompressed).unwrap()
    }
}

pub const AIR: BlockType = BlockType(0);

pub const DIRT: BlockType = BlockType(2);

pub const STONE: BlockType = BlockType(1);

#[derive(Debug, Serialize, Deserialize)]

pub struct ServerInfo {
    pub name: String,
    pub motd: String,

    pub player_count: u32,
    pub max_player_count: u32,
}

#[derive(Debug, Serialize, Deserialize)]

pub enum ServerMessage {
    Pong(ServerInfo),
    Chunk(CompressedChunk),
}

pub const PROTOCOL_ID: u64 = 1000;
