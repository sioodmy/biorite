use crate::prelude::*;

pub fn handle_block_updates(
    msg: Res<CurrentServerMessages>,
    mut save: ResMut<SaveFile>,
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
            let x = pos.x.div_euclid(CHUNK_DIM as i32);
            let y = pos.y.div_euclid(CHUNK_DIM as i32);
            let z = pos.z.div_euclid(CHUNK_DIM as i32);

            let r_x = pos.x.rem_euclid(CHUNK_DIM as i32) + 1;
            let r_y = pos.y.rem_euclid(CHUNK_DIM as i32) + 1;
            let r_z = pos.z.rem_euclid(CHUNK_DIM as i32) + 1;

            let chunk_pos = IVec3::new(x, y, z);
            if let Some(mut chunk) = save.modify_chunk(chunk_pos) {
                info!("modifying chunk");
                chunk.blocks[ChunkShape::linearize([
                    r_x.try_into().unwrap(),
                    r_y.try_into().unwrap(),
                    r_z.try_into().unwrap(),
                ]) as usize] = *block;
                ServerMessage::BlockDelta {
                    pos: *pos,
                    block: *block,
                }
                .send(&mut server, *id);
                info!("saving chunk");
                save.save_region(chunk_pos >> REGION_DIM);
            } else {
                warn!("couldnt get chunk");
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
