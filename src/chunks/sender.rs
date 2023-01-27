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
    // query: Query<&mut GlobalTransform, Changed<PlayerInput>>,
) {
    for (id, message) in msg.iter() {
        if let ClientMessage::RequestChunk(vec) = message {
            // TODO: Validate each request
            // TODO: Queue chunk pending requests so that server sends chunks
            // evenly
            let (tx, rx) = bounded(1000);
            if vec.len() > 512 {
                warn!("Client requested too many chunks. Disconnecting");
                server.disconnect(*id);
            }
            let chunk_sender = tx.clone();

            let mesh_sender = mtx.0.clone();
            if let Some(player_entity) = lobby.players.get(id) {
                let coords = query.get(*player_entity).expect("amogus");
                vec.par_iter().for_each(move |pos| {
                    debug!("Validating request");

                    let player = coords.translation();
                    // FIXME: Actually its incorrect (I think)
                    let dx = pos.x as f32 - player.x / CHUNK_DIM as f32;
                    let dy = pos.y as f32 - player.y / CHUNK_DIM as f32;
                    let dz = pos.z as f32 - player.z / CHUNK_DIM as f32;

                    // https://en.wikipedia.org/wiki/Euclidean_distance
                    let distance = (dx * dx + dy * dy + dz * dz).sqrt();

                    if distance < 2. * (RENDER_DISTANCE + 1) as f32 {
                        debug!("Generating chunk at {:?}", pos);
                        let chunk = chunk_generator(pos);
                        mesh_sender
                            .send(QueuedChunk {
                                chunk,
                                is_new: true,
                            })
                            .unwrap();
                        chunk_sender.send(chunk.compress()).unwrap();
                    } else {
                        warn!(
                            "Client {} tried requesting chunk too far away",
                            id
                        );
                    }
                });
            }

            let vector: Vec<CompressedChunk> = rx.try_iter().collect();

            debug!("Sending chunk batch with len {}", vector.len());
            ServerChunkMessage::ChunkBatch(vector).send(&mut server, *id);
        }
    }
}
