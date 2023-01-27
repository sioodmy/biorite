use crate::prelude::*;
use bevy::tasks::AsyncComputeTaskPool;
use scc::HashSet;

pub use block_mesh::ndshape::{ConstShape, ConstShape3i32};
use bracket_noise::prelude::*;

use std::{fs, path::Path};

// TODO: Divide chunks into regions

/// Size of chunk region defined as power of 2
/// by default: 2^5 = 32
pub const REGION_DIM: u32 = 5;

pub type Region = ConstShape3i32<
    { 2_i32.pow(REGION_DIM) },
    { 2_i32.pow(REGION_DIM) },
    { 2_i32.pow(REGION_DIM) },
>;

/// Modified regions
#[derive(Resource, Default)]
pub struct ModifiedRegions(pub HashSet<IVec3>);

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

impl SaveFile {
    pub fn default() -> Self {
        SaveFile {
            name: "New world".into(),
            regions: HashMap::new(),
            seed: 2137,
            dirty: HashSet::new(),
        }
    }

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
}

pub fn chunk_saver(
    mut save: ResMut<SaveFile>,
    mut modified: ResMut<ModifiedRegions>,
) {
    modified.0.for_each(|region| {
        info!("saving region {:?}", region);
        // save.save_region(*region);
        // modified.0.remove(region);
    });
}
pub fn chunk_generator(position: &IVec3, seed: u64) -> Chunk {
    // TODO: world regions
    // TODO: async

    trace!("Generating new chunk");
    // TODO: rng seed

    let mut level_noise = FastNoise::seeded(seed);
    level_noise.set_noise_type(NoiseType::Perlin);
    level_noise.set_fractal_octaves(2);
    level_noise.set_fractal_gain(0.39);
    level_noise.set_fractal_lacunarity(0.15);
    level_noise.set_frequency(0.039);

    let mut temperature_noise = FastNoise::seeded(seed);
    temperature_noise.set_noise_type(NoiseType::Value);
    temperature_noise.set_fractal_octaves(16);
    temperature_noise.set_fractal_gain(1.9);
    temperature_noise.set_fractal_lacunarity(1.25);
    temperature_noise.set_frequency(0.03);

    let mut moisture_noise = FastNoise::seeded(12);
    moisture_noise.set_noise_type(NoiseType::Value);
    moisture_noise.set_fractal_octaves(4);
    moisture_noise.set_fractal_gain(5.5);
    moisture_noise.set_fractal_lacunarity(0.75);
    moisture_noise.set_frequency(0.03);

    let mut forest_noise = FastNoise::seeded(seed);
    forest_noise.set_noise_type(NoiseType::WhiteNoise);
    forest_noise.set_fractal_octaves(8);
    forest_noise.set_fractal_gain(5.5);
    forest_noise.set_fractal_lacunarity(4.75);
    forest_noise.set_frequency(0.1);

    // placeholder for propper chunk generation
    let mut blocks: [BlockType; ChunkShape::SIZE as usize] =
        [BlockType::Air; ChunkShape::SIZE as usize];

    // TODO: propper seed handling
    // TODO: async chunk generation

    let offset = 10.0;
    // let factor = 7.37;
    let factor = 19.0;
    let flat: f64 = 5.0;
    let _rng = rand::thread_rng();

    // 16^3 chunk with one block boundary
    for x in 1..CHUNK_DIM + 1 {
        for z in 1..CHUNK_DIM + 1 {
            for y in 1..CHUNK_DIM + 1 {
                // Global cords
                let gx = position.x as f32 * CHUNK_DIM as f32 + x as f32;
                let gy = position.y as f32 * CHUNK_DIM as f32 + y as f32;
                let gz = position.z as f32 * CHUNK_DIM as f32 + z as f32;

                let n = level_noise.get_noise(gx, gz) * factor + offset;

                let surface = flat + n as f64;
                let i = ChunkShape::linearize([x, y, z]);
                if gy as f64 > surface {
                    blocks[i as usize] = BlockType::Air;
                } else if gy < surface as f32 {
                    blocks[i as usize] = BlockType::Stone;
                }

                if gy == surface.floor() as f32 {
                    level_noise.set_fractal_octaves(1);
                    level_noise.set_fractal_lacunarity(1.0);

                    let temp =
                        (temperature_noise.get_noise(gx, gz) + 1.0) * 2.5;
                    let moisture =
                        (moisture_noise.get_noise(gx, gz) + 1.0) * 2.5;

                    let forest = (forest_noise.get_noise(gx, gz) + 1.0) / 2.0;

                    debug!(
                        "moisture {:?}, temp: {:?}, forest: {:?}",
                        moisture, temp, forest
                    );

                    if temp > 2.0 && moisture < 2.0 {
                        // desert
                        blocks[i as usize] = BlockType::Sand;
                    } else {
                        blocks[i as usize] = BlockType::Grass;
                    }
                    if forest > 0.1 && forest < 0.16 {
                        blocks[i as usize] = BlockType::Wood;
                    }
                }
            }
        }
    }
    let new_chunk = Chunk {
        position: *position,
        blocks,
        ..Default::default()
    };

    new_chunk
}
