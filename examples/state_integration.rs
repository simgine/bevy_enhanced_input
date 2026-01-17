/*!
Demonstrates automatic context activation based on [`bevy_state`](bevy::state).

This example shows how [`ActiveInState`] eliminates manual context switching.
Compare with the `context_switch` example which requires manual [`ContextActivity`] management.

In this example:
- [`Playing`] state activates [`Player`] context (WASD to move, Space to attack, Escape to pause)
- [`Paused`] state activates [`PauseMenu`] context (Escape to resume)

State transitions automatically toggle the correct contexts.

Run with: `cargo run --example state_integration --features bevy_state`
*/

use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, EnhancedInputPlugin))
        .init_state::<GameState>()
        .add_input_context::<Player>()
        .add_input_context::<PauseMenu>()
        .sync_context_to_state::<GameState, Player>()
        .sync_context_to_state::<GameState, PauseMenu>()
        .add_observer(apply_movement)
        .add_observer(attack)
        .add_observer(pause_game)
        .add_observer(resume_game)
        .add_systems(Startup, spawn)
        .run();
}

fn spawn(mut commands: Commands) {
    commands.spawn((
        Player,
        ActiveInState::<GameState, Player>::new(GameState::Playing),
        actions!(Player[
            (
                Action::<Movement>::new(),
                DeadZone::default(),
                Bindings::spawn((Cardinal::wasd_keys(), Axial::left_stick())),
            ),
            (Action::<Attack>::new(), bindings![KeyCode::Space, GamepadButton::South]),
            (
                Action::<Pause>::new(),
                ActionSettings {
                    require_reset: true,
                    ..Default::default()
                },
                bindings![KeyCode::Escape, GamepadButton::Start],
            ),
        ]),
    ));

    commands.spawn((
        PauseMenu,
        ActiveInState::<GameState, PauseMenu>::new(GameState::Paused),
        actions!(
            PauseMenu[(
                Action::<Resume>::new(),
                ActionSettings {
                    require_reset: true,
                    ..Default::default()
                },
                bindings![KeyCode::Escape, GamepadButton::Start],
            )]
        ),
    ));
}

fn apply_movement(movement: On<Fire<Movement>>) {
    info!("moving: {}", movement.value);
}

fn attack(_trigger: On<Fire<Attack>>) {
    info!("attacking");
}

fn pause_game(_trigger: On<Start<Pause>>, mut next_state: ResMut<NextState<GameState>>) {
    info!("pausing game");
    next_state.set(GameState::Paused);
}

fn resume_game(_trigger: On<Start<Resume>>, mut next_state: ResMut<NextState<GameState>>) {
    info!("resuming game");
    next_state.set(GameState::Playing);
}

#[derive(States, Clone, PartialEq, Eq, Hash, Debug, Default)]
enum GameState {
    #[default]
    Playing,
    Paused,
}

#[derive(Component)]
struct Player;

#[derive(Component)]
struct PauseMenu;

#[derive(InputAction)]
#[action_output(Vec2)]
struct Movement;

#[derive(InputAction)]
#[action_output(bool)]
struct Attack;

#[derive(InputAction)]
#[action_output(bool)]
struct Pause;

#[derive(InputAction)]
#[action_output(bool)]
struct Resume;
