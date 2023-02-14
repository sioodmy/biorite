use super::{chunk_generator, QueuedChunk};
use crate::{chunk::*, REGION_DIM};
use bevy::{prelude::*, utils::HashMap};
use crossbeam_channel::{bounded, Sender};
use rayon::prelude::*;
use scc::HashSet;
use std::fs;
/// Optimized world save storage format
/// Chunks are now stored in 32x32x32 regions.
/// Its more efficient to store them that way
/// Inspired by amazing minecraft mod made by Scaevolus
#[derive(Resource)]
pub struct SaveFile {
    pub name: String,
    pub regions: HashMap<IVec3, HashMap<IVec3, Chunk>>,
    pub seed: u64,
    pub dirty: HashSet<IVec3>,
}
impl Default for SaveFile {
    fn default() -> Self {
        SaveFile {
            name: "New world".into(),
            regions: HashMap::new(),
            seed: 2137,
            dirty: HashSet::new(),
        }
    }
}

impl SaveFile {
    pub fn insert_chunk(&mut self, chunk: Chunk) {
        let region_pos = chunk.position >> REGION_DIM;
        info!(
            "inserting chunk {:?} at region {:?}",
            chunk.position, region_pos
        );
        self.regions
            .entry(region_pos)
            .or_default()
            .entry(chunk.position)
            .or_insert(chunk);
    }

    pub fn save_region(&self, region_pos: IVec3) {
        if let Some(region) = self.regions.get(&region_pos) {
            let bytes = bincode::serialize(region).unwrap();
            fs::write(
                format!(
                    "world/regions/r.{}.{}.{}.cum",
                    region_pos.x, region_pos.y, region_pos.z
                ),
                bytes,
            )
            .unwrap();
        }
    }

    pub fn load_region(&mut self, region_pos: IVec3) -> HashMap<IVec3, Chunk> {
        if let Ok(bytes) = fs::read(format!(
            "world/regions/r.{}.{}.{}.cum",
            region_pos.x, region_pos.y, region_pos.z
        )) {
            bincode::deserialize(&bytes).unwrap()
        } else {
            HashMap::new()
        }
    }

    pub fn modify_chunk(&mut self, chunk_pos: IVec3) -> Option<&mut Chunk> {
        self.regions
            .get_mut(&(chunk_pos >> REGION_DIM))?
            .get_mut(&chunk_pos)
    }

    pub fn get_compressed_chunk(
        &mut self,
        vec: Vec<IVec3>,
        mtx: Sender<QueuedChunk>,
    ) -> Vec<CompressedChunk> {
        let mut par_queue = Vec::new();

        let (tx, rx) = bounded::<CompressedChunk>(1000);
        let chunk_sender = tx;
        let mesh_sender = mtx;

        for pos in vec.iter() {
            match self.regions.get(pos) {
                Some(c) => match c.get(pos) {
                    Some(chunk) => {
                        mesh_sender
                            .send(QueuedChunk {
                                chunk: *chunk,
                                is_new: true,
                            })
                            .unwrap();
                        chunk_sender.send(chunk.compress()).unwrap()
                    }
                    None => par_queue.push(pos),
                },
                None => match self.load_region(*pos >> 5).get(pos) {
                    Some(chunk) => {
                        mesh_sender
                            .send(QueuedChunk {
                                chunk: *chunk,
                                is_new: true,
                            })
                            .unwrap();
                        chunk_sender.send(chunk.compress()).unwrap()
                    }
                    None => par_queue.push(pos),
                },
            }
        }

        par_queue.par_iter().for_each(|pos| {
            chunk_sender
                .send(chunk_generator(pos, self.seed).compress())
                .unwrap();
        });

        let vector: Vec<CompressedChunk> = rx.try_iter().collect();
        vector
        // self.save_region(*pos >> 5);
    }
}
