use crate::prelude::*;

/// Resource storing the current chunk the player is in as well as its current
/// coords.
#[derive(Resource)]
pub struct CurrentLocalPlayerChunk {
    pub chunk_min: IVec3,
    pub world_pos: IVec3,
}

pub fn update_player_pos(
    player: Query<&GlobalTransform, (With<Camera>, Changed<GlobalTransform>)>,
    mut chunk_pos: ResMut<CurrentLocalPlayerChunk>,
) {
    if let Ok(ply) = player.get_single() {
        let player_coords = ply.translation().as_ivec3();
        let nearest_chunk_origin =
            !IVec3::splat((CHUNK_DIM - 1) as i32) & player_coords;

        chunk_pos.world_pos = player_coords;

        if chunk_pos.chunk_min != nearest_chunk_origin.into() {
            chunk_pos.chunk_min = nearest_chunk_origin.into();
        }
    }
}
