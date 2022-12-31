use bazerite::*;

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    window: WindowDescriptor {
                        width: 1280.,
                        height: 720.,
                        title: format!("Bazerite {}", env!("CARGO_PKG_VERSION")).to_string(),
                        resizable: true,
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .set(ImagePlugin::default_nearest()),
        )
        .add_plugin(RenetClientPlugin::default())
        .add_plugin(RenderClientPlugin)
        .add_plugin(NetworkClientPlugin)
        .add_plugin(DebugPlugin)
        .run();
}
