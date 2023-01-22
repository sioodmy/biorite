use crate::prelude::*;
use block_mesh::{MergeVoxel, Voxel, VoxelVisibility};

pub struct BlockTexture {
    top: i32,
    side: i32,
    bottom: i32,
}

impl BlockTexture {
    pub fn full(id: i32) -> Self {
        BlockTexture {
            top: id,
            side: id,
            bottom: id,
        }
    }
    pub fn new(top: i32, side: i32, bottom: i32) -> Self {
        BlockTexture { top, side, bottom }
    }

    pub fn from_direction(&self, dir: BlockFace) -> i32 {
        use BlockFace::*;
        match dir {
            Top => self.top,
            Bottom => self.bottom,
            Side => self.side,
        }
    }
}

pub enum BlockFace {
    Top,
    Bottom,
    Side,
}

impl BlockFace {
    pub fn from_normal(normal: [[f32; 3]; 4]) -> Self {
        if normal
            == [
                [0.0, 1.0, 0.0],
                [0.0, 1.0, 0.0],
                [0.0, 1.0, 0.0],
                [0.0, 1.0, 0.0],
            ]
        {
            return BlockFace::Top;
        }
        if normal
            == [
                [0.0, -1.0, 0.0],
                [0.0, -1.0, 0.0],
                [0.0, -1.0, 0.0],
                [0.0, -1.0, 0.0],
            ]
        {
            return BlockFace::Bottom;
        }
        BlockFace::Side
    }
}
#[derive(
    PartialEq, Eq, Clone, Copy, Debug, Serialize, Deserialize, Default,
)]
pub enum BlockType {
    #[default]
    Air,
    Grass,
    Dirt,
    Stone,
    Sand,
    Wood,
}

impl BlockType {
    pub fn get_texture(&self) -> BlockTexture {
        use BlockType::*;
        match *self {
            Air => unreachable!(),
            Grass => BlockTexture::new(0, 1, 2),
            Dirt => BlockTexture::full(2),
            Stone => BlockTexture::full(3),
            Sand => BlockTexture::full(4),
            Wood => BlockTexture::new(6, 5, 6),
        }
    }
}
impl Voxel for BlockType {
    fn get_visibility(&self) -> VoxelVisibility {
        if self == &BlockType::Air {
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
