//! Demonstrates the concept of context layering in input handling.
//! One context can be applied on top of another, overriding some of the bindings.
//!
//! The [`ContextPriority`] component is used to determine the order of contexts,
//! with higher priority contexts taking precedence over lower priority ones.
//! This influences the order in which actions are evaluated and inputs are consumed.
//! See [`ActionSettings::consume_input`] for more details and control over this behavior.
//!
//! In this example, we have a [`Player`] context that allows basic movement and jumping.
//! When the player enters a vehicle, we add a [`Driving`] context on top of the [`Player`] context.
//! The [`Driving`] context overrides the jump action with a brake action and adds actions for entering
//! and exiting the vehicle.

use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, EnhancedInputPlugin))
        .add_input_context::<Player>()
        .add_input_context::<Driving>()
        .add_observer(apply_movement)
        .add_observer(jump)
        .add_observer(exit_car)
        .add_observer(enter_car)
        .add_observer(brake)
        .add_systems(Startup, spawn)
        .run();
}

fn spawn(mut commands: Commands) {
    commands.spawn((
        Player,
        actions!(Player[
            (
                Action::<Movement>::new(),
                DeadZone::default(),
                DeltaScale::default(),
                Scale::splat(10.0),
                Bindings::spawn((Cardinal::wasd_keys(), Axial::left_stick())),
            ),
            (
                Action::<Jump>::new(),
                bindings![KeyCode::Space, GamepadButton::South]
            ),
            (
                Action::<EnterCar>::new(),
                bindings![KeyCode::Enter, GamepadButton::North]
            ),
        ]),
    ));
}

fn apply_movement(movement: On<Fire<Movement>>) {
    info!("moving: {}", movement.value);
}

fn jump(_on: On<Start<Jump>>) {
    info!("jumping");
}

fn enter_car(enter: On<Start<EnterCar>>, mut commands: Commands) {
    // `Player` has lower priority, so `Brake` and `ExitCar` consume inputs first,
    // preventing `Rotate` and `EnterCar` from being triggered.
    // The consuming behavior can be configured using `ActionSettings` component.
    info!("entering car");
    commands.entity(enter.context).insert((
        Driving,
        ContextPriority::<Driving>::new(1),
        actions!(Driving[
            (
                Action::<Brake>::new(),
                bindings![KeyCode::Space, GamepadButton::South]
            ),
            (
                Action::<ExitCar>::new(),
                ActionSettings {
                    // We set `require_reset` to `true` because `EnterCar` action uses the same input,
                    // and we want it to be triggerable only after the button is released.
                    require_reset: true,
                    ..Default::default()
                },
                bindings![KeyCode::Enter, GamepadButton::North]
            ),
        ]),
    ));
}

fn brake(_on: On<Fire<Brake>>) {
    info!("braking");
}

fn exit_car(exit: On<Start<ExitCar>>, mut commands: Commands) {
    info!("exiting car");
    commands
        .entity(exit.context)
        .remove_with_requires::<Driving>() // Necessary to fully remove the context.
        .despawn_related::<Actions<Driving>>();
}

#[derive(Component)]
struct Player;

#[derive(InputAction)]
#[action_output(Vec2)]
struct Movement;

#[derive(InputAction)]
#[action_output(bool)]
struct Jump;

/// Adds [`Driving`].
#[derive(InputAction)]
#[action_output(bool)]
struct EnterCar;

/// Overrides some actions from [`Player`].
#[derive(Component)]
struct Driving;

/// This action overrides [`Jump`] when the player is [`Driving`].
#[derive(InputAction)]
#[action_output(bool)]
struct Brake;

/// Removes [`Driving`].
#[derive(InputAction)]
#[action_output(bool)]
struct ExitCar;
