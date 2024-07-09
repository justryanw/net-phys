use std::ops::{Add, Mul};

use bevy::prelude::*;
use client::ComponentSyncMode;
use lightyear::prelude::*;

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub enum Inputs {
    Direction(Vec2),
    Spawn,
    Delete,
    None,
}

#[derive(Component, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct PlayerId(ClientId);

#[derive(Component, Serialize, Deserialize, Clone, Debug, PartialEq, Deref, DerefMut)]
pub struct PlayerPosition(Vec2);

impl Add for PlayerPosition {
    type Output = PlayerPosition;
    #[inline]
    fn add(self, rhs: PlayerPosition) -> PlayerPosition {
        PlayerPosition(self.0.add(rhs.0))
    }
}

impl Mul<f32> for &PlayerPosition {
    type Output = PlayerPosition;
    fn mul(self, rhs: f32) -> Self::Output {
        PlayerPosition(self.0 * rhs)
    }
}

#[derive(Component, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct PlayerColor(pub(crate) Color);

pub struct ProtocolPlugin;

impl Plugin for ProtocolPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(InputPlugin::<Inputs>::default());

        app.register_component::<PlayerId>(ChannelDirection::ServerToClient)
            .add_prediction(ComponentSyncMode::Once)
            .add_interpolation(ComponentSyncMode::Once);

        app.register_component::<PlayerPosition>(ChannelDirection::ServerToClient)
            .add_prediction(ComponentSyncMode::Full)
            .add_interpolation(ComponentSyncMode::Full)
            .add_linear_interpolation_fn();

        app.register_component::<PlayerColor>(ChannelDirection::ServerToClient)
            .add_prediction(ComponentSyncMode::Once)
            .add_interpolation(ComponentSyncMode::Once);
    }
}
