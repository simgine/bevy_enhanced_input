//! One context completely replaces another.

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

fn apply_movement(trigger: Trigger<Fired<Move>>) {
    info!("moving: {}", trigger.value);
}

fn attack(_trigger: Trigger<Fired<Attack>>) {
    info!("attacking");
}

fn open_inventory(trigger: Trigger<Started<OpenInventory>>, mut commands: Commands) {
    info!("opening inventory");
    commands.entity(trigger.target()).insert((
        ContextActivity::<Player>::INACTIVE,
        ContextActivity::<Inventory>::ACTIVE,
    ));
}

fn navigate_inventory(_trigger: Trigger<Fired<NavigateInventory>>) {
    info!("navigating inventory");
}

fn close_inventory(trigger: Trigger<Started<CloseInventory>>, mut commands: Commands) {
    info!("closing inventory");
    commands.entity(trigger.target()).insert((
        ContextActivity::<Player>::ACTIVE,
        ContextActivity::<Inventory>::INACTIVE,
    ));
}

fn player_bundle() -> impl Bundle {
    (
        Player,
        actions!(Player[
            (
                Action::<Move>::new(),
                DeadZone::default(),
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
struct Move;

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
