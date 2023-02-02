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
            let (tx, rx) = bounded(1000);
            if vec.len() > 512 {
                warn!("Client requested too many chunks. Disconnecting");
                server.disconnect(*id);
            }
            let chunk_sender = tx.clone();

            let mesh_sender = mtx.0.clone();
            if let Some(player_entity) = lobby.players.get(id) {
                let coords = query.get(*player_entity).expect("amogus");
                vec.iter().for_each(|pos| {
                    debug!("Validating request");

                    let player = coords.translation();
                    // FIXME: Actually its incorrect (I think)
                    let dx = pos.x as f32 - player.x / CHUNK_DIM as f32;
                    let dy = pos.y as f32 - player.y / CHUNK_DIM as f32;
                    let dz = pos.z as f32 - player.z / CHUNK_DIM as f32;

                    // https://en.wikipedia.org/wiki/Euclidean_distance
                    let distance = (dx * dx + dy * dy + dz * dz).sqrt();

                    debug!("Generating chunk at {:?}", pos);
                    let chunk: Chunk = match save.regions.get(&pos) {
                        Some(c) => match c.get(&pos) {
                            Some(mc) => *mc,
                            None => {
                                let chunk = chunk_generator(pos, save.seed);
                                save.insert_chunk(chunk);
                                save.save_region(*pos >> 5);
                                debug!("generating");
                                chunk
                            }
                        },
                        None => match save.load_region(*pos >> 5).get(pos) {
                            Some(lc) => *lc,
                            None => {
                                let chunk = chunk_generator(pos, save.seed);
                                save.insert_chunk(chunk);
                                save.save_region(*pos >> 5);
                                debug!("generating");
                                chunk
                            }
                        },
                    };
                    debug!("sending");
                    chunk_sender.send(chunk.compress()).unwrap();
                    mesh_sender
                        .send(QueuedChunk {
                            chunk,
                            is_new: true,
                        })
                        .unwrap();
                });
            }

            // let vector: Vec<CompressedChunk> = rx.try_iter().collect();
            for batch in
                rx.try_iter().collect::<Vec<CompressedChunk>>().chunks(5)
            {
                // debug!("Sending chunk batch with len {}", bat.len());
                ServerChunkMessage::ChunkBatch(batch.to_vec())
                    .send(&mut server, *id);
            }
            // let vectors: Vec<Vec<CompressedChunk>> = .drain().collect();
        }
    }
}
