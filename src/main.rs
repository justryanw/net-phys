use bevy::{prelude::*, render::camera::ScalingMode};
use bevy_xpbd_2d::prelude::*;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::BLACK))
        .add_plugins((DefaultPlugins, PhysicsPlugins::default()))
        .add_systems(Startup, setup)
        .add_systems(Update, rotate)
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Camera
    commands.spawn(Camera2dBundle {
        projection: OrthographicProjection {
            scaling_mode: ScalingMode::AutoMin {
                min_width: 10.0,
                min_height: 10.0,
            },
            ..Default::default()
        },
        ..Default::default()
    });

    // Sprite
    // let img = asset_server.load("bevy.png");
    commands.spawn((
        RigidBody::Dynamic,
        Collider::cuboid(1.0, 1.0),
        SpriteBundle {
            // texture: img.clone(),
            sprite: Sprite {
                custom_size: Some(Vec2::new(1.0, 1.0)),
                ..Default::default()
            },
            ..Default::default()
        },
    ));

    commands.spawn((
        RigidBody::Dynamic,
        Collider::cuboid(1.0, 1.0),
        SpriteBundle {
            transform: Transform::from_xyz(0.2, 2.0, 0.0),
            // texture: img.clone(),
            sprite: Sprite {
                custom_size: Some(Vec2::new(1.0, 1.0)),
                ..Default::default()
            },
            ..Default::default()
        },
    ));

    commands.spawn((
        RigidBody::Dynamic,
        Collider::cuboid(1.0, 1.0),
        SpriteBundle {
            transform: Transform::from_xyz(-0.2, 4.0, 0.0),
            // texture: img.clone(),
            sprite: Sprite {
                custom_size: Some(Vec2::new(1.0, 1.0)),
                ..Default::default()
            },
            ..Default::default()
        },
    ));

    commands.spawn((
        RigidBody::Static,
        Collider::cuboid(10.0, 1.0),
        SpriteBundle {
            transform: Transform::from_xyz(0.0, -4.5, 0.0),
            sprite : Sprite {
                color: Color::GRAY,
                custom_size: Some(Vec2::new(10.0, 1.0)),
                ..Default::default()
            },
            ..Default::default()
        }
    ));
}

fn rotate(mut query: Query<&mut Transform, With<Sprite>>, time: Res<Time>) {
    // for mut tr in &mut query {
    //     tr.rotate_local_z(-time.delta_seconds());
    // }
}
