use bevy::prelude::*;
use bevy_mod_raycast::Intersection;
use biorite_generator::blocks::BlockType;
use biorite_shared::net::protocol::{ClientMessage, RenetClient};

// TODO: Move
use biorite_shared::consts::REACH;

#[allow(dead_code)]
#[derive(Clone, Copy)]
pub enum Item {
    Block { block: BlockType, stackable: bool },
    Misc { item: u64, stackable: bool },
}

#[derive(Clone, Copy)]
pub struct ItemSlot(pub Option<Item>);

#[derive(Resource)]
pub struct HoldingItem(pub Option<Item>);

#[derive(Resource)]
pub struct Hotbar {
    slots: [ItemSlot; 9],
    selected: u8,
}

impl Default for Hotbar {
    fn default() -> Self {
        Hotbar {
            slots: [ItemSlot(None); 9],
            selected: 1,
        }
    }
}

impl Hotbar {
    pub fn debug() -> Self {
        Hotbar {
            slots: [
                ItemSlot(Some(Item::Block {
                    block: BlockType::Bricks,
                    stackable: true,
                })),
                ItemSlot(Some(Item::Block {
                    block: BlockType::Stone,
                    stackable: true,
                })),
                ItemSlot(Some(Item::Block {
                    block: BlockType::Wood,
                    stackable: true,
                })),
                ItemSlot(Some(Item::Block {
                    block: BlockType::Bricks,
                    stackable: true,
                })),
                ItemSlot(Some(Item::Block {
                    block: BlockType::Bricks,
                    stackable: true,
                })),
                ItemSlot(Some(Item::Block {
                    block: BlockType::Bricks,
                    stackable: true,
                })),
                ItemSlot(Some(Item::Block {
                    block: BlockType::Bricks,
                    stackable: true,
                })),
                ItemSlot(Some(Item::Block {
                    block: BlockType::Bricks,
                    stackable: true,
                })),
                ItemSlot(Some(Item::Block {
                    block: BlockType::Bricks,
                    stackable: true,
                })),
            ],
            selected: 1,
        }
    }
}

pub fn hotbar_prototype(
    mut hotbar: ResMut<Hotbar>,
    input: Res<Input<KeyCode>>,
) {
    if input.just_pressed(KeyCode::Key1) {
        hotbar.selected = 0;
    }
    if input.just_pressed(KeyCode::Key2) {
        hotbar.selected = 1;
    }
    if input.just_pressed(KeyCode::Key3) {
        hotbar.selected = 2;
    }
    if input.just_pressed(KeyCode::Key4) {
        hotbar.selected = 3;
    }
}

pub fn holding_item(hotbar: ResMut<Hotbar>, mut holding: ResMut<HoldingItem>) {
    holding.0 = hotbar.slots[hotbar.selected as usize].0;
}

pub struct ChunkRaycast;
/// Report intersections
pub fn intersection(
    mut client: ResMut<RenetClient>,
    holding: Res<HoldingItem>,
    query: Query<&Intersection<ChunkRaycast>>,
    input: Res<Input<MouseButton>>,
) {
    for intersection in query.iter() {
        if let (Some(dis), Some(pos), Some(normal)) = (
            intersection.distance(),
            intersection.position(),
            intersection.normal(),
        ) {
            // Placing block
            if input.just_pressed(MouseButton::Right) {
                if dis <= REACH && dis > 1. {
                    let x = if normal.x < 0. { -1. } else { 0. };
                    let y = if normal.y < 0. { -1. } else { 0. };
                    let z = if normal.z < 0. { -1. } else { 0. };
                    let target_block = IVec3::new(
                        (pos.x.floor() + x) as i32 - 1,
                        (pos.y.floor() + y) as i32 - 1,
                        (pos.z.floor() + z) as i32 - 1,
                    );

                    if let Some(Item::Block { block, .. }) = holding.0 {
                        ClientMessage::PlaceBlock {
                            pos: target_block,
                            block,
                        }
                        .send(&mut client);
                    }
                }
            } else if input.just_pressed(MouseButton::Left) {
                info!("{:?}", normal);
                let target_block = IVec3::new(
                    pos.x.floor() as i32 - 1 - normal.x as i32,
                    pos.y.floor() as i32 - 1 - normal.y as i32,
                    pos.z.floor() as i32 - 1 - normal.z as i32,
                );

                // ClientMessage::BreakBlock(IVec3::new(
                //     target_block.x.floor() as i32,
                //     target_block.y.floor() as i32,
                //     target_block.z.floor() as i32,
                // ))
                // .send(&mut client);
                ClientMessage::BreakBlock(target_block).send(&mut client);
            }
        }
        // breaking block
    }
}
