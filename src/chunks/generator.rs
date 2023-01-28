use crate::prelude::*;
use splines::{Interpolation, Key, Spline};

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
    level_noise.set_fractal_gain(1.39);
    level_noise.set_fractal_lacunarity(0.15);
    level_noise.set_frequency(0.039);

    let mut temperature_noise = FastNoise::seeded(seed);
    temperature_noise.set_noise_type(NoiseType::Value);
    temperature_noise.set_fractal_octaves(16);
    temperature_noise.set_fractal_gain(1.9);
    temperature_noise.set_fractal_lacunarity(1.25);
    temperature_noise.set_frequency(0.03);

    let mut hill_noise = FastNoise::seeded(seed);
    hill_noise.set_noise_type(NoiseType::Value);
    hill_noise.set_fractal_octaves(16);
    hill_noise.set_fractal_gain(1.9);
    hill_noise.set_fractal_lacunarity(1.25);
    hill_noise.set_frequency(0.03);

    let mut moisture_noise = FastNoise::seeded(12);
    moisture_noise.set_noise_type(NoiseType::Value);
    moisture_noise.set_fractal_octaves(4);
    moisture_noise.set_fractal_gain(5.5);
    moisture_noise.set_fractal_lacunarity(0.75);
    moisture_noise.set_frequency(0.03);

    let mut cave_noise = FastNoise::seeded(seed);
    cave_noise.set_noise_type(NoiseType::Perlin);
    cave_noise.set_fractal_octaves(16);
    cave_noise.set_fractal_gain(0.45);
    cave_noise.set_fractal_lacunarity(3.75);
    cave_noise.set_frequency(0.07);

    // placeholder for propper chunk generation
    let mut blocks: [BlockType; ChunkShape::SIZE as usize] =
        [BlockType::Air; ChunkShape::SIZE as usize];

    // TODO: propper seed handling
    // TODO: async chunk generation

    let offset = 10.0;
    // let factor = 7.37;
    let factor = 19.0;
    let flat: f64 = 10.0;
    let _rng = rand::thread_rng();

    let start = Key::new(0., 0., Interpolation::Linear);
    let mid = Key::new(3., 10., Interpolation::Linear);
    let end = Key::new(7., 40., Interpolation::default());
    let hill_spline = Spline::from_vec(vec![start, mid, end]);

    // 16^3 chunk with one block boundary
    for x in 1..CHUNK_DIM + 1 {
        for z in 1..CHUNK_DIM + 1 {
            for y in 1..CHUNK_DIM + 1 {
                // Global cords
                let gx = position.x as f32 * CHUNK_DIM as f32 + x as f32;
                let gy = position.y as f32 * CHUNK_DIM as f32 + y as f32;
                let gz = position.z as f32 * CHUNK_DIM as f32 + z as f32;

                let n = cave_noise.get_noise3d(gx, gy, gz) * factor + offset;
                let moisture = (moisture_noise.get_noise(gx, gz) + 1.0) * 2.5;

                debug!("{:?}", n);
                let i = ChunkShape::linearize([x, y, z]);
                // let density = if gy > 34. { 13. } else { 3. };
                // let density = if gy > 20. { 8. + gy / 10. } else { 3. };
                // let density = moisture + 5. + gy / 5.;
                // // let density = gy * 3.8;
                // if n > density {
                //     blocks[i as usize] = BlockType::Stone;
                // } else {
                //     blocks[i as usize] = BlockType::Air;
                // }

                let terrain = level_noise.get_noise(gx, gz) * factor + offset;
                let temp = (temperature_noise.get_noise(gx, gz) + 1.0) * 2.5;
                let hill = hill_noise.get_noise(gx, gz) * 10.;

                if let Some(s) = hill_spline.clamped_sample(hill) {
                    let surface = flat + terrain as f64 + s;
                    if gy < surface as f32 {
                        blocks[i as usize] = BlockType::Stone;
                    }
                    if gy == surface.floor() as f32 {
                        blocks[i as usize] = BlockType::Grass;
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
