use crate::render::*;
use crate::*;
use block_mesh::ndshape::{ConstShape, ConstShape3u32};

pub fn spawn_chunk(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    // shitass chunk generation
    let mut voxels: [BlockType; ChunkShape::SIZE as usize] = [AIR; ChunkShape::SIZE as usize];

    for i in 0..ChunkShape::SIZE {
        let [x, y, z] = ChunkShape::delinearize(i);
        voxels[i as usize] = if ((x * x + y * y + z * z) as f32).sqrt() < 15.0 {
            DIRT
        } else {
            AIR
        };
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
        transform: Transform::from_xyz(0.0, 0.5, 0.0),
        ..default()
    });
}
