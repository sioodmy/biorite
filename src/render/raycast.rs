use crate::prelude::*;

pub struct MyRaycastSet;
/// Report intersections
pub fn intersection(
    mut client: ResMut<RenetClient>,
    query: Query<&Intersection<MyRaycastSet>>,
    input: Res<Input<MouseButton>>,
) {
    if let Ok(intersection) = &query.get_single() {
        if input.just_pressed(MouseButton::Left) {
            if let Some(dis) = intersection.distance() {
                if dis <= REACH {
                    if let Some(pos) = intersection.position() {
                        let target_block = IVec3::new(
                            pos.x.round() as i32,
                            pos.y.round() as i32,
                            pos.z.round() as i32,
                        );
                        ClientMessage::BreakBlock(target_block)
                            .send(&mut client);
                        info!("{:?} {:?}", target_block, dis);
                    }
                }
            }
        }
    }
}
