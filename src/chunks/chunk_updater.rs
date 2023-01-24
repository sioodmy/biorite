use crate::prelude::*;
use crossbeam_channel::{Receiver, Sender};

#[derive(Resource)]
pub struct ChunkUpdaterSender(pub Sender<Chunk>);

#[derive(Resource)]
pub struct ChunkUpdaterReceiver(pub Receiver<Chunk>);
