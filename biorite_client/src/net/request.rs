use biorite_generator::MeshQueueSender;

use super::*;
use bevy::prelude::*;
use biorite_generator::{
    chunk::{Chunk, CHUNK_DIM},
    QueuedChunk,
};
use biorite_shared::{
    consts::*,
    net::{data_types::ControlledPlayer, protocol::*},
};
use rayon::prelude::*;

/// Avoid requesting same chunk twice
#[derive(Resource, Default)]
pub struct AlreadyRequested(pub Vec<IVec3>);

/// Push received chunks from the server into mesher
pub fn receive_chunk(
    mut chunk_messages: ResMut<CurrentClientChunkMessages>,
    mesh_queue: Res<MeshQueueSender>,
) {
    for message in chunk_messages.drain(..) {
        if let ServerChunkMessage::ChunkBatch(compressed_batch) = message {
            compressed_batch.par_iter().for_each(|compressed_chunk| {
                let chunk = Chunk::from_compressed(compressed_chunk);
                mesh_queue
                    .0
                    .clone()
                    .send(QueuedChunk {
                        chunk,
                        // TODO: chagnne
                        is_new: false,
                    })
                    .unwrap();
            });
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
        for x in (chunk_x - RENDER_DISTANCE)..=(chunk_x + RENDER_DISTANCE) {
            for y in (chunk_y - RENDER_DISTANCE)..=(chunk_y + RENDER_DISTANCE) {
                for z in
                    (chunk_z - RENDER_DISTANCE)..=(chunk_z + RENDER_DISTANCE)
                {
                    let chunk = IVec3::new(x, y, z);
                    if !sent.0.contains(&chunk) {
                        to_request.push(chunk);
                        sent.0.push(chunk);
                    }
                }
            }
        }

        if !to_request.is_empty() {
            let limit = usize::min(to_request.len(), REQUEST_LIMIT);
            let batch = to_request.drain(..limit).collect();
            ClientMessage::RequestChunk(batch).send(&mut client);
            info!("requesting");
        }
    }
}
