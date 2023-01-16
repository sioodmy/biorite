use crate::prelude::*;
use bracket_noise::prelude::*;

use std::{fs, path::Path};

pub fn chunk_generator(position: &IVec3) -> Chunk {
    // TODO: world regions
    // TODO: async

    // Chunk Unit Mechanism format (cum)

    let filename = format!(
        "world/chunks/{}_{}_{}.cum",
        position.x, position.y, position.z
    );
    let filename = Path::new(&filename);

    if filename.exists() {
        debug!("Chunk exists, passing from save file");
        let chunk_bytes = fs::read(filename).expect("Couldnt read chunk data");
        Chunk::from_compressed(&chunk_bytes)
    } else {
        trace!("Generating new chunk");
        // TODO: rng seed
        let seed: u64 = 2137;

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

        // placeholder for propper chunk generation
        let mut blocks: [BlockType; ChunkShape::SIZE as usize] =
            [AIR; ChunkShape::SIZE as usize];

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

                    let n = level_noise.get_noise(gx as f32, gz as f32)
                        * factor
                        + offset;

                    let surface = flat as f64 + n as f64;
                    let i = ChunkShape::linearize([x, y, z]);
                    if gy as f64 > surface {
                        blocks[i as usize] = AIR;
                    } else if gy < surface as f32 {
                        blocks[i as usize] = STONE;
                    }

                    if gy == surface.floor() as f32 {
                        level_noise.set_fractal_octaves(1);
                        level_noise.set_fractal_lacunarity(1.0);

                        let temp = (temperature_noise
                            .get_noise(gx as f32, gz as f32)
                            + 1.0)
                            * 2.5;
                        let moisture = (moisture_noise
                            .get_noise(gx as f32, gz as f32)
                            + 1.0)
                            * 2.5;

                        debug!("moisture {:?}, temp: {:?}", moisture, temp);

                        if temp > 2.0 && moisture < 2.0 {
                            // desert
                            blocks[i as usize] = SAND;
                        } else if temp > 1.0 {
                            blocks[i as usize] = GRASS;
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
        trace!("Saving chunk");
        fs::write(filename, new_chunk.compress())
            .expect("Couldnt write chunk save");
        new_chunk
    }
}
