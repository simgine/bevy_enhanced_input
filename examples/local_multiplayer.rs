//! Demonstrates how to set up local multiplayer input handling.
//!
//! The same context ([`Player`]) is used for both players, but each player has their own unique entity.
//! This allows us to enable or disable players independently and reuse the same entity for gameplay,
//! and assign unique input bindings to each player.
//!
//! Repeats the best practices used in `character_controller` example.

use bevy::{
    input::gamepad::{GamepadConnection, GamepadConnectionEvent},
    prelude::*,
};
use bevy_enhanced_input::prelude::{Press, *};

const BORDER_WIDTH: f32 = 650.0;
const STROKE_WIDTH: f32 = 5.0;
const BALL_RAD: f32 = 16.0;
const ACCELERATION: f32 = 800.0;
const KICK_IMPULSE: f32 = 1000.0;
const FRICTION: f32 = 25.0;
const KINETIC_FRICTION_COEFF: f32 = 3.0;
const BOUNCINESS: f32 = 0.8;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, EnhancedInputPlugin))
        .add_input_context::<Player>()
        .init_resource::<FixedUpdateRan>()
        .add_systems(Startup, setup)
        .add_systems(PreUpdate, (reset_fixed_update_ran, update_gamepads))
        .add_systems(FixedPreUpdate, set_fixed_update_ran)
        .add_systems(FixedUpdate, apply_input)
        .add_systems(FixedPostUpdate, advance_physics)
        .add_systems(
            RunFixedMainLoop,
            clear_input
                .run_if(fixed_update_ran)
                .in_set(RunFixedMainLoopSystems::AfterFixedMainLoop),
        )
        .add_observer(accumulate_roll)
        .add_observer(accumulate_kick)
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
    let material1 = materials.add(Color::srgb(0.1, 0.1, 0.9));
    commands.spawn(player_bundle(
        Player::First,
        gamepad1,
        ball_mesh.clone(),
        material1,
        Transform::from_xyz(-80.0, 0.0, 0.0),
    ));

    let material2 = materials.add(Color::srgb(0.9, 0.1, 0.1));
    commands.spawn(player_bundle(
        Player::Second,
        gamepad2,
        ball_mesh,
        material2,
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
        AccumulatedInput::default(),
        mesh.into(),
        material.into(),
        transform,
        Actions::<Player>::spawn(SpawnWith(move |context: &mut ActionSpawner<_>| {
            context.spawn((
                Action::<Roll>::new(),
                DeadZone::default(),
                DeltaScale::default(),
                Scale::splat(ACCELERATION),
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

fn accumulate_roll(roll: On<Fire<Roll>>, mut input: Query<&mut AccumulatedInput>) {
    let mut input = input.get_mut(roll.context).unwrap();
    input.roll = roll.value;
}

fn accumulate_kick(kick: On<Fire<Kick>>, mut input: Query<&mut AccumulatedInput>) {
    let mut input = input.get_mut(kick.context).unwrap();
    input.kick = Some(kick.value);
}

fn clear_input(mut inputs: Query<&mut AccumulatedInput>) {
    for mut inputs in &mut inputs {
        *inputs = Default::default();
    }
}

fn apply_input(players: Query<(&mut PlayerPhysics, &AccumulatedInput)>) {
    for (mut physics, input) in players {
        physics.velocity += input.roll;

        if let Some(kick) = input.kick {
            // Normalize the input to treat vectors that are barely inside the threshold
            // the same way as a vector along the edge.
            let dir = kick.normalize();
            physics.velocity = dir * KICK_IMPULSE;
        }
    }
}

fn advance_physics(time: Res<Time>, players: Query<(&mut Transform, &mut PlayerPhysics)>) {
    for (mut transform, mut physics) in players {
        transform.translation += (physics.velocity * time.delta_secs()).extend(0.0);

        // Apply friction.
        if physics.velocity.length_squared() > KINETIC_FRICTION_COEFF {
            let friction_dir = physics.velocity.normalize() * -1.0;
            physics.velocity += friction_dir * FRICTION * time.delta_secs();
        }

        // Check collision with walls and bounce.
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

#[derive(Component, Default)]
struct AccumulatedInput {
    roll: Vec2,
    kick: Option<Vec2>,
}

/// True if FixedPreUpdate was run this frame.
#[derive(Resource, Deref, DerefMut, Default)]
struct FixedUpdateRan(bool);

fn reset_fixed_update_ran(mut ran: ResMut<FixedUpdateRan>) {
    **ran = false;
}

fn set_fixed_update_ran(mut ran: ResMut<FixedUpdateRan>) {
    **ran = true;
}

fn fixed_update_ran(ran: Res<FixedUpdateRan>) -> bool {
    **ran
}
