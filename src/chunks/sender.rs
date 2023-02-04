use crate::prelude::*;
use rayon::prelude::*;

// pub struct Pending(HashMap<u64, IVec3>);

/// Send chunks to the client
pub fn chunk_send(
    mut server: ResMut<RenetServer>,
    mtx: ResMut<MeshQueueSender>,
    msg: Res<CurrentServerMessages>,
    lobby: Res<Lobby>,
    query: Query<&GlobalTransform, With<Player>>,
    mut modified: ResMut<ModifiedRegions>,
    mut save: ResMut<SaveFile>,
    // query: Query<&mut GlobalTransform, Changed<PlayerInput>>,
) {
    for (id, message) in msg.iter() {
        if let ClientMessage::RequestChunk(vec) = message {
            // TODO: Validate each request
            // TODO: Queue chunk pending requests so that server sends chunks
            // evenly
            if vec.len() > 512 {
                warn!("Client requested too many chunks. Disconnecting");
                server.disconnect(*id);
            }

            let mesh_sender = mtx.0.clone();
            if let Some(player_entity) = lobby.players.get(id) {
                let coords = query.get(*player_entity).expect("amogus");
                let chunks =
                    save.get_compressed_chunk(vec.to_vec(), mesh_sender);
                for batch in chunks.chunks(5) {
                    // debug!("Sending chunk batch with len {}", bat.len());
                    ServerChunkMessage::ChunkBatch(batch.to_vec())
                        .send(&mut server, *id);
                }
            }

            // let vector: Vec<CompressedChunk> = rx.try_iter().collect();
            // let vectors: Vec<Vec<CompressedChunk>> = .drain().collect();
        }
    }
}
