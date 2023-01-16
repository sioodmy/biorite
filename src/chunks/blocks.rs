use crate::prelude::*;
use block_mesh::{MergeVoxel, Voxel, VoxelVisibility};

#[derive(
    PartialEq, Eq, Clone, Copy, Debug, Serialize, Deserialize, Default,
)]
pub struct BlockType(pub u16);

pub const AIR: BlockType = BlockType(0);
pub const GRASS: BlockType = BlockType(1);
pub const DIRT: BlockType = BlockType(2);
pub const STONE: BlockType = BlockType(3);
pub const SAND: BlockType = BlockType(4);

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
    type MergeValueFacingNeighbour = bool;

    fn merge_value(&self) -> Self::MergeValue {
        *self
    }
    fn merge_value_facing_neighbour(&self) -> Self::MergeValueFacingNeighbour {
        // TODO
        true
    }
}
