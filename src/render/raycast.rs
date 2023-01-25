use crate::prelude::*;

// TODO: Move
#[derive(Clone, Copy)]
pub struct ItemSlot(pub Option<BlockType>);

#[derive(Resource)]
pub struct HoldingItem(pub Option<BlockType>);

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
                ItemSlot(Some(BlockType::Bricks)),
                ItemSlot(Some(BlockType::Stone)),
                ItemSlot(Some(BlockType::Wood)),
                ItemSlot(Some(BlockType::Dirt)),
                ItemSlot(Some(BlockType::Sand)),
                ItemSlot(Some(BlockType::Bricks)),
                ItemSlot(Some(BlockType::Bricks)),
                ItemSlot(Some(BlockType::Bricks)),
                ItemSlot(Some(BlockType::Bricks)),
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

pub struct MyRaycastSet;
/// Report intersections
pub fn intersection(
    mut client: ResMut<RenetClient>,
    holding: Res<HoldingItem>,
    query: Query<&Intersection<MyRaycastSet>>,
    input: Res<Input<MouseButton>>,
) {
    if let Ok(intersection) = &query.get_single() {
        if let Some(dis) = intersection.distance() {
            if dis <= REACH && dis > 1. {
                if let Some(pos) = intersection.position() {
                    if let Some(normal) = intersection.normal() {
                        // Placing block
                        if input.just_pressed(MouseButton::Right) {
                            let x = if normal.x < 0. { -1. } else { 0. };
                            let y = if normal.y < 0. { -1. } else { 0. };
                            let z = if normal.z < 0. { -1. } else { 0. };
                            let target_block = IVec3::new(
                                (pos.x.floor() + x) as i32 - 1,
                                (pos.y.floor() + y) as i32 - 1,
                                (pos.z.floor() + z) as i32 - 1,
                            );

                            if let Some(block) = holding.0 {
                                ClientMessage::PlaceBlock {
                                    pos: target_block,
                                    block,
                                }
                                .send(&mut client);
                                info!(
                                    "{:?} {:?} {:?} {:?}",
                                    target_block,
                                    dis,
                                    pos,
                                    intersection.normal()
                                );
                            }
                        } else if input.just_pressed(MouseButton::Left) {
                            let target_block = IVec3::new(
                                pos.x.floor() as i32
                                    - 1
                                    - normal.x.abs() as i32,
                                pos.y.floor() as i32
                                    - 1
                                    - normal.y.abs() as i32,
                                pos.z.floor() as i32
                                    - 1
                                    - normal.z.abs() as i32,
                            );

                            ClientMessage::BreakBlock(target_block)
                                .send(&mut client);
                            info!(
                                "{:?} {:?} {:?} {:?}",
                                target_block,
                                dis,
                                pos,
                                intersection.normal()
                            );
                        }
                    }
                }
            }
        }
        // breaking block
    }
}
