use std::{env, path::PathBuf};

use bevy::prelude::*;

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
        .insert_resource(ClearColor(Color::BLACK))
        .add_systems(Startup, setup)
        .add_systems(Update, rotate)
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
        bevy.rotate_local_z(time.delta_seconds());
    }
}
