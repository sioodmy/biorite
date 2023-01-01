use crate::prelude::*;
use bracket_noise::prelude::*;

#[derive(Resource)]
pub struct NoiseResource(pub FastNoise);
pub fn get_noise() -> NoiseResource {
    let mut noise = FastNoise::seeded(2137);
    noise.set_noise_type(NoiseType::Perlin);
    noise.set_fractal_octaves(3);
    noise.set_fractal_gain(0.06);
    noise.set_fractal_lacunarity(0.25);
    noise.set_frequency(0.07);

    NoiseResource(noise)
}

pub fn chunk_generator(position: IVec3, noise: &FastNoise) -> Chunk {
    // placeholder for propper chunk generation
    let mut blocks: [BlockType; ChunkShape::SIZE as usize] = [AIR; ChunkShape::SIZE as usize];

    // TODO: propper seed handling
    // TODO: async chunk generation

    let offset = 8.0;
    let factor = 7.37;
    let flat: f64 = 5.0;

    /// 16^3 chunk with one block boundary
    for x in 1..CHUNK_DIM + 1 {
        for z in 1..CHUNK_DIM + 1 {
            for y in 1..CHUNK_DIM + 1 {
                // Global cords
                let gx = position.x as f32 * CHUNK_DIM as f32 + x as f32;
                let gy = position.y as f32 * CHUNK_DIM as f32 + y as f32;
                let gz = position.z as f32 * CHUNK_DIM as f32 + z as f32;

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
