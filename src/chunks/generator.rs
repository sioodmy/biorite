use crate::prelude::*;
use splines::{Interpolation, Key, Spline};

use bevy::tasks::AsyncComputeTaskPool;
use rand::Rng;

use scc::HashSet;

pub use block_mesh::ndshape::{ConstShape, ConstShape2u32, ConstShape3i32};
use bracket_noise::prelude::*;

use std::{fs, path::Path};

// See: https://www.youtube.com/watch?v=CSa5O6knuwI&t=1194s

/// Size of chunk region defined as power of 2
/// by default: 2^5 = 32
pub const REGION_DIM: u32 = 5;

type HeightMapShape = ConstShape2u32<{ CHUNK_DIM + 2 }, { CHUNK_DIM + 2 }>;
type HeightMap = [u32; HeightMapShape::USIZE];

pub type Blocks = [BlockType; ChunkShape::USIZE];
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

fn get_global_coords(
    position: &IVec3,
    x: u32,
    y: u32,
    z: u32,
) -> (f32, f32, f32) {
    (
        position.x as f32 * CHUNK_DIM as f32 + x as f32,
        position.y as f32 * CHUNK_DIM as f32 + y as f32,
        position.z as f32 * CHUNK_DIM as f32 + z as f32,
    )
}

fn get_height(x: f32, z: f32, seed: u64) -> Option<u32> {
    let mut level_noise = FastNoise::seeded(seed);
    level_noise.set_noise_type(NoiseType::Perlin);
    level_noise.set_fractal_octaves(2);
    level_noise.set_fractal_gain(1.39);
    level_noise.set_fractal_lacunarity(0.15);
    level_noise.set_frequency(0.039);

    let mut hill_noise = FastNoise::seeded(seed);
    hill_noise.set_noise_type(NoiseType::Value);
    hill_noise.set_fractal_octaves(16);
    hill_noise.set_fractal_gain(1.9);
    hill_noise.set_fractal_lacunarity(0.45);
    hill_noise.set_frequency(0.0115);

    let mut erosion_noise = FastNoise::seeded(seed + 3);
    erosion_noise.set_noise_type(NoiseType::Simplex);
    erosion_noise.set_fractal_octaves(8);
    erosion_noise.set_fractal_gain(1.3);
    erosion_noise.set_fractal_lacunarity(1.75);
    erosion_noise.set_frequency(0.03);

    let offset = 10.0;
    // let factor = 7.37;
    let factor = 9.0;
    let flat: f64 = 10.0;
    let mut rng = rand::thread_rng();

    let hill_spline = Spline::from_vec(vec![
        Key::new(-10., 0., Interpolation::Linear),
        Key::new(-6., 15., Interpolation::Linear),
        Key::new(0., 0., Interpolation::Linear),
        Key::new(3., 10., Interpolation::Linear),
        Key::new(6., 28., Interpolation::Linear),
        Key::new(7., 40., Interpolation::Linear),
        Key::new(8., 10., Interpolation::Linear),
        Key::new(9., 40., Interpolation::Linear),
    ]);
    let terrain = level_noise.get_noise(x, z) * factor + offset;
    let hill = hill_noise.get_noise(x, z) * 10.;

    let s = hill_spline.clamped_sample(hill)?;
    Some((flat + terrain as f64 + s).floor() as u32)
}

fn heightmap(position: &IVec3, seed: u64) -> HeightMap {
    let mut heightmap: HeightMap = [1; HeightMapShape::USIZE];

    for x in 1..=CHUNK_DIM {
        for z in 1..=CHUNK_DIM {
            let gx = position.x as f32 * CHUNK_DIM as f32 + x as f32;
            let gz = position.z as f32 * CHUNK_DIM as f32 + z as f32;
            heightmap[HeightMapShape::linearize([x, z]) as usize] =
                if let Some(height) = get_height(gx, gz, seed) {
                    height
                } else {
                    warn!("Couldn't generate heightmap for {}/{}. Inserting 0 instead", gx, gz);
                    0
                };
        }
    }

    heightmap
}

fn base_terrain(heightmap: HeightMap, position: &IVec3) -> Blocks {
    let mut blocks: Blocks = [BlockType::Air; ChunkShape::SIZE as usize];
    for x in 1..=CHUNK_DIM {
        for z in 1..=CHUNK_DIM {
            for y in 1..=CHUNK_DIM {
                let gy = position.y as f32 * CHUNK_DIM as f32 + y as f32;

                let h = heightmap[HeightMapShape::linearize([x, z]) as usize];
                let i = ChunkShape::linearize([x, y, z]);
                if h < gy as u32 {
                    continue;
                }

                if h == gy as u32 {
                    blocks[i as usize] = BlockType::Grass;
                } else if h - 3 <= gy as u32 {
                    blocks[i as usize] = BlockType::Dirt;
                } else {
                    blocks[i as usize] = BlockType::Stone;
                }
            }
        }
    }

    blocks
}

fn carve_caves(mut blocks: Blocks, position: &IVec3, seed: u64) -> Blocks {
    let mut cave_noise = FastNoise::seeded(seed);
    cave_noise.set_noise_type(NoiseType::Simplex);
    cave_noise.set_fractal_octaves(16);
    cave_noise.set_fractal_gain(1.9);
    cave_noise.set_fractal_lacunarity(0.4);
    cave_noise.set_frequency(0.01);

    let mut spaghetti_noise = FastNoise::seeded(seed);
    spaghetti_noise.set_noise_type(NoiseType::Simplex);
    spaghetti_noise.set_fractal_octaves(5);
    spaghetti_noise.set_fractal_gain(1.3);
    spaghetti_noise.set_fractal_lacunarity(50.7);
    spaghetti_noise.set_frequency(0.021);

    let mut second_noise = FastNoise::seeded(seed + 69);
    second_noise.set_noise_type(NoiseType::Simplex);
    second_noise.set_fractal_octaves(5);
    second_noise.set_fractal_gain(1.3);
    second_noise.set_fractal_lacunarity(50.7);
    second_noise.set_frequency(0.021);
    for x in 1..CHUNK_DIM + 1 {
        for z in 1..CHUNK_DIM + 1 {
            for y in 1..CHUNK_DIM + 1 {
                let gx = position.x as f32 * CHUNK_DIM as f32 + x as f32;
                let gy = position.y as f32 * CHUNK_DIM as f32 + y as f32;
                let gz = position.z as f32 * CHUNK_DIM as f32 + z as f32;

                let cave = cave_noise.get_noise3d(gx, gy, gz);
                let spaghetti = spaghetti_noise.get_noise3d(gx, gy, gz) * 10.;
                let second = second_noise.get_noise3d(gx, gy, gz) * 10.;

                // debug!("{:?}", (spaghetti + cave).abs());

                let i = ChunkShape::linearize([x, y, z]);
                if cave > 0.6 {
                    if (spaghetti.powf(2.) + second.powf(2.)) < 5.5 {
                        blocks[i as usize] = BlockType::Air;
                    }
                }
                // if spaghetti.abs() + second.abs() < 1. {
                //     blocks[i as usize] = BlockType::Air;
                // }
            }
        }
    }
    blocks
}

fn terrain_features(
    mut blocks: Blocks,
    position: &IVec3,
    heightmap: HeightMap,
    seed: u64,
) -> Blocks {
    let mut forest_noise = FastNoise::seeded(seed);
    forest_noise.set_noise_type(NoiseType::Simplex);
    forest_noise.set_fractal_octaves(2);
    forest_noise.set_fractal_gain(1.9);
    forest_noise.set_fractal_lacunarity(1.25);
    forest_noise.set_frequency(0.03);

    let mut intersection_noise = FastNoise::seeded(seed + 50);
    intersection_noise.set_noise_type(NoiseType::WhiteNoise);
    intersection_noise.set_fractal_octaves(2);
    intersection_noise.set_fractal_gain(1.9);
    intersection_noise.set_fractal_lacunarity(1.25);
    intersection_noise.set_frequency(0.09);

    for x in 1..CHUNK_DIM + 1 {
        for z in 1..CHUNK_DIM + 1 {
            let gx = position.x as f32 * CHUNK_DIM as f32 + x as f32;
            let gz = position.z as f32 * CHUNK_DIM as f32 + z as f32;

            let forest = forest_noise.get_noise(gx, gz);
            let intersection = intersection_noise.get_noise(gx, gz);

            if forest.powf(2.) + intersection.powf(2.) < 0.01 {
                // info!("forest noise {:?} {:?}", forest, intersection);
                let surface =
                    heightmap[HeightMapShape::linearize([x, z]) as usize];
                // let gy = position.y as f32 * CHUNK_DIM as f32 +  as f32;

                let surface_chunk: bool =
                    (surface / CHUNK_DIM) as i32 == position.y;

                if surface < CHUNK_DIM - 7
                    && x <= CHUNK_DIM - 5
                    && z <= CHUNK_DIM - 5
                    && x >= 3
                    && z >= 3
                    && surface != 1
                    && surface_chunk
                {
                    for i in 1..4 {
                        let i = ChunkShape::linearize([x, surface + i, z]);
                        blocks[i as usize] = BlockType::Wood;
                    }

                    // TODO: make some actual trees
                    for l in 0..4 {
                        for j in -2..=2i32 {
                            for k in -2..=2i32 {
                                let i = ChunkShape::linearize([
                                    (x as i32 + j).try_into().unwrap(),
                                    surface + 4 + l,
                                    (z as i32 + k).try_into().unwrap(),
                                ]);
                                blocks[i as usize] = BlockType::Leaves;
                            }
                        }
                    }
                }
            }
        }
    }

    blocks
}

// I know this is poorly written
// its just a prototype
// im just playing around with it (we good?)
// the idea is to split chunk generation into stages
pub fn chunk_generator(position: &IVec3, seed: u64) -> Chunk {
    // TODO: world regions
    // TODO: async

    trace!("Generating new chunk");
    // TODO: rng seed

    let heightmap = heightmap(position, seed);
    let mut blocks = base_terrain(heightmap, position);
    blocks = carve_caves(blocks, position, seed);
    blocks = terrain_features(blocks, position, heightmap, seed);
    let new_chunk = Chunk {
        position: *position,
        blocks,
        ..Default::default()
    };

    new_chunk
}
