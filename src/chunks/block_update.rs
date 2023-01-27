use crate::prelude::*;

pub fn handle_block_updates(
    msg: Res<CurrentServerMessages>,
    mut save: ResMut<SaveFile>,
    mut server: ResMut<RenetServer>,
) {
    for (id, message) in msg.iter() {
        if let ClientMessage::BreakBlock(block) = message {
            debug!("got break block packet at {:?} from {}", block, id);

            let x = block.x.div_euclid(CHUNK_DIM as i32);
            let y = block.y.div_euclid(CHUNK_DIM as i32);
            let z = block.z.div_euclid(CHUNK_DIM as i32);

            let r_x = block.x.rem_euclid(CHUNK_DIM as i32) + 1;
            let r_y = block.y.rem_euclid(CHUNK_DIM as i32) + 1;
            let r_z = block.z.rem_euclid(CHUNK_DIM as i32) + 1;

            let chunk_pos = IVec3::new(x, y, z);
            if let Some(mut chunk) = save.modify_chunk(chunk_pos) {
                info!("modifying chunk");
                let i = ChunkShape::linearize([
                    r_x.try_into().unwrap(),
                    r_y.try_into().unwrap(),
                    r_z.try_into().unwrap(),
                ]);
                if chunk.blocks[i as usize] != BlockType::Air {
                    chunk.blocks[i as usize] = BlockType::Air
                } else {
                    warn!("Incorrect block break packet from client {}", id);
                }
                ServerMessage::BlockDelta {
                    pos: *block,
                    block: BlockType::Air,
                }
                .broadcast(&mut server);
                info!("saving chunk");
                save.save_region(chunk_pos >> REGION_DIM);
            }
        }
        if let ClientMessage::PlaceBlock { pos, block } = message {
            debug!("Got block place packet at {:?}, from {}", block, id);
            let x = pos.x.div_euclid(CHUNK_DIM as i32);
            let y = pos.y.div_euclid(CHUNK_DIM as i32);
            let z = pos.z.div_euclid(CHUNK_DIM as i32);

            let r_x = pos.x.rem_euclid(CHUNK_DIM as i32) + 1;
            let r_y = pos.y.rem_euclid(CHUNK_DIM as i32) + 1;
            let r_z = pos.z.rem_euclid(CHUNK_DIM as i32) + 1;

            let chunk_pos = IVec3::new(x, y, z);
            if let Some(mut chunk) = save.modify_chunk(chunk_pos) {
                info!("modifying chunk");
                // Avoid replacing already existing blocks
                let i = ChunkShape::linearize([
                    r_x.try_into().unwrap(),
                    r_y.try_into().unwrap(),
                    r_z.try_into().unwrap(),
                ]);
                if chunk.blocks[i as usize] == BlockType::Air {
                    chunk.blocks[i as usize] = *block;
                } else {
                    warn!("Client {} tried to replace existing block", id);
                }
                ServerMessage::BlockDelta {
                    pos: *pos,
                    block: *block,
                }
                .broadcast(&mut server);
                info!("saving chunk");
                save.save_region(chunk_pos >> REGION_DIM);
            }
        }
    }
}

pub fn client_block_updates(
    msg: Res<CurrentClientMessages>,
    mut chunks: ResMut<LoadedChunks>,
    mesh_queue: ResMut<MeshQueueSender>,
) {
    for message in msg.iter() {
        if let ServerMessage::BlockDelta { pos, block } = message {
            info!("Got block delta at {:?} {:?}", pos, block);
            // TODO: rewrite all of this shit
            // Chunk coords
            let x = pos.x.div_euclid(CHUNK_DIM as i32);
            let y = pos.y.div_euclid(CHUNK_DIM as i32);
            let z = pos.z.div_euclid(CHUNK_DIM as i32);

            let r_x = pos.x.rem_euclid(CHUNK_DIM as i32) + 1;
            let r_y = pos.y.rem_euclid(CHUNK_DIM as i32) + 1;
            let r_z = pos.z.rem_euclid(CHUNK_DIM as i32) + 1;

            chunks.0.entry(IVec3::new(x, y, z)).and_modify(|e| {
                e.chunk.blocks[ChunkShape::linearize([
                    r_x.try_into().unwrap(),
                    r_y.try_into().unwrap(),
                    r_z.try_into().unwrap(),
                ]) as usize] = *block;
                mesh_queue
                    .0
                    .send(QueuedChunk {
                        chunk: e.chunk,
                        is_new: false,
                    })
                    .unwrap();
            });
        };
    }
}
