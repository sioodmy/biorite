use crate::prelude::*;

pub fn handle_block_updates(
    msg: Res<CurrentServerMessages>,
    mut server: ResMut<RenetServer>,
) {
    for (id, message) in msg.iter() {
        if let ClientMessage::BreakBlock(block) = message {
            debug!("got break block packet at {:?} from {}", block, id);

            ServerMessage::BlockDelta {
                pos: *block,
                block: BlockType::Air,
            }
            .send(&mut server, *id);
        }
    }
}

pub fn client_block_updates(
    msg: Res<CurrentClientMessages>,
    mut chunks: ResMut<LoadedChunks>,
    _commands: Commands,
) {
    for message in msg.iter() {
        if let ServerMessage::BlockDelta { pos, block } = message {
            info!("Got block delta at {:?} {:?}", pos, block);
            // Chunk coords
            let x: i32 = pos.x / CHUNK_DIM as i32;
            let y: i32 = pos.y / CHUNK_DIM as i32;
            let z: i32 = pos.z / CHUNK_DIM as i32;
            info!("chunk {}/{}/{}", x, y, z);

            let r_x = pos.x.abs() - x.abs() * CHUNK_DIM as i32;
            let r_y = pos.y.abs() - y.abs() * CHUNK_DIM as i32;
            let r_z = pos.z.abs() - z.abs() * CHUNK_DIM as i32;
            info!("relative to chunk {}/{}/{}", r_x, r_y, r_z);

            if let Some(entry) = chunks.0.get_mut(&IVec3::new(x, y, z)) {
                let i = ChunkShape::linearize([
                    r_x.try_into().unwrap(),
                    r_y.try_into().unwrap(),
                    r_z.try_into().unwrap(),
                ]);
                entry.chunk.blocks[i as usize] = *block;
            };
        };
    }
}
