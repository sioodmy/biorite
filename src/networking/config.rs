use crate::prelude::*;

const CHANNEL_CONFIG: Vec<ChannelConfig> = vec![
    ChannelConfig::Chunk(BlockChannelConfig {
        packet_budget: 30000,
        message_send_queue_size: 64,
        ..Default::default()
    }),
    ChannelConfig::Reliable(ReliableChannelConfig::default()),
    ChannelConfig::Unreliable(UnreliableChannelConfig::default()),
];

pub const RENET_CONNECTION_CONFIG: RenetConnectionConfig = 