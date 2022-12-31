use bevy::utils::FloatOrd;

use crate::prelude::*;

pub fn request_spawn_chunks(mut client: ResMut<RenetClient>, dist: i32) {
    let mut request = Vec::default();
    for x in -dist..=dist {
        for y in -dist..=dist {
            for z in -dist..=dist {
                if x * x + y * y + z * z <= dist * dist {
                    debug!("Requesting chunk at ({}, {}, {})", x, y, z);
                    request.push(IVec3::new(x, y, z));
                }
            }
        }
    }
    request.sort_by_key(|pos| FloatOrd(Vec3::distance(Vec3::ZERO, pos.as_vec3())));

    request.iter().for_each(|request| {
        if client.can_send_message(Channel::Reliable.id()) {
            info!("Requesting Chunk {:?}", request);
            ClientMessage::RequestChunk(*request).send(&mut client);
        }
    });
}

pub fn chunk_reciever(
    mut client: ResMut<RenetClient>,
    mut queue: ResMut<ChunkRenderQueue>,
    chunk_messages: Res<CurrentClientChunkMessages>,
) {
    for message in chunk_messages.iter() {
        if let ServerChunkMessage::Chunk(compressed_chunk) = message {
            let chunk = Chunk::from_compressed(compressed_chunk);
            info!("Got chunk at {:?}", &chunk.position);
            queue.0.push(chunk);
        }
    }
}

pub fn chunk_test(
    mut client: ResMut<RenetClient>,
    keyboard: Res<Input<KeyCode>>,
    messages: Res<CurrentClientMessages>,
) {
    if keyboard.just_pressed(KeyCode::Z) {
        info!("Requesting chunks");
        request_spawn_chunks(client, 6);
    }
}
