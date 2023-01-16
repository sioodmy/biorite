use crate::prelude::*;
use std::sync::mpsc::channel;

pub fn chunk_send(
    mut server: ResMut<RenetServer>,
    mut lobby: ResMut<Lobby>,

    query: Query<&mut GlobalTransform, Changed<PlayerInput>>,
) {
    if let Ok(ply) = query.get_single() {
        let player_coords = ply.translation().as_ivec3();
        let nearest_chunk_origin =
            !IVec3::splat((CHUNK_DIM - 1) as i32) & player_coords;

        let chunk_x = nearest_chunk_origin.x / CHUNK_DIM as i32;
        let chunk_y = nearest_chunk_origin.y / CHUNK_DIM as i32;
        let chunk_z = nearest_chunk_origin.z / CHUNK_DIM as i32;

        let mut chunks_to_send = Vec::new();

        for player_id in lobby.players.clone().keys() {
            for x in (chunk_x - RENDER_DISTANCE as i32)
                ..=(chunk_x + RENDER_DISTANCE as i32)
            {
                for y in (chunk_y - RENDER_DISTANCE as i32)
                    ..=(chunk_y + RENDER_DISTANCE as i32)
                {
                    for z in (chunk_z - RENDER_DISTANCE as i32)
                        ..=(chunk_z + RENDER_DISTANCE as i32)
                    {
                        let chunk = IVec3::new(x as i32, y as i32, z as i32);
                        if !lobby
                            .sent_chunks
                            .entry(*player_id)
                            .or_default()
                            .contains(&chunk)
                        {
                            chunks_to_send.push(chunk);
                        }
                    }
                }
            }
            if !chunks_to_send.is_empty() {
                debug!("cum");
                let (tx, rx) = channel();
                for chunk in chunks_to_send.iter() {
                    lobby
                        .sent_chunks
                        .entry(*player_id)
                        .or_default()
                        .push(*chunk);
                    tx.send(chunk_generator(chunk).compress()).unwrap();
                }
                let vector: Vec<CompressedChunk> = rx.try_iter().collect();
                ServerChunkMessage::ChunkBatch(vector)
                    .send(&mut server, *player_id);
            }
        }
    }
}