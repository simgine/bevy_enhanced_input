//! Demonstrates how to set up local multiplayer input handling.
//!
//! The same context ([`Player`]) is used for both players, but each player has their own unique entity.
//! This allows us to enable or disable players independently and reuse the same entity for gameplay,
//! and assign unique input bindings to each player.

use bevy::{
    input::gamepad::{GamepadConnection, GamepadConnectionEvent},
    prelude::*,
};
use bevy_enhanced_input::prelude::{Press, *};

const BORDER_WIDTH: f32 = 650.0;
const STROKE_WIDTH: f32 = 5.0;
const BALL_RAD: f32 = 16.0;
const ACCELERATION: f32 = 800.0;
const MAX_ROLL_SPEED: f32 = 400.0;
const KICK_IMPULSE: f32 = 1000.0;
const FRICTION: f32 = 25.0;
const KINETIC_FRICTION_COEFF: f32 = 3.0;
const BOUNCINESS: f32 = 0.8;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, EnhancedInputPlugin))
        .add_input_context::<Player>()
        .add_systems(Startup, setup)
        .add_systems(Update, (calculate_physics, update_gamepads))
        .add_observer(apply_roll)
        .add_observer(apply_kick)
        .run();
}

fn setup(
    mut commands: Commands,
    gamepads: Query<Entity, With<Gamepad>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn(Camera2d);

    // Border
    let border_mesh = meshes.add(Rectangle::new(BORDER_WIDTH, STROKE_WIDTH));
    let border_mat = materials.add(Color::WHITE);
    // Bottom
    commands.spawn((
        Mesh2d(border_mesh.clone()),
        MeshMaterial2d(border_mat.clone()),
        Transform::from_translation(Vec3::Y * (-BORDER_WIDTH / 2.0)),
    ));
    // Top
    commands.spawn((
        Mesh2d(border_mesh.clone()),
        MeshMaterial2d(border_mat.clone()),
        Transform::from_translation(Vec3::Y * (BORDER_WIDTH / 2.0)),
    ));
    // Left
    commands.spawn((
        Mesh2d(border_mesh.clone()),
        MeshMaterial2d(border_mat.clone()),
        Transform::from_translation(Vec3::X * (-BORDER_WIDTH / 2.0))
            .with_rotation(Quat::from_rotation_z(90.0f32.to_radians())),
    ));
    // Right
    commands.spawn((
        Mesh2d(border_mesh),
        MeshMaterial2d(border_mat),
        Transform::from_translation(Vec3::X * (BORDER_WIDTH / 2.0))
            .with_rotation(Quat::from_rotation_z(90.0f32.to_radians())),
    ));

    let mut gamepads = gamepads.iter();
    let (gamepad1, gamepad2) = (gamepads.next(), gamepads.next());
    let ball_mesh = meshes.add(Circle::new(BALL_RAD));

    // Player 1
    let p1_mat = materials.add(Color::srgb(0.1, 0.1, 0.9));
    commands.spawn(player_bundle(
        Player::First,
        gamepad1,
        ball_mesh.clone(),
        p1_mat,
        Transform::from_xyz(-80.0, 0.0, 0.0),
    ));

    let p2_mat = materials.add(Color::srgb(0.9, 0.1, 0.1));
    commands.spawn(player_bundle(
        Player::Second,
        gamepad2,
        ball_mesh,
        p2_mat,
        Transform::from_xyz(80.0, 0.0, 0.0),
    ));
}

fn update_gamepads(
    mut gamepad_connections: MessageReader<GamepadConnectionEvent>,
    mut players: Query<&mut GamepadDevice>,
) {
    for connection in gamepad_connections.read() {
        match connection.connection {
            GamepadConnection::Connected { .. } => {
                // Assign to a player without a gamepad.
                if let Some(mut gamepad) = players
                    .iter_mut()
                    .find(|gamepad| **gamepad == GamepadDevice::None)
                {
                    *gamepad = connection.gamepad.into();
                }
            }
            GamepadConnection::Disconnected => {
                // Unassign the disconnected gamepad.
                // Not necessary to do, but allows us conveniently
                // detect which player don't have a gamepad.
                if let Some(mut gamepad) = players
                    .iter_mut()
                    .find(|gamepad| **gamepad == connection.gamepad.into())
                {
                    *gamepad = GamepadDevice::None;
                }
            }
        }
    }
}

fn player_bundle(
    player: Player,
    gamepad: Option<Entity>,
    mesh: impl Into<Mesh2d>,
    material: impl Into<MeshMaterial2d<ColorMaterial>>,
    transform: Transform,
) -> impl Bundle {
    // Assign different bindings based on the player index.
    let dir_bindings = match player {
        Player::First => (Cardinal::wasd_keys(), Axial::left_stick()),
        Player::Second => (Cardinal::arrows(), Axial::left_stick()),
    };
    let arm_kick_binding = match player {
        Player::First => bindings![KeyCode::Space],
        Player::Second => bindings![KeyCode::ShiftRight],
    };

    (
        player,
        GamepadDevice::from(gamepad),
        PlayerPhysics::default(),
        mesh.into(),
        material.into(),
        transform,
        Actions::<Player>::spawn(SpawnWith(move |context: &mut ActionSpawner<_>| {
            context.spawn((
                Action::<Roll>::new(),
                DeadZone::default(),
                DeltaScale::default(),
                Bindings::spawn(dir_bindings),
            ));

            // For controller: Kick by flicking the controller stick
            context.spawn((
                Action::<Kick>::new(),
                // Replicate feel of Smash Bros., which requires the controller
                // to flick in ~2 frames (1/30 second) for certain actions
                Flick::new(0.0333).with_actuation(0.9),
                Bindings::spawn(Axial::left_stick()),
            ));

            // As an alternative for keyboard players, kick by "arming" and then
            // pressing a movement key.
            let arm = context
                .spawn((Action::<ArmKick>::new(), arm_kick_binding))
                .id();
            context.spawn((
                Action::<Kick>::new(),
                Chord::single(arm),
                Press::new(0.9),
                Bindings::spawn(dir_bindings),
            ));
        })),
    )
}

fn apply_roll(roll: On<Fire<Roll>>, mut physics: Query<&mut PlayerPhysics>) {
    let mut physics = physics.get_mut(roll.context).unwrap();
    let cur_vel = physics.velocity;
    let input_dir = roll.value;
    let mut delta_v = ACCELERATION * input_dir;

    // Cap speed, but allow pulling against speed when moving too fast
    if (cur_vel + delta_v).length_squared() >= MAX_ROLL_SPEED * MAX_ROLL_SPEED {
        // Only apply perpendicular component of velocity to disallow user
        // from speeding up past max
        let perp_v = Vec2::new(cur_vel.y, -cur_vel.x).normalize_or_zero();
        let dot = input_dir.dot(perp_v);
        delta_v = perp_v * dot * ACCELERATION;
    }

    physics.velocity += delta_v;
}

fn apply_kick(kick: On<Fire<Kick>>, mut physics: Query<&mut PlayerPhysics>) {
    let mut physics = physics.get_mut(kick.context).unwrap();
    // Normalize the input to treat vectors that are barely inside the threshold
    // the same way as a vector along the edge.
    let dir = kick.value.normalize();

    physics.velocity = dir * KICK_IMPULSE;
}

fn calculate_physics(time: Res<Time>, players: Query<(&mut Transform, &mut PlayerPhysics)>) {
    for (mut transform, mut physics) in players {
        // Apply velocity to transform
        transform.translation += (physics.velocity * time.delta_secs()).extend(0.0);

        // Apply friction to velocity
        if physics.velocity.length_squared() > KINETIC_FRICTION_COEFF {
            let friction_dir = physics.velocity.normalize() * -1.0;
            physics.velocity += friction_dir * FRICTION * time.delta_secs();
        }

        // Check collision with walls and bounce
        const BORDER_DIST: f32 = BORDER_WIDTH / 2.0 - BALL_RAD;
        if transform.translation.x > BORDER_DIST {
            transform.translation.x = BORDER_DIST;
            physics.velocity.x *= -BOUNCINESS;
        }
        if transform.translation.x < -BORDER_DIST {
            transform.translation.x = -BORDER_DIST;
            physics.velocity.x *= -BOUNCINESS;
        }
        if transform.translation.y > BORDER_DIST {
            transform.translation.y = BORDER_DIST;
            physics.velocity.y *= -BOUNCINESS;
        }
        if transform.translation.y < -BORDER_DIST {
            transform.translation.y = -BORDER_DIST;
            physics.velocity.y *= -BOUNCINESS;
        }
    }
}

#[derive(Component)]
enum Player {
    First,
    Second,
}

#[derive(Component, Default)]
struct PlayerPhysics {
    velocity: Vec2,
}

#[derive(InputAction)]
#[action_output(Vec2)]
struct Roll;

#[derive(InputAction)]
#[action_output(Vec2)]
struct Kick;

#[derive(InputAction)]
#[action_output(bool)]
struct ArmKick;
