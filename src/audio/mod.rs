use crate::prelude::*;
use bevy_kira_audio::prelude::Audio;
use bevy_kira_audio::prelude::*;

fn start_background_audio(asset_server: Res<AssetServer>, audio: Res<Audio>) {
    audio
        .play(asset_server.load("audio/music/glacier.mp3"))
        .looped();
}

pub struct SoundsPlugin;
impl Plugin for SoundsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(AudioPlugin)
            .add_startup_system(start_background_audio);
    }
}
