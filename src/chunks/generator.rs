use crate::prelude::*;
use bracket_noise::prelude::*;

pub fn chunk_sender(
    messages: Res<CurrentServerMessages>,
    mut server: ResMut<RenetServer>,
    mut queued_requests: Local<Vec<(u64, IVec3)>>,
) {
    queued_requests.retain(|(id, position)| {
        if server.can_send_message(*id, Channel::Chunk.id()) {
            for (client_id, message) in messages.iter() {
                debug!("Client {} requested chunk at {:?}", client_id, position);
                let chunk = chunk_generator(*position).compress();
                debug!("Packet size {}", chunk.len() * 8);
                ServerChunkMessage::Chunk(chunk).send(&mut server, *client_id);
                return false;
            }
        }
        true
    });

    for message in messages.iter() {
        if let (id, ClientMessage::RequestChunk(position)) = message {
            if server.can_send_message(*id, Channel::Chunk.id()) {
                debug!("Sending Chunk! {}", position);
                let chunk = chunk_generator(*position).compress();
                ServerChunkMessage::Chunk(chunk).send(&mut server, *id);
            } else {
                queued_requests.push((*id, *position));
            }
        }
    }
}

pub fn chunk_generator(position: IVec3) -> Chunk {
    // placeholder for propper chunk generation
    let mut blocks: [BlockType; ChunkShape::SIZE as usize] = [AIR; ChunkShape::SIZE as usize];

    // TODO: propper seed handling
    // TODO: async chunk generation

    info!("Generating chunk {:?}", &position);
    let mut noise = FastNoise::seeded(2137);
    noise.set_noise_type(NoiseType::Perlin);
    noise.set_fractal_octaves(3);
    noise.set_fractal_gain(0.06);
    noise.set_fractal_lacunarity(0.25);
    noise.set_frequency(0.07);

    let offset = 8.0;
    let factor = 7.37;
    let flat: f64 = 5.0;

    for x in 1..17 {
        for z in 1..17 {
            for y in 1..17 {
                // Global cords
                let gx = position.x as f32 * 16.0 + x as f32;
                let gy = position.y as f32 * 16.0 + y as f32;
                let gz = position.z as f32 * 16.0 + z as f32;

                let noise = noise.get_noise(gx as f32, gz as f32) * factor + offset;

                let surface = flat as f64 + noise as f64;
                let i = ChunkShape::linearize([x, y, z]);
                if gy as f64 > surface {
                    blocks[i as usize] = AIR;
                } else {
                    blocks[i as usize] = DIRT;
                }
            }
        }
    }

    Chunk {
        position,
        blocks,
        ..Default::default()
    }
}
