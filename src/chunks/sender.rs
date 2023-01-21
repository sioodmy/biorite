use crate::prelude::*;

use std::sync::mpsc::channel;

/// Send chunks to the client
pub fn chunk_send(
    mut server: ResMut<RenetServer>,
    mut queue: ResMut<MeshQueue>,
    msg: Res<CurrentServerMessages>,
    // query: Query<&mut GlobalTransform, Changed<PlayerInput>>,
) {
    for (id, message) in msg.iter() {
        if let ClientMessage::RequestChunk(vec) = message {
            // TODO: Validate each request
            let (tx, rx) = channel();
            for pos in vec.iter() {
                info!("Generating chunk at {:?}", pos);
                let sender = tx.clone();
                let chunk = chunk_generator(pos);
                sender.clone().send(chunk.compress()).unwrap();
                queue.0.push(chunk);
            }
            let vector: Vec<CompressedChunk> = rx.try_iter().collect();

            ServerChunkMessage::ChunkBatch(vector).send(&mut server, *id);
        }
    }
}
