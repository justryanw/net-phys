use avian2d::prelude::*;
use bevy::color::palettes::css;
use bevy::prelude::*;
use leafwing_input_manager::prelude::*;
use lightyear::prelude::client::{Confirmed, Predicted};
use lightyear::prelude::server::*;
use lightyear::prelude::*;

use crate::protocol::*;
use crate::shared::{shared_movement_behaviour, FixedSet};

pub struct ServerPlugin {
    pub(crate) predict_all: bool,
}

#[derive(Resource)]
pub struct Global {
    predict_all: bool,
}

impl Plugin for ServerPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Global {
            predict_all: self.predict_all,
        });

        app.add_systems(Startup, (start_server, init));
        app.add_systems(PreUpdate, replicate_inputs.after(MainSet::EmitEvents));
        app.add_systems(
            PreUpdate,
            replicate_players.in_set(ServerReplicationSet::ClientReplication),
        );
        app.add_systems(FixedUpdate, movement.in_set(FixedSet::Main));
    }
}

fn start_server(mut commands: Commands) {
    commands.start_server();
}

fn init(mut commands: Commands, global: Res<Global>) {
    commands.spawn(
        TextBundle::from_section(
            "Server",
            TextStyle {
                font_size: 30.0,
                color: Color::WHITE,
                ..default()
            },
        )
        .with_style(Style {
            align_self: AlignSelf::End,
            ..default()
        }),
    );

    let spacing = 40;
    for y in -5..5 {
        for x in -8..8 {
            commands.spawn(BallBundle::new(
                Vec2::new((x * spacing + spacing / 2) as f32, (y * spacing + spacing * 4) as f32),
                css::AZURE.into(),
                global.predict_all,
            ));
        }
    }
}

pub(crate) fn movement(
    mut action_query: Query<
        (
            Entity,
            &Position,
            &mut LinearVelocity,
            &ActionState<PlayerActions>,
        ),
        (Without<Confirmed>, Without<Predicted>),
    >,
) {
    for (_, _, velocity, action) in action_query.iter_mut() {
        if !action.get_pressed().is_empty() {
            shared_movement_behaviour(velocity, action);
        }
    }
}

pub(crate) fn replicate_inputs(
    mut connection: ResMut<ConnectionManager>,
    mut input_events: EventReader<MessageEvent<InputMessage<PlayerActions>>>,
) {
    for event in input_events.read() {
        let inputs = event.message();
        let client_id = event.context();

        connection
            .send_message_to_target::<InputChannel, _>(
                inputs,
                NetworkTarget::AllExceptSingle(*client_id),
            )
            .unwrap()
    }
}

pub(crate) fn replicate_players(
    global: Res<Global>,
    mut commands: Commands,
    query: Query<(Entity, &Replicated), (Added<Replicated>, With<PlayerId>)>,
) {
    for (entity, replicated) in query.iter() {
        let client_id = replicated.client_id();

        if let Some(mut e) = commands.get_entity(entity) {
            let mut sync_target = SyncTarget::default();

            if global.predict_all {
                sync_target.prediction = NetworkTarget::All;
            } else {
                sync_target.interpolation = NetworkTarget::AllExceptSingle(client_id);
            }
            let replicate = Replicate {
                sync: sync_target,
                controlled_by: ControlledBy {
                    target: NetworkTarget::Single(client_id),
                    ..default()
                },
                group: REPLICATION_GROUP,
                ..default()
            };
            e.insert((
                replicate,
                OverrideTargetComponent::<PrePredicted>::new(NetworkTarget::Single(client_id)),
                PhysicsBundle::player(),
            ));
        }
    }
}
