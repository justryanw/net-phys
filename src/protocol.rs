
use avian2d::prelude::*;
use bevy::prelude::*;
use leafwing_input_manager::InputManagerBundle;
use lightyear::{
    prelude::*,
    client::{
        components::{ComponentSyncMode, LerpFn},
        interpolation::LinearInterpolator
    },
    utils::avian2d::*,
};
use leafwing_input_manager::prelude::*;

pub const BALL_SIZE: f32 = 15.0;
pub const PLAYER_SIZE: f32 = 40.0;
pub const REPLICATION_GROUP: ReplicationGroup = ReplicationGroup::new_id(1);

#[derive(Bundle)]
pub(crate) struct PlayerBundle {
    id: PlayerId,
    position: Position,
    color: ColorComponent,
    replicate: client::Replicate,
    physics: PhysicsBundle,
    inputs: InputManagerBundle<PlayerActions>,
    pre_predicted: PrePredicted,
}

impl PlayerBundle {
    pub(crate) fn new(id: ClientId, position: Vec2, input_map: InputMap<PlayerActions>) -> Self {
        let color = color_from_id(id);
        Self {
            id: PlayerId(id),
            position: Position(position),
            color: ColorComponent(color),
            replicate: client::Replicate {
                group: REPLICATION_GROUP,
                ..default()
            },
            physics: PhysicsBundle::player(),
            inputs: InputManagerBundle::<PlayerActions> {
                action_state: ActionState::default(),
                input_map,
            },
            pre_predicted: PrePredicted::default(),
        }
    }
}

#[derive(Bundle)]
pub(crate) struct BallBundle {
    position: Position,
    color: ColorComponent,
    replicate: server::Replicate,
    marker: BallMarker,
    physics: PhysicsBundle
}

impl BallBundle {
    pub(crate) fn new(position: Vec2, color: Color, predicted: bool) -> Self {
        let mut sync_target = server::SyncTarget::default();
        let mut group = ReplicationGroup::default();
        if predicted {
            sync_target.prediction = NetworkTarget::All;
            group = REPLICATION_GROUP;
        } else {
            sync_target.interpolation = NetworkTarget::All;
        }
        let replicate = server::Replicate {
            sync: sync_target,
            group,
            ..default()
        };
        Self {
            position: Position(position),
            color: ColorComponent(color),
            replicate,
            physics: PhysicsBundle::ball(),
            marker: BallMarker,
        }
    }
}

#[derive(Bundle)]
pub(crate) struct PhysicsBundle {
    pub(crate) collider: Collider,
    pub(crate) collider_density: ColliderDensity,
    pub(crate) rigid_body: RigidBody,
}

impl PhysicsBundle {
    pub(crate) fn ball() -> Self {
        Self {
            collider: Collider::circle(BALL_SIZE),
            collider_density: ColliderDensity(0.05),
            rigid_body: RigidBody::Dynamic,
        }
    }

    pub(crate) fn player() -> Self {
        Self {
            collider: Collider::rectangle(PLAYER_SIZE, PLAYER_SIZE),
            collider_density: ColliderDensity(0.2),
            rigid_body: RigidBody::Dynamic,
        }
    }
}

#[derive(Component, Serialize, Deserialize, Clone, Debug, PartialEq, Reflect)]
pub struct PlayerId(pub ClientId);
 
#[derive(Component, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ColorComponent(pub(crate) Color);

#[derive(Component, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct BallMarker;

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Copy, Hash, Reflect, Actionlike)]
pub enum PlayerActions {
    Up,
    Down,
    Left,
    Right
}

pub struct ProtocolPlugin;

impl Plugin for ProtocolPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(LeafwingInputPlugin::<PlayerActions>::default());

        app.register_component::<PlayerId>(ChannelDirection::Bidirectional)
            .add_prediction(ComponentSyncMode::Once)
            .add_interpolation(ComponentSyncMode::Once);

        app.register_component::<ColorComponent>(ChannelDirection::Bidirectional)
            .add_prediction(ComponentSyncMode::Once)
            .add_interpolation(ComponentSyncMode::Once);

        app.register_component::<BallMarker>(ChannelDirection::Bidirectional)
            .add_prediction(ComponentSyncMode::Once)
            .add_interpolation(ComponentSyncMode::Once);

        app.register_component::<Position>(ChannelDirection::Bidirectional)
            .add_prediction(ComponentSyncMode::Full)
            .add_interpolation(ComponentSyncMode::Full)
            .add_interpolation_fn(position::lerp)
            .add_correction_fn(position::lerp);

        app.register_component::<Rotation>(ChannelDirection::Bidirectional)
            .add_prediction(ComponentSyncMode::Full)
            .add_interpolation(ComponentSyncMode::Full)
            .add_interpolation_fn(rotation::lerp)
            .add_correction_fn(rotation::lerp);

        app.register_component::<LinearVelocity>(ChannelDirection::Bidirectional)
            .add_prediction(ComponentSyncMode::Full);

        app.register_component::<AngularVelocity>(ChannelDirection::Bidirectional)
            .add_prediction(ComponentSyncMode::Full);
    }
}
