use bevy::render::render_resource::*;
use bevy::render::settings::{Backends, WgpuSettings};
use bevy::window::PresentMode;
use biorite::*;

fn main() {
    App::new()
        .insert_resource(WgpuSettings {
            backends: Some(Backends::VULKAN),
            ..Default::default()
        })
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    window: WindowDescriptor {
                        width: 1280.,
                        height: 720.,
                        title: format!("Biorite {}", env!("CARGO_PKG_VERSION")),
                        resizable: true,
                        present_mode: PresentMode::AutoVsync,
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .set(ImagePlugin {
                    default_sampler: SamplerDescriptor {
                        address_mode_u: AddressMode::Repeat,
                        address_mode_v: AddressMode::Repeat,
                        address_mode_w: AddressMode::Repeat,
                        mag_filter: FilterMode::Nearest,
                        min_filter: FilterMode::Nearest,
                        ..Default::default()
                    },
                }),
        )
        .add_plugin(RenetClientPlugin::default())
        .add_plugin(RenderClientPlugin)
        .add_plugin(NetworkClientPlugin)
        .add_plugin(UiPlugin)
        .add_plugin(DebugPlugin)
        .add_state(AppState::MainMenu)
        .run();
}
