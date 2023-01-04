use bevy::utils::FloatOrd;
use bevy::utils::HashMap;

use crate::prelude::*;

#[derive(Resource)]
// pub struct RenderDistance(pub [Chunk; RENDER_DISTANCE as usize * RENDER_DISTANCE as usize]);
pub struct RenderDistance(pub HashMap<IVec3, bool>);

pub fn new_chunks(
    mut client: ResMut<RenetClient>,
    renderd: ResMut<RenderDistance>,
    q: Query<(&Player, &mut Transform)>,
) {
    for (_player, t) in q.iter() {
        debug!("{:?}", (t.translation.x / 16.0).round());
        let x = (t.translation.x / 16.0).round() as i32;
        let y = (t.translation.y / 16.0).round() as i32;
        let z = (t.translation.z / 16.0).round() as i32;
        let chunk_cords = IVec3::new(x, y, z);
        match renderd.0.get(&chunk_cords) {
            Some(is_loaded) => {
                if !*is_loaded {
                    continue;
                }
            }
            None => {
                debug!("unknown chunk");
                if client.can_send_message(Channel::Reliable.id()) {
                    info!("Requesting non spawn Chunk {:?}", &chunk_cords);
                    ClientMessage::RequestChunk(chunk_cords).send(&mut client);
                }
            }
        }
    }
}
pub fn request_spawn_chunks(mut client: ResMut<RenetClient>, renderd: ResMut<RenderDistance>) {
    let mut request = Vec::default();
    for x in -RENDER_DISTANCE..=RENDER_DISTANCE {
        for y in -RENDER_DISTANCE..=RENDER_DISTANCE {
            for z in -RENDER_DISTANCE..=RENDER_DISTANCE {
                if x * x + y * y + z * z <= RENDER_DISTANCE * RENDER_DISTANCE {
                    match renderd.0.get(&IVec3::new(x as i32, y as i32, z as i32)) {
                        Some(is_loaded) => {
                            if *is_loaded {
                                debug!("Chunk loaded, skipping");
                                continue;
                            }
                        }
                        None => (),
                    }

                    debug!("Requesting chunk at ({}, {}, {})", x, y, z);
                    request.push(IVec3::new(x as i32, y as i32, z as i32));
                }
            }
        }
    }

    request.sort_by_key(|pos| FloatOrd(Vec3::distance(Vec3::ZERO, pos.as_vec3())));

    if client.can_send_message(Channel::Reliable.id()) {
        info!("Requesting Chunk Batch {:?}", request);
        ClientMessage::RequestChunkBatch(request).send(&mut client);
    }
}

pub fn chunk_reciever(
    mut queue: ResMut<ChunkRenderQueue>,
    chunk_messages: Res<CurrentClientChunkMessages>,
) {
    for message in chunk_messages.iter() {
        #[allow(irrefutable_let_patterns)]
        if let ServerChunkMessage::Chunk(compressed_chunk) = message {
            let chunk = Chunk::from_compressed(compressed_chunk);
            info!("Got chunk at {:?}", &chunk.position);
            queue.0.push(chunk);
        }
        if let ServerChunkMessage::ChunkBatch(compressed_batch) = message {
            debug!("Chunk batch");
            compressed_batch.iter().for_each(|x| {
                queue.0.push(Chunk::from_compressed(x));
            });
        }
    }
}

pub fn chunk_test(
    client: ResMut<RenetClient>,
    renderd: ResMut<RenderDistance>,
    keyboard: Res<Input<KeyCode>>,
) {
    if keyboard.just_pressed(KeyCode::Z) {
        info!("Requesting chunks");
        request_spawn_chunks(client, renderd);
    }
}
