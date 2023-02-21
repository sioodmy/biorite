use bevy::prelude::Resource;
use clap::Parser;

#[derive(Parser, Debug, Resource)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[arg(short, long, default_value = "0.0.0.0:42069")]
    pub ip: String,
    #[arg(short, long, default_value = "0.0.0.0:8080")]
    pub auth_ip: String,
}
