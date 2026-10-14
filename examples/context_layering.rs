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
//! The [`Driving`] context consumes the jump inputs with a brake action and adds an action for exiting
//! the vehicle. The action for entering the vehicle belongs to the [`Player`] context.

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
    commands.spawn_scene(bsn! {
        Player
        Actions<Player> [
            Action::<Movement>::new()
            DeadZone
            DeltaScale
            ~Scale::splat(10.0)
            Bindings [
                {preset::cardinal::wasd_keys()}
                --
                {preset::axial::left_stick()}
            ]
            --
            Action::<Jump>::new()
            Bindings [
                Binding(KeyCode::Space)
                --
                Binding(GamepadButton::South)
            ]
            --
            Action::<EnterCar>::new()
            Bindings [
                Binding(KeyCode::Enter)
                --
                Binding(GamepadButton::North)
            ]
        ]
    });
}

fn apply_movement(movement: On<Fire<Movement>>) {
    info!("moving: {}", movement.value);
}

fn jump(_on: On<Start<Jump>>) {
    info!("jumping");
}

fn enter_car(enter: On<Start<EnterCar>>, mut commands: Commands) {
    // `Player` has lower priority, so `Brake` and `ExitCar` consume inputs first,
    // preventing `Jump` and `EnterCar` from being triggered.
    // The consuming behavior can be configured using `ActionSettings` component.
    info!("entering car");
    commands.entity(enter.context).apply_scene(bsn! {
        Driving
        ~ContextPriority::<Driving>::new(1)
        Actions<Driving> [
            Action::<Brake>::new()
            ActionSettings {
                consume_input: true,
            }
            Bindings [
                Binding(KeyCode::Space)
                --
                Binding(GamepadButton::South)
            ]
            --
            Action::<ExitCar>::new()
            ActionSettings {
                // We set `require_reset` to `true` because `EnterCar` action uses the same input,
                // and we want it to be triggerable only after the button is released.
                require_reset: true,
                // We also set `consume_input` to `true` otherwise, we would immediately re-trigger `EnterCar`.
                consume_input: true,
            }
            Bindings [
                Binding(KeyCode::Enter)
                --
                Binding(GamepadButton::North)
            ]
        ]
    });
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

#[derive(Component, Default, Clone)]
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
#[derive(Component, Default, Clone)]
struct Driving;

/// This action overrides [`Jump`] when the player is [`Driving`].
#[derive(InputAction)]
#[action_output(bool)]
struct Brake;

/// Removes [`Driving`].
#[derive(InputAction)]
#[action_output(bool)]
struct ExitCar;
