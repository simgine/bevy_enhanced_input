//! Demonstrates how to create a simple platforming 2D character controller
//! using actions with both keyboard and gamepad controls.
//!
//! For kinematic character controllers, input should be accumulated and applied
//! to physics in a fixed timestep as recommended in [this Bevy example](https://bevy.org/examples/movement/physics-in-fixed-timestep/)
//! and as used in [bevy_ahoy](https://github.com/janhohenheim/bevy_ahoy).

use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;

const GROUND_LEVEL: f32 = -200.0;
const GROUND_WIDTH: f32 = 1200.0;
const PLAYER: Vec2 = Vec2::new(50.0, 100.0);
const JUMP_VELOCITY: f32 = 300.0;
const GRAVITY: f32 = 900.0;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, EnhancedInputPlugin))
        .add_input_context::<Player>()
        .add_systems(Startup, setup)
        .init_resource::<FixedUpdateRan>()
        .add_systems(PreUpdate, reset_fixed_update_ran)
        .add_systems(FixedPreUpdate, set_fixed_update_ran)
        .add_systems(FixedUpdate, calculate_physics)
        .add_systems(RunFixedMainLoop, clear_input.run_if(fixed_update_ran))
        .add_systems(FixedPostUpdate, apply_input)
        .add_observer(apply_movement)
        .add_observer(apply_jump)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn(Camera2d);

    // Ground
    commands.spawn((
        Mesh2d(meshes.add(Rectangle::new(GROUND_WIDTH, 5.0))),
        MeshMaterial2d(materials.add(Color::srgb(0.0, 1.0, 0.5))),
        Transform::from_translation(Vec3::Y * GROUND_LEVEL),
    ));

    commands.spawn((
        Player,
        Mesh2d(meshes.add(Rectangle::new(PLAYER.x, PLAYER.y))),
        MeshMaterial2d(materials.add(Color::srgb(1.0, 0.0, 0.5))),
        Transform::from_translation(Vec3::Y * (GROUND_LEVEL + 500.0)),
        PlayerPhysics::default(),
        AccumulatedInput::default(),
        actions!(Player[
            (
                Action::<Movement>::new(),
                DeadZone::default(),
                SmoothNudge::default(),
                // We don't apply `DeltaScale` here because the movement vector is
                // multiplied by delta time during the physics calculation.
                Scale::splat(450.0),
                Bindings::spawn((
                    Bidirectional::new(KeyCode::KeyD, KeyCode::KeyA),
                    Bidirectional::new(KeyCode::ArrowRight, KeyCode::ArrowLeft),
                    Axial::left_stick(),
                )),
            ),
            (
                Action::<Jump>::new(),
                bindings![KeyCode::Space, GamepadButton::South],
            )
        ]),
    ));
}

fn apply_movement(movement: On<Fire<Movement>>, mut inputs: Query<&mut AccumulatedInput>) {
    let mut accumulated_inputs = inputs.get_mut(movement.context).unwrap();
    accumulated_inputs.movement = movement.value;
}

fn apply_jump(jump: On<Fire<Jump>>, mut inputs: Query<&mut AccumulatedInput>) {
    let mut accumulated_inputs = inputs.get_mut(jump.context).unwrap();
    accumulated_inputs.jump = true;
}

fn clear_input(mut inputs: Query<&mut AccumulatedInput>) {
    for mut inputs in &mut inputs {
        *inputs = Default::default();
    }
}

fn apply_input(players: Query<(&mut PlayerPhysics, &AccumulatedInput)>) {
    for (mut physics, input) in players {
        physics.velocity.x = input.movement;
        if input.jump && physics.is_grounded {
            physics.velocity.y = JUMP_VELOCITY;
            physics.is_grounded = false;
        }
    }
}

fn calculate_physics(
    fixed_time: Res<Time<Fixed>>,
    mut players: Query<(&mut Transform, &mut PlayerPhysics)>,
) {
    for (mut transform, mut physics) in &mut players {
        physics.velocity.y -= GRAVITY * fixed_time.delta_secs();
        transform.translation.y += physics.velocity.y * fixed_time.delta_secs();
        transform.translation.x += physics.velocity.x * fixed_time.delta_secs();

        // Prevent moving off screen.
        const MAX_X: f32 = GROUND_WIDTH / 2.0 - PLAYER.x / 2.0;
        transform.translation.x = transform.translation.x.clamp(-MAX_X, MAX_X);

        // Check for ground collision.
        const GROUNDED_Y: f32 = GROUND_LEVEL + PLAYER.y / 2.0;
        if transform.translation.y <= GROUNDED_Y {
            transform.translation.y = GROUNDED_Y;
            physics.velocity.y = 0.0;
            physics.is_grounded = true;
        }
    }
}

#[derive(Component)]
struct Player;

#[derive(Component, Default)]
struct PlayerPhysics {
    velocity: Vec2,
    is_grounded: bool,
}

#[derive(Debug, InputAction)]
#[action_output(f32)]
struct Movement;

#[derive(Debug, InputAction)]
#[action_output(bool)]
struct Jump;

/// Accumulated input since the last fixed update.
#[derive(Component, Default)]
struct AccumulatedInput {
    movement: f32,
    jump: bool,
}

// Fixed timestep boilerplate
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
