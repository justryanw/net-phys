use avian2d::prelude::*;
use bevy::color::palettes::css;
use bevy::diagnostic::LogDiagnosticsPlugin;
use bevy::prelude::*;
use bevy::render::RenderPlugin;
use bevy::utils::Duration;
use leafwing_input_manager::prelude::ActionState;

use lightyear::client::prediction::diagnostics::PredictionDiagnosticsPlugin;
use lightyear::prelude::client::*;
use lightyear::prelude::TickManager;
use lightyear::prelude::*;
use lightyear::transport::io::IoDiagnosticsPlugin;

use crate::protocol::*;
const MAX_VELOCITY: f32 = 200.0;
const WALL_SIZE: f32 = 350.0;

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub enum FixedSet {
    // main fixed update systems (handle inputs)
    Main,
    // apply physics steps
    Physics,
}

#[derive(Clone)]
pub struct SharedPlugin {
    pub(crate) show_confirmed: bool,
}

impl Plugin for SharedPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ProtocolPlugin);
        if app.is_plugin_added::<RenderPlugin>() {
            app.add_systems(Startup, init_camera);

            app.add_systems(
                PostUpdate,
                (draw_elements, draw_confirmed_shadows)
                    .after(InterpolationSet::Interpolate)
                    .after(PredictionSet::VisualCorrection),
            );
        }

        app.add_systems(Startup, init);

        app.add_plugins(PhysicsPlugins::new(FixedUpdate))
            .insert_resource(Time::new_with(Physics::fixed_once_hz(FIXED_TIMESTEP_HZ)))
            .insert_resource(Gravity(Vec2::ZERO));
        app.configure_sets(
            FixedUpdate,
            (
                (
                    PhysicsSet::Prepare,
                    PhysicsSet::StepSimulation,
                    PhysicsSet::Sync,
                )
                    .in_set(FixedSet::Physics),
                (FixedSet::Main, FixedSet::Physics).chain(),
            ),
        );

        app.register_type::<PlayerId>();
    }
}

pub(crate) fn color_from_id(client_id: ClientId) -> Color {
    let h = client_id.to_bits() as f32 * 10.0;
    let s = 1.0;
    let l = 0.5;
    Color::hsl(h, s, l)
}

fn init_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

pub(crate) fn init(mut commands: Commands) {
    commands.spawn(WallBundle::new(
        Vec2::new(-WALL_SIZE, -WALL_SIZE),
        Vec2::new(-WALL_SIZE, WALL_SIZE),
        Color::WHITE,
    ));
    commands.spawn(WallBundle::new(
        Vec2::new(-WALL_SIZE, WALL_SIZE),
        Vec2::new(WALL_SIZE, WALL_SIZE),
        Color::WHITE,
    ));
    commands.spawn(WallBundle::new(
        Vec2::new(WALL_SIZE, WALL_SIZE),
        Vec2::new(WALL_SIZE, -WALL_SIZE),
        Color::WHITE,
    ));
    commands.spawn(WallBundle::new(
        Vec2::new(WALL_SIZE, -WALL_SIZE),
        Vec2::new(-WALL_SIZE, -WALL_SIZE),
        Color::WHITE,
    ));
}

pub(crate) fn shared_movement_behaviour(
    mut velocity: Mut<LinearVelocity>,
    action: &ActionState<PlayerActions>,
) {
    const MOVE_SPEED: f32 = 10.0;

    let axis_pair = action.axis_pair(&PlayerActions::Move).expect("Could not get Move action");
    velocity.0 += axis_pair.xy() * MOVE_SPEED;

    *velocity = LinearVelocity(velocity.clamp_length_max(MAX_VELOCITY));
}

pub(crate) fn draw_elements(
    mut gizmos: Gizmos,
    players: Query<(&Position, &Rotation, &ColorComponent), (Without<Confirmed>, With<PlayerId>)>,
    balls: Query<(&Position, &ColorComponent), (Without<Confirmed>, With<BallMarker>)>,
    walls: Query<(&Wall, &ColorComponent), (Without<BallMarker>, Without<PlayerId>)>,
) {
    for (position, rotation, color) in &players {
        gizmos.rect_2d(
            Vec2::new(position.x, position.y),
            rotation.as_radians(),
            Vec2::ONE * PLAYER_SIZE,
            color.0,
        );
    }
    for (position, color) in &balls {
        gizmos.circle_2d(Vec2::new(position.x, position.y), BALL_SIZE, color.0);
    }
    for (wall, color) in &walls {
        gizmos.line_2d(wall.start, wall.end, color.0);
    }
}

pub(crate) fn draw_confirmed_shadows(
    mut gizmos: Gizmos,
    confirmed_q: Query<(&Position, &Rotation, &LinearVelocity, &Confirmed), With<PlayerId>>,
    predicted_q: Query<&Position, With<PlayerId>>,
) {
    for (position, rotation, velocity, confirmed) in confirmed_q.iter() {
        let speed = velocity.length() / MAX_VELOCITY;
        let ghost_col = css::GRAY.with_alpha(speed);
        gizmos.rect_2d(
            Vec2::new(position.x, position.y),
            rotation.as_radians(),
            Vec2::ONE * PLAYER_SIZE,
            ghost_col,
        );
        if let Some(e) = confirmed.predicted {
            if let Ok(pos) = predicted_q.get(e) {
                gizmos.line_2d(**position, **pos, ghost_col);
            }
        }
    }
}

#[derive(Bundle)]
pub(crate) struct WallBundle {
    color: ColorComponent,
    physics: PhysicsBundle,
    wall: Wall,
}

#[derive(Component)]
pub(crate) struct Wall {
    start: Vec2,
    end: Vec2,
}

impl WallBundle {
    pub(crate) fn new(start: Vec2, end: Vec2, color: Color) -> Self {
        Self {
            color: ColorComponent(color),
            physics: PhysicsBundle {
                collider: Collider::segment(start, end),
                collider_density: ColliderDensity(1.0),
                rigid_body: RigidBody::Static,
            },
            wall: Wall { start, end },
        }
    }
}

pub const FIXED_TIMESTEP_HZ: f64 = 64.0;
pub const SERVER_REPLICATION_INTERVAL: Duration = Duration::ZERO;

pub fn shared_config(mode: Mode) -> SharedConfig {
    SharedConfig {
        server_replication_send_interval: SERVER_REPLICATION_INTERVAL,
        tick: TickConfig {
            tick_duration: Duration::from_secs_f64(1.0 / FIXED_TIMESTEP_HZ),
        },
        mode,
    }
}
