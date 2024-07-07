use std::{
    collections::HashMap,
    env,
    net::{IpAddr, Ipv4Addr},
    path::PathBuf,
};

use bevy::{app::ScheduleRunnerPlugin, log::LogPlugin, prelude::*};
use bevy_quinnet::{
    server::{
        certificate::CertificateRetrievalMode, ConnectionEvent, ConnectionLostEvent, Endpoint,
        QuinnetServer, QuinnetServerPlugin, ServerEndpointConfiguration,
    },
    shared::{channels::ChannelsConfiguration, ClientId},
};

use lib::protocol::{ClientMessage, ServerChannel, ServerMessage};

#[derive(Debug, Clone, Default)]
struct Player {
    input: Vec2,
}

#[derive(Resource, Debug, Clone, Default)]
struct Players {
    map: HashMap<ClientId, Player>,
}

#[derive(Component)]
struct Cube {
    player_id: ClientId,
}

fn main() {
    App::new()
        .add_plugins((
            ScheduleRunnerPlugin::default(),
            LogPlugin::default(),
            QuinnetServerPlugin::default(),
        ))
        .insert_resource(Players::default())
        .add_systems(Startup, start_listening)
        .add_systems(Update, (handle_server_events, handle_client_messages, update))
        .run();
}

fn start_listening(mut server: ResMut<QuinnetServer>) {
    server
        .start_endpoint(
            ServerEndpointConfiguration::from_ip(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 6000),
            CertificateRetrievalMode::GenerateSelfSigned {
                server_hostname: "127.0.0.1".to_string(),
            },
            ChannelsConfiguration::default(),
        )
        .unwrap();
}

fn handle_server_events(
    mut commands: Commands,
    mut connection_events: EventReader<ConnectionEvent>,
    mut connection_lost_events: EventReader<ConnectionLostEvent>,
    mut players: ResMut<Players>,
    mut server: ResMut<QuinnetServer>,
) {
    for client in connection_events.read() {
        players.map.insert(client.id, Player { input: Vec2::ZERO });

        commands.spawn((
            Cube {
                player_id: client.id,
            },
            TransformBundle::default(),
        ));
    }

    for client in connection_lost_events.read() {
        handle_disconnect(server.endpoint_mut(), &mut players, client.id);
    }
}

fn handle_disconnect(endpoint: &mut Endpoint, players: &mut ResMut<Players>, client_id: ClientId) {
    players.map.remove(&client_id);

    let _ = endpoint.disconnect_client(client_id);

    // TODO remove player on disconnect
    // let Some((entity, _)) = cubes.iter_mut().find(|(_, cube)| cube.player_id == client.id) else { return; };
    // commands.entity(entity).despawn();
}

fn handle_client_messages(mut server: ResMut<QuinnetServer>, mut players: ResMut<Players>) {
    let endpoint = server.endpoint_mut();
    for client_id in endpoint.clients() {
        while let Some((_, message)) = endpoint.try_receive_message_from::<ClientMessage>(client_id)
        {
            match message {
                ClientMessage::PlayerInput(input) => {
                    if let Some(player) = players.map.get_mut(&client_id) {
                        player.input = input;
                    }
                }
            }
        }
    }
}

fn update(
    mut cubes: Query<(&mut Transform, &Cube, Entity)>,
    players: ResMut<Players>,
    server: Res<QuinnetServer>,
) {
    for (mut transform, cube, entity) in cubes.iter_mut() {
        let Some(player) = players.map.get(&cube.player_id) else {
            continue;
        };

        let Vec2 { x, y } = player.input.normalize_or_zero();

        // if player.input != Vec2::ZERO {
            transform.translation.x += x * 0.1;
            transform.translation.y += y * 0.1;
        // }

        server.endpoint().try_send_group_message_on(
            players.map.keys().into_iter(),
            ServerChannel::CubeUpdates,
            ServerMessage::CubeMoved {
                entity,
                position: transform.translation.xy(),
            },
        );
    }
}
