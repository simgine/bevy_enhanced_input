//! Two players that use the same context type, but with different bindings.

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
            shadows_enabled: true,
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

fn apply_movement(trigger: Trigger<Fired<Move>>, mut players: Query<&mut Transform>) {
    let mut transform = players.get_mut(trigger.target()).unwrap();

    // Adjust axes for top-down movement.
    transform.translation.z -= trigger.value.x;
    transform.translation.x -= trigger.value.y;

    // Prevent from moving out of plane.
    transform.translation.z = transform.translation.z.clamp(-10.0, 10.0);
    transform.translation.x = transform.translation.x.clamp(-10.0, 10.0);
}

fn update_gamepads(
    mut event_reader: EventReader<GamepadConnectionEvent>,
    mut players: Query<&mut GamepadDevice>,
) {
    for event in event_reader.read() {
        match event.connection {
            GamepadConnection::Connected { .. } => {
                // Assign to a player without a gamepad.
                if let Some(mut gamepad) = players
                    .iter_mut()
                    .find(|gamepad| **gamepad == GamepadDevice::None)
                {
                    *gamepad = event.gamepad.into();
                }
            }
            GamepadConnection::Disconnected => {
                // Unassign the disconnected gamepad.
                // Not necessary to do, but allows us conveniently
                // detect which player don't have a gamepad.
                if let Some(mut gamepad) = players
                    .iter_mut()
                    .find(|gamepad| **gamepad == event.gamepad.into())
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
                Action::<Move>::new(),
                DeadZone::default(),
                SmoothNudge::default(),
                Scale::splat(0.4),
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
struct Move;
