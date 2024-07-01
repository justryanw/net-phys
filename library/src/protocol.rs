use bevy::prelude::*;
use bevy_quinnet::shared::ClientId;
use serde::{Deserialize, Serialize};

// Client

#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PaddleInput {
    #[default]
    None,
    Up,
    Down,
    Left,
    Right,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClientMessage {
    PaddleInput(PaddleInput),
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

