use bevy::prelude::*;
use bevy_quinnet::shared::{channels::{ChannelId, ChannelType, ChannelsConfiguration}, ClientId};
use serde::{Deserialize, Serialize};

// Client

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClientMessage {
    PlayerInput(Vec2),
}

// Server

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServerMessage {
    InitClient(ClientId),
    SpawnCube {
      owner_client_id: ClientId,
      entity: Entity,
      position: Vec2,
    },
    CubeMoved {
      entity: Entity,
      position: Vec2,
    }
}

#[repr(u8)]
pub enum ServerChannel {
    CubeUpdates,
}

impl Into<ChannelId> for ServerChannel {
    fn into(self) -> ChannelId {
        self as ChannelId
    }
}

impl ServerChannel {
    pub fn channels_configuration() -> ChannelsConfiguration {
        ChannelsConfiguration::from_types(vec![
            ChannelType::Unreliable,
        ])
        .unwrap()
    }
}
