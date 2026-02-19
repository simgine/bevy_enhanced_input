//! Demonstrates the use of context switching in input handling.
//!
//! When a context switch action is triggered, the active context changes,
//! enabling a different set of actions and bindings.
//! This is done by setting the [`ContextActivity`] component on each context entity,
//! enabling or disabling them as needed.
//!
//! In this example, the player can move and attack in the [`Player`] context.
//! Pressing the [`OpenInventory`] action switches to the [`Inventory`] context,
//! where the player can navigate the inventory and close it to return to the [`Player`] context.

use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, EnhancedInputPlugin))
        .add_input_context::<Player>()
        .add_input_context::<Inventory>()
        .add_observer(apply_movement)
        .add_observer(attack)
        .add_observer(open_inventory)
        .add_observer(navigate_inventory)
        .add_observer(close_inventory)
        .add_systems(Startup, spawn)
        .run();
}

fn spawn(mut commands: Commands) {
    commands.spawn(player_bundle());
}

fn apply_movement(movement: On<Fire<Movement>>) {
    info!("moving: {}", movement.value);
}

fn attack(_on: On<Fire<Attack>>) {
    info!("attacking");
}

fn open_inventory(open: On<Start<OpenInventory>>, mut commands: Commands) {
    info!("opening inventory");
    commands.entity(open.context).insert((
        ContextActivity::<Player>::INACTIVE,
        ContextActivity::<Inventory>::ACTIVE,
    ));
}

fn navigate_inventory(_on: On<Fire<NavigateInventory>>) {
    info!("navigating inventory");
}

fn close_inventory(close: On<Start<CloseInventory>>, mut commands: Commands) {
    info!("closing inventory");
    commands.entity(close.context).insert((
        ContextActivity::<Player>::ACTIVE,
        ContextActivity::<Inventory>::INACTIVE,
    ));
}

fn player_bundle() -> impl Bundle {
    (
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
                Action::<Attack>::new(),
                bindings![MouseButton::Left, GamepadButton::West],
            ),
            (
                Action::<OpenInventory>::new(),
                // We set `require_reset` to `true` because `CloseInventory` action uses the same input,
                // and we want it to be triggerable only after the button is released.
                ActionSettings {
                    require_reset: true,
                    ..Default::default()
                },
                bindings![KeyCode::KeyI, GamepadButton::Select],
            ),
        ]),
        Inventory,
        actions!(Inventory[
            (
                Action::<NavigateInventory>::new(),
                Bindings::spawn((Cardinal::wasd_keys(), Axial::left_stick())),
                Pulse::new(0.2), // Avoid triggering every frame on hold for UI.
            ),
            (
                Action::<CloseInventory>::new(),
                ActionSettings {
                    require_reset: true,
                    ..Default::default()
                },
                bindings![KeyCode::KeyI, GamepadButton::Select],
            )
        ]),
    )
}

#[derive(Component)]
struct Player;

#[derive(InputAction)]
#[action_output(Vec2)]
struct Movement;

#[derive(InputAction)]
#[action_output(bool)]
struct Attack;

/// Switches context to [`Inventory`].
#[derive(InputAction)]
#[action_output(bool)]
struct OpenInventory;

#[derive(Component)]
struct Inventory;

#[derive(InputAction)]
#[action_output(Vec2)]
struct NavigateInventory;

/// Switches context to [`Player`].
#[derive(InputAction)]
#[action_output(bool)]
struct CloseInventory;
