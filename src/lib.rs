pub use bevy::prelude::*;
pub use bevy_renet::{renet::*, *};
use block_mesh::{ndshape::ConstShape3u32, MergeVoxel, Voxel, VoxelVisibility};
use serde::{Deserialize, Serialize};

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize)]

pub enum ClientMessage {
    Ping,
}

// Render prototype
#[derive(PartialEq, Eq, Clone, Copy, Debug)]

pub enum BlockType {
    Air,
    Dirt,
    Stone,
}

impl Default for BlockType {
    fn default() -> Self {
        BlockType::Air
    }
}

#[allow(unreachable_patterns)]

impl Voxel for BlockType {
    fn get_visibility(&self) -> VoxelVisibility {
        match self {
            BlockType::Air => VoxelVisibility::Empty,
            BlockType::Dirt => VoxelVisibility::Opaque,

            _ => VoxelVisibility::Opaque,
        }
    }
}

impl MergeVoxel for BlockType {
    type MergeValue = Self;

    fn merge_value(&self) -> Self::MergeValue {
        *self
    }
}

pub type ChunkShape = ConstShape3u32<18, 18, 18>;

// This chunk will cover just a single octant of a
// sphere SDF (radius 15).

pub const AIR: BlockType = BlockType::Air;

pub const DIRT: BlockType = BlockType::Dirt;

pub const STONE: BlockType = BlockType::Stone;

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
}

pub const PROTOCOL_ID: u64 = 1000;
