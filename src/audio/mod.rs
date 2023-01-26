use crate::prelude::*;
use rand::Rng;

use bevy::time::FixedTimestep;
use bevy_kira_audio::prelude::{Audio, *};

#[allow(dead_code)]
fn start_background_audio(asset_server: Res<AssetServer>, audio: Res<Audio>) {
    audio
        .play(asset_server.load("audio/music/glacier.mp3"))
        .looped();
}

/// Naive step sound system
fn step_system(
    asset_server: Res<AssetServer>,
    audio: Res<Audio>,
    movement: Query<&Velocity, (With<ControlledPlayer>, Changed<Transform>)>,
) {
    let mut rng = rand::thread_rng();
    for i in movement.iter() {
        if (i.linvel.x.abs() > 1. || i.linvel.z.abs() > 1.)
            && i.linvel.y.round() == 0.
        {
            let step_sound = if rng.gen_range(1..=2) == 1 {
                "audio/sfx/grass4.wav"
            } else {
                "audio/sfx/grass3.wav"
            };
            audio.play(asset_server.load(step_sound));
            break;
        }
    }
}

pub struct SoundsPlugin;
impl Plugin for SoundsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(AudioPlugin).add_system_set(
            SystemSet::new()
                .with_run_criteria(FixedTimestep::step(30. / 60.))
                .with_system(step_system),
        );
        // .add_startup_system(start_background_audio);
    }
}
