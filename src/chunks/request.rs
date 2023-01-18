use crate::prelude::*;

/// Avoid requesting same chunk twice
#[derive(Resource, Default)]
pub struct AlreadyRequested(pub Vec<IVec3>);

/// Push received chunks from the server into mesher
pub fn receive_chunk(
    mut chunk_messages: ResMut<CurrentClientChunkMessages>,
    mut mesh_queue: ResMut<MeshQueue>,
) {
    for message in chunk_messages.drain(..) {
        if let ServerChunkMessage::ChunkBatch(compressed_batch) = message {
            for compressed_chunk in compressed_batch.iter() {
                let chunk = Chunk::from_compressed(compressed_chunk);
                mesh_queue.0.push(chunk)
            }
        }
    }
}

/// Request chunks around the player
/// To save bandwidth we only send cube boundaries
/// as `[IVec3; 4]`
pub fn request_chunk(
    players: Query<&GlobalTransform, &ControlledPlayer>,
    mut client: ResMut<RenetClient>,
    mut sent: ResMut<AlreadyRequested>,
) {
    if let Ok(ply) = players.get_single() {
        let nearest_chunk_origin = !IVec3::splat((CHUNK_DIM - 1) as i32)
            & ply.translation().as_ivec3();
        let chunk_x = nearest_chunk_origin.x / CHUNK_DIM as i32;
        let chunk_y = nearest_chunk_origin.y / CHUNK_DIM as i32;
        let chunk_z = nearest_chunk_origin.z / CHUNK_DIM as i32;

        let mut to_request = Vec::new();
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
                    if !sent.0.contains(&chunk) {
                        to_request.push(chunk);
                        sent.0.push(chunk);
                    }
                }
            }
        }

        if !to_request.is_empty() {
            ClientMessage::RequestChunk(to_request).send(&mut client);
            info!("requesting");
        }
    }
}

pub fn client_chunk_despawner(
    mut commands: Commands,
    mut loaded_chunks: ResMut<LoadedChunks>,
    mut sent: ResMut<AlreadyRequested>,
    player_query: Query<&GlobalTransform, With<Player>>,
) {
    // List of chunks that we actually need
    let mut relevant = Vec::new();
    for player in player_query.iter() {
        let player_coords = player.translation().as_ivec3();
        // Nearest chunk origin
        let no = !IVec3::splat((CHUNK_DIM - 1) as i32) & player_coords;
        let chunk_x = no.x / CHUNK_DIM as i32;
        let chunk_y = no.y / CHUNK_DIM as i32;
        let chunk_z = no.z / CHUNK_DIM as i32;
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
                    relevant.push(chunk);
                }
            }
        }
    }

    loaded_chunks
        .0
        .drain_filter(|pos, _| !relevant.contains(pos))
        .for_each(|(_, entry)| commands.entity(entry.entity).despawn());
    loaded_chunks.0.shrink_to_fit();
    sent.0.retain(|chunk| relevant.contains(chunk));
}
