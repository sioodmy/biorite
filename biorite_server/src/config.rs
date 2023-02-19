use bevy::prelude::Resource;
use clap::Parser;

#[derive(Parser, Debug, Resource)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[arg(short, long)]
    pub ip: Option<String>,
}
