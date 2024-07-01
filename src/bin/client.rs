use std::net::{IpAddr, Ipv4Addr};

use bevy::prelude::*;
use bevy_quinnet::{client::{certificate::CertificateVerificationMode, connection::ClientEndpointConfiguration, QuinnetClient, QuinnetClientPlugin}, shared::channels::ChannelsConfiguration};

use net_phys::protocol::ServerMessage;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(QuinnetClientPlugin::default())
        .insert_resource(ClearColor(Color::BLACK))
        .add_systems(Startup, (setup, start_connection))
        .add_systems(Update, (rotate, handle_server_messages))
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Camera
    commands.spawn(Camera2dBundle::default());

    // Sprite
    commands.spawn(SpriteBundle {
        texture: asset_server.load("bevy.png"),
        ..default()
    });
}

fn rotate(mut query: Query<&mut Transform, With<Sprite>>, time: Res<Time>) {
    for mut bevy in &mut query {
        bevy.rotate_local_z(-time.delta_seconds());
    }
}

fn start_connection(mut client: ResMut<QuinnetClient>) {
    let _ = client.open_connection(
        ClientEndpointConfiguration::from_ips(
            IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
            6000,
            IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)),
            0,
        ),
        CertificateVerificationMode::SkipVerification,
        ChannelsConfiguration::default(),
    );
}



fn handle_server_messages(
    mut client: ResMut<QuinnetClient>,
) {
    while let Ok(Some((_, message))) = client.connection_mut().receive_message::<ServerMessage>() {
        match message {
            ServerMessage::InitClient( client_id ) => {
                info!("InitClient");
            },
            ServerMessage::SpawnCube { owner_client_id, entity, position } => {
                info!("SpawnCube");
            },
            ServerMessage::CubeMoved { entity, position } => {
                info!("CubeMoved");
            }
        }
    }
}
