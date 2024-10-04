use avian2d::prelude::*;
use bevy::ecs::system::SystemState;
use bevy::ecs::world;
use bevy::prelude::*;
use leafwing_input_manager::prelude::*;
use lightyear::prelude::client::*;
use lightyear::prelude::*;

use crate::protocol::*;
use crate::shared::{shared_movement_behaviour, FixedSet};

pub struct ClientPlugin;

impl Plugin for ClientPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(MockInputSettings::default())
            .add_systems(Startup, init)
            .add_systems(
                PreUpdate,
                handle_connection
                    .after(MainSet::Receive)
                    .before(PredictionSet::SpawnPrediction),
            )
            .add_systems(FixedUpdate, player_movement.in_set(FixedSet::Main))
            .add_systems(
                Update,
                (
                    add_ball_physics,
                    add_player_physics,
                    handle_predicted_spawn,
                    handle_interpolated_spawn,
                    mock_input.run_if(|mis: Res<MockInputSettings>| mis.enabled),
                    mock_input_toggle,
                ),
            );
    }
}

#[derive(Resource)]
pub struct MockInputSettings {
    enabled: bool,
}

impl Default for MockInputSettings {
    fn default() -> Self {
        Self { enabled: true }
    }
}

pub(crate) fn init(mut commands: Commands) {
    commands.connect_client();
}

pub(crate) fn handle_connection(
    mut commands: Commands,
    mut connection_event: EventReader<ConnectEvent>,
) {
    for event in connection_event.read() {
        let client_id = event.client_id();
        commands.spawn(TextBundle::from_section(
            format!("Client {}", client_id),
            TextStyle {
                font_size: 30.0,
                color: Color::WHITE,
                ..default()
            },
        ));
        let y = (client_id.to_bits() as f32 * 50.0) % 500.0 - 250.0;
        commands.spawn(PlayerBundle::new(
            client_id,
            Vec2::new(-50.0, y),
            InputMap::new([(
                PlayerActions::Move,
                VirtualDPad {
                    up: KeyCode::KeyW.into(),
                    down: KeyCode::KeyS.into(),
                    left: KeyCode::KeyA.into(),
                    right: KeyCode::KeyD.into(),
                },
            )]),
        ));
    }
}

fn add_ball_physics(
    mut commands: Commands,
    mut ball_query: Query<
        Entity,
        (
            With<BallMarker>,
            Or<(Added<Interpolated>, Added<Predicted>)>,
        ),
    >,
) {
    for entity in ball_query.iter_mut() {
        commands.entity(entity).insert(PhysicsBundle::ball());
    }
}

fn add_player_physics(
    connection: Res<ClientConnection>,
    mut commands: Commands,
    mut player_query: Query<(Entity, &PlayerId), (Or<(Added<Interpolated>, Added<Predicted>)>,)>,
) {
    let client_id = connection.id();
    for (entity, player_id) in player_query.iter_mut() {
        if player_id.0 == client_id {
            continue;
        }
        commands.entity(entity).insert(PhysicsBundle::player());
    }
}

fn player_movement(
    tick_manager: Res<TickManager>,
    mut velocity_query: Query<
        (
            Entity,
            &PlayerId,
            &Position,
            &mut LinearVelocity,
            &ActionState<PlayerActions>,
        ),
        With<Predicted>,
    >,
) {
    for (entity, player_id, position, velocity, action_state) in velocity_query.iter_mut() {
        if !action_state.get_pressed().is_empty() {
            trace!(?entity, tick = ?tick_manager.tick(), ?position, actions = ?action_state.get_pressed(), "applying movement to predicted player");

            shared_movement_behaviour(velocity, action_state);
        }
    }
}

fn mock_input_toggle() {}

fn mock_input(
    world: &mut World,
    params: &mut SystemState<(
        Res<Time>,
        Res<ButtonInput<KeyCode>>,
        ResMut<MockInputSettings>,
    )>,
) {
    world.release_input(KeyCode::KeyA);
    world.release_input(KeyCode::KeyD);

    let (time, keys, mut mis) = params.get_mut(world);

    if keys.just_pressed(KeyCode::KeyT) {
        mis.enabled = !mis.enabled;
    }

    if (time.elapsed_seconds() * 1.5) as i32 & 1 == 0 {
        world.send_input(KeyCode::KeyA)
    } else {
        world.send_input(KeyCode::KeyD)
    }
}

pub(crate) fn handle_predicted_spawn(mut predicted: Query<&mut ColorComponent, Added<Predicted>>) {
    for mut color in predicted.iter_mut() {
        let hsva = Hsva {
            saturation: 0.4,
            ..Hsva::from(color.0)
        };
        color.0 = Color::from(hsva);
    }
}

pub(crate) fn handle_interpolated_spawn(
    mut interpolated: Query<&mut ColorComponent, Added<Interpolated>>,
) {
    for mut color in interpolated.iter_mut() {
        let hsva = Hsva {
            saturation: 0.1,
            ..Hsva::from(color.0)
        };
        color.0 = Color::from(hsva);
    }
}
