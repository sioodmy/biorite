use crate::prelude::*;

pub struct MyRaycastSet;
/// Report intersections
pub fn intersection(
    mut client: ResMut<RenetClient>,
    query: Query<&Intersection<MyRaycastSet>>,
    input: Res<Input<MouseButton>>,
) {
    if let Ok(intersection) = &query.get_single() {
        if input.just_pressed(MouseButton::Right) {
            if let Some(dis) = intersection.distance() {
                if dis <= REACH && dis > 1. {
                    if let Some(pos) = intersection.position() {
                        if let Some(normal) = intersection.normal() {
                            let x = if normal.x < 0. { -1. } else { 0. };
                            let y = if normal.y < 0. { -1. } else { 0. };
                            let z = if normal.z < 0. { -1. } else { 0. };
                            let target_block = IVec3::new(
                                (pos.x.floor() + x) as i32 - 1,
                                (pos.y.floor() + y) as i32 - 1,
                                (pos.z.floor() + z) as i32 - 1,
                            );

                            ClientMessage::PlaceBlock {
                                pos: target_block,
                                block: BlockType::Stone,
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
                    }
                }
            }
        }
    }
}
