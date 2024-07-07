#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{
    env,
    net::{IpAddr, Ipv4Addr},
    path::PathBuf,
};

use bevy::{input::keyboard, prelude::*};
use bevy_quinnet::{
    client::{
        certificate::CertificateVerificationMode, connection::ClientEndpointConfiguration,
        QuinnetClient, QuinnetClientPlugin,
    },
    shared::channels::ChannelsConfiguration,
};

use lib::protocol::{ClientChannel, ClientMessage, ServerMessage};

fn main() {
    let asset_path = match env::var("CARGO_MANIFEST_DIR") {
        Ok(manifest_dir) => PathBuf::from(manifest_dir)
            .parent()
            .unwrap()
            .to_path_buf()
            .join("assets"),
        _ => PathBuf::from("assets"),
    };

    App::new()
        .add_plugins(DefaultPlugins.set(AssetPlugin {
            file_path: asset_path.to_str().unwrap().to_string(),
            ..default()
        }))
        .add_plugins(QuinnetClientPlugin::default())
        .insert_resource(ClearColor(Srgba::gray(0.25).into()))
        .add_systems(Startup, (setup, start_connection))
        .add_systems(Update, (update, handle_server_messages))
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

fn update(
    mut query: Query<&mut Transform, With<Sprite>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    client: Res<QuinnetClient>,
) {
    let mut input = Vec2::ZERO;

    if keyboard_input.pressed(KeyCode::KeyA) {
        input.x -= 1.0;
    };
    if keyboard_input.pressed(KeyCode::KeyD) {
        input.x += 1.0;
    };
    if keyboard_input.pressed(KeyCode::KeyW) {
        input.y += 1.0;
    };
    if keyboard_input.pressed(KeyCode::KeyS) {
        input.y -= 1.0;
    };

    client
        .connection()
        .try_send_message_on(ClientChannel::CubeInput, ClientMessage::PlayerInput(input))
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

fn handle_server_messages(mut client: ResMut<QuinnetClient>, mut bevy: Query<&mut Transform, With<Sprite>>) {
    while let Ok(Some((_, message))) = client.connection_mut().receive_message::<ServerMessage>() {
        match message {
            ServerMessage::InitClient(client_id) => {
                info!("InitClient");
            }
            ServerMessage::SpawnCube {
                owner_client_id,
                entity,
                position,
            } => {
                info!("SpawnCube");
            }
            ServerMessage::CubeMoved { entity, position } => {
                info!("CubeMoved, {position}");

                let Ok(mut transform) = bevy.get_single_mut() else { continue; };

                transform.translation.x = position.x;
                transform.translation.y = position.y;
            }
        }
    }
}
