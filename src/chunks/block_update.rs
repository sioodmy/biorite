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
        if let ClientMessage::PlaceBlock { pos, block } = message {
            debug!("Got block place packet at {:?}, from {}", block, id);
            ServerMessage::BlockDelta {
                pos: *pos,
                block: *block,
            }
            .send(&mut server, *id);
        }
    }
}

pub fn client_block_updates(
    msg: Res<CurrentClientMessages>,
    mut chunks: ResMut<LoadedChunks>,
    mut mesh_queue: ResMut<MeshQueueSender>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    loading_texture: Res<LoadingTexture>,
) {
    for message in msg.iter() {
        if let ServerMessage::BlockDelta { pos, block } = message {
            info!("Got block delta at {:?} {:?}", pos, block);
            // TODO: rewrite all of this shit
            // Chunk coords
            let x: i32 = pos.x / CHUNK_DIM as i32;
            let y: i32 = pos.y / CHUNK_DIM as i32;
            let z: i32 = pos.z / CHUNK_DIM as i32;

            info!("chunk {}/{}/{}", x, y, z);

            let x = pos.x.div_euclid(CHUNK_DIM as i32);
            let y = pos.y.div_euclid(CHUNK_DIM as i32);
            let z = pos.z.div_euclid(CHUNK_DIM as i32);

            let r_x = pos.x.rem_euclid(CHUNK_DIM as i32) + 1;
            let r_y = pos.y.rem_euclid(CHUNK_DIM as i32) + 1;
            let r_z = pos.z.rem_euclid(CHUNK_DIM as i32) + 1;

            info!("relative to chunk {}/{}/{}", r_x, r_y, r_z);

            if let Some(entry) =
                chunks.0.get_mut(&IVec3::new(x as i32, y as i32, z as i32))
            {
                let i = ChunkShape::linearize([
                    r_x.try_into().unwrap(),
                    r_y.try_into().unwrap(),
                    r_z.try_into().unwrap(),
                ]);
                // TODO
                let mut newchunk = entry.chunk;
                newchunk.blocks[i as usize] = BlockType::Stone;
                mesh_queue.0.send(newchunk).unwrap();
            };
        };
    }
}
