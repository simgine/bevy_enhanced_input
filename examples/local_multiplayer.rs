//! Demonstrates how to set up local multiplayer input handling.
//!
//! The same context ([`Player`]) is used for both players, but each player has their own unique entity.
//! This allows us to enable or disable players independently and reuse the same entity for gameplay,
//! and assign unique input bindings to each player.

use bevy::{
    input::gamepad::{GamepadConnection, GamepadConnectionEvent},
    prelude::*,
};
use bevy_enhanced_input::prelude::*;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, EnhancedInputPlugin))
        .add_input_context::<Player>()
        .add_observer(apply_movement)
        .add_systems(Startup, spawn)
        .add_systems(Update, update_gamepads)
        .run();
}

fn spawn(
    mut commands: Commands,
    gamepads: Query<Entity, With<Gamepad>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 25.0, 0.0).looking_at(-Vec3::Y, Vec3::Y),
    ));

    commands.spawn((
        Mesh3d(meshes.add(Plane3d::new(Vec3::Y, Vec2::splat(10.0)))),
        MeshMaterial3d(materials.add(Color::WHITE)),
    ));
    commands.spawn((
        PointLight {
            shadow_maps_enabled: true,
            ..Default::default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0),
    ));

    // By default actions read inputs from all gamepads,
    // but for local multiplayer we need assign specific
    // gamepad index.
    let mut gamepads = gamepads.iter();
    let (gamepad1, gamepad2) = (gamepads.next(), gamepads.next());
    let capsule = meshes.add(Capsule3d::new(0.5, 2.0));

    // Spawn two players with different controls.
    commands.spawn(player_bundle(
        Player::First,
        gamepad1,
        capsule.clone(),
        materials.add(Color::srgb_u8(124, 144, 255)),
        Transform::from_xyz(0.0, 1.5, 8.0),
    ));
    commands.spawn(player_bundle(
        Player::Second,
        gamepad2,
        capsule,
        materials.add(Color::srgb_u8(220, 90, 90)),
        Transform::from_xyz(0.0, 1.5, -8.0),
    ));
}

fn apply_movement(movement: On<Fire<Movement>>, mut players: Query<&mut Transform>) {
    let mut transform = players.get_mut(movement.context).unwrap();

    // Adjust axes for top-down movement.
    transform.translation.z -= movement.value.x;
    transform.translation.x -= movement.value.y;

    // Prevent from moving out of plane.
    transform.translation.z = transform.translation.z.clamp(-10.0, 10.0);
    transform.translation.x = transform.translation.x.clamp(-10.0, 10.0);
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
    mesh: impl Into<Mesh3d>,
    material: impl Into<MeshMaterial3d<StandardMaterial>>,
    transform: Transform,
) -> impl Bundle {
    // Assign different bindings based on the player index.
    let move_bindings = match player {
        Player::First => Bindings::spawn((Cardinal::wasd_keys(), Axial::left_stick())),
        Player::Second => Bindings::spawn((Cardinal::arrows(), Axial::left_stick())),
    };

    (
        player,
        GamepadDevice::from(gamepad),
        mesh.into(),
        material.into(),
        transform,
        actions!(
            Player[(
                Action::<Movement>::new(),
                DeadZone::default(),
                SmoothNudge::default(),
                DeltaScale::default(),
                Scale::splat(10.0),
                move_bindings,
            )]
        ),
    )
}

#[derive(Component, Clone, Copy, PartialEq, Eq, Hash)]
enum Player {
    First,
    Second,
}

#[derive(Debug, InputAction)]
#[action_output(Vec2)]
struct Movement;
