//! Demonstrates the concept of context layering in input handling. One context
//! can be applied on top of another, overriding some of the bindings.
//!
//! The [`ContextPriority`] component is used to determine the order of
//! contexts, with higher priority contexts taking precedence over lower
//! priority ones. This influences the order in which actions are evaluated and
//! inputs are consumed. See [`ActionSettings::consume_input`] for more details
//! and control over this behavior.
//!
//! In this example, we have a [`Player`] context that allows basic movement,
//! jumping, and muting audio. When the player enters a vehicle, we add a
//! [`Driving`] context on top of the [`Player`] context. The [`Driving`]
//! context overrides the jump action with a brake action and adds actions for
//! entering and exiting the vehicle, plus a map display action.
//!
//! # Exact Modifier Key Matching
//!
//! This example also demonstrates exact modifier key matching:
//! - The [`Player`] context has a [`ToggleMute`] action bound to `Shift+M`
//! - The [`Driving`] context has a [`ToggleMap`] action bound to just `M` (no
//!   modifiers)
//! - When driving and pressing `Shift+M`, only the mute action fires (not the
//!   map action)
//! - When driving and pressing just `M`, only the map action fires
//!
//! This works because bindings require **exact** modifier matches:
//! - A binding with no modifiers (`M`) only matches when NO modifiers are
//!   pressed.
//! - A binding with modifiers (`Shift+M`) only matches when EXACTLY those
//!   modifiers are pressed.
//!
//! # Controls
//!
//! - `WASD` or Left Stick: Move
//! - `Space` or South Button: Jump (or Brake when driving)
//! - `Enter` or North Button: Enter/Exit car
//! - `M`: Toggle map (only when driving, only without Shift)
//! - `Shift+M`: Toggle mute (works both on foot and while driving)

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
        .add_observer(toggle_mute)
        .add_observer(toggle_map)
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
            (
                Action::<ToggleMute>::new(),
                bindings![KeyCode::KeyM.with_mod_keys(ModKeys::SHIFT)]
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
    // `Player` has lower priority, so `Brake`, `ExitCar`, and `ToggleMap` consume inputs first,
    // preventing `Jump` and `EnterCar` from being triggered.
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
            (
                Action::<ToggleMap>::new(),
                ActionSettings {
                    // We set `consume_input` to `false` to allow the `ToggleMute` action
                    // (bound to Shift+M) to fire even when this action (bound to M) exists.
                    // Without this, the M key would be consumed even when Shift is pressed,
                    // but with exact modifier matching, M without modifiers won't match when Shift+M is pressed.
                    consume_input: false,
                    ..Default::default()
                },
                bindings![KeyCode::KeyM]
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

/// Toggle audio mute (bound to Shift+M in Player context).
#[derive(InputAction)]
#[action_output(bool)]
struct ToggleMute;

/// Toggle map display (bound to M in Driving context).
/// This demonstrates exact modifier matching: when Shift+M is pressed,
/// this action won't fire because it requires no modifiers.
#[derive(InputAction)]
#[action_output(bool)]
struct ToggleMap;

fn toggle_mute(_on: On<Start<ToggleMute>>) {
    info!("Toggling mute (Shift+M) - works on foot and while driving");
}

fn toggle_map(_on: On<Start<ToggleMap>>) {
    info!("Toggling map (M only, no modifiers) - only available while driving");
    info!("   Note: Pressing Shift+M will NOT trigger this action due to exact modifier matching");
}
