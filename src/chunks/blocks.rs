use crate::prelude::*;
use block_mesh::{MergeVoxel, Voxel, VoxelVisibility};

#[derive(PartialEq, Eq, Clone, Copy, Debug, Serialize, Deserialize)]
pub struct BlockType(pub u16);

pub const AIR: BlockType = BlockType(0);
pub const DIRT: BlockType = BlockType(2);
pub const STONE: BlockType = BlockType(1);

impl Default for BlockType {
    fn default() -> Self {
        // Air
        BlockType(0)
    }
}

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
