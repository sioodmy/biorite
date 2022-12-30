use bevy::pbr::wireframe::WireframeConfig;
use block_mesh::ndshape::ConstShape;
use rand::Rng;

use crate::{render::*, ChunkShape, AIR, DIRT, STONE, *};

pub fn wireframe(mut wireframe_config: ResMut<WireframeConfig>, keyboard: Res<Input<KeyCode>>) {
    if keyboard.just_pressed(KeyCode::F3) {
        info!("Toggle debug mode");

        wireframe_config.global = !wireframe_config.global;
    }
}

// TODO: Refactor code to pull chunks from render queue
pub fn spawn_chunk(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    // shitass chunk generation
    let mut voxels: [BlockType; ChunkShape::SIZE as usize] = [AIR; ChunkShape::SIZE as usize];

    // for i in 0..ChunkShape::SIZE {
    //     let [x, y, z] = ChunkShape::delinearize(i);
    //     voxels[i as usize] = if ((x * x + y * y + z
    // * z) as f32).sqrt() < 15.0 {         DIRT } else { AIR };
    // }

    // placeholder for propper chunk generation
    for x in 1..17 {
        for z in 1..17 {
            for y in 1..12 {
                let i = ChunkShape::linearize([x, y, z]);
                voxels[i as usize] = STONE;
            }
            for y in 12..15 {
                let i = ChunkShape::linearize([x, y, z]);
                voxels[i as usize] = DIRT;
            }
        }
    }

    // mesh generation
    let greedy_mesh = greedy_mesh(&mut meshes, voxels);

    // texture handling stuff
    let texture_handle = asset_server.load("textures/stone.png");

    // this material renders the texture normally
    let material = materials.add(StandardMaterial {
        base_color_texture: Some(texture_handle.clone()),
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        ..default()
    });

    // rendering chunk
    commands.spawn(PbrBundle {
        mesh: greedy_mesh,
        material,
        transform: Transform::from_xyz(0.0, 2.0, 0.0),
        ..default()
    });
}
