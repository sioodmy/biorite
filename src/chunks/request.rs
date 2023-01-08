use bevy::utils::HashMap;

use crate::prelude::*;

#[derive(Resource)]
// pub struct RenderDistance(pub [Chunk; RENDER_DISTANCE as usize *
// RENDER_DISTANCE as usize]);
pub struct RenderDistance(pub HashMap<IVec3, (bool, Entity)>);

pub fn chunk_receiver(
    mut queue: ResMut<ChunkRenderQueue>,
    chunk_messages: Res<CurrentClientChunkMessages>,
) {
    for message in chunk_messages.iter() {
        if let ServerChunkMessage::ChunkBatch(compressed_batch) = message {
            debug!("Chunk batch");
            compressed_batch.iter().for_each(|x| {
                queue.0.push(Chunk::from_compressed(x));
            });
        }
    }
}
