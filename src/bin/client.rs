use bevy::{prelude::*, render::camera::ScalingMode};
use bevy_xpbd_2d::prelude::*;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::BLACK))
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands) {
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

    commands.spawn((
        RigidBody::Dynamic,
        Collider::cuboid(1.0, 1.0),
        SpriteBundle {
            sprite: Sprite {
                custom_size: Some(Vec2::new(1.0, 1.0)),
                ..Default::default()
            },
            ..Default::default()
        },
    ));
}