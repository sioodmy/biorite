use crate::prelude::*;

pub fn handle_block_updates(msg: Res<CurrentServerMessages>) {
    for (id, message) in msg.iter() {
        if let ClientMessage::BreakBlock(block) = message {
            debug!("got break block packet at {:?} from {}", block, id);
        }
    }
}
