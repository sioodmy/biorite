pub use crate::prelude::*;
pub use bevy::prelude::*;
pub use bevy_renet::{renet::*, *};
use lz4::block::{compress, CompressionMode};

mod chunks;
mod debugging;
mod networking;
pub mod prelude;
mod render;

pub const PROTOCOL_ID: u64 = 1000;
