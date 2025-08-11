//! Demonstrates all available input conditions.
//! Press keys from the number row on the keyboard to trigger actions and observe the output in console.

use bevy::{ecs::spawn::SpawnWith, log::LogPlugin, prelude::*};
use bevy_enhanced_input::prelude::*;

fn main() {
    // Setup logging to display triggered events.
    let mut log_plugin = LogPlugin::default();
    log_plugin.filter += ",bevy_enhanced_input=debug";

    App::new()
        .add_plugins((
            DefaultPlugins.set(log_plugin),
            EnhancedInputPlugin,
            GamePlugin,
        ))
        .run();
}

struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_input_context::<TestContext>()
            .add_systems(Startup, spawn);
    }
}

fn spawn(mut commands: Commands) {
    commands.spawn((
        TestContext,
        Actions::<TestContext>::spawn(SpawnWith(|context: &mut ActionSpawner<_>| {
            context.spawn((
                Action::<TestDown>::new(),
                Down::default(),
                bindings![TestDown::KEY],
            ));
            context.spawn((
                Action::<TestPress>::new(),
                Press::default(),
                bindings![TestPress::KEY],
            ));
            context.spawn((
                Action::<TestHold>::new(),
                Hold::new(1.0),
                bindings![TestHold::KEY],
            ));
            context.spawn((
                Action::<TestHoldAndRelease>::new(),
                HoldAndRelease::new(1.0),
                bindings![TestHoldAndRelease::KEY],
            ));
            context.spawn((
                Action::<TestPulse>::new(),
                Pulse::new(1.0),
                bindings![TestPulse::KEY],
            ));
            context.spawn((
                Action::<TestRelease>::new(),
                Release::default(),
                bindings![TestRelease::KEY],
            ));
            context.spawn((
                Action::<TestTap>::new(),
                Tap::new(0.5),
                bindings![TestTap::KEY],
            ));

            let member1 = context
                .spawn((Action::<ChordMember1>::new(), bindings![ChordMember1::KEY]))
                .id();
            let member2 = context
                .spawn((Action::<ChordMember2>::new(), bindings![ChordMember2::KEY]))
                .id();

            context.spawn((Action::<TestChord>::new(), Chord::new([member1, member2])));

            let blocker = context
                .spawn((Action::<Blocker>::new(), bindings![Blocker::KEY]))
                .id();
            context.spawn((
                Action::<TestBlockBy>::new(),
                BlockBy::single(blocker),
                bindings![TestBlockBy::KEY],
            ));
        })),
    ));
}

#[derive(Component)]
struct TestContext;

#[derive(InputAction)]
#[action_output(bool)]
struct TestDown;

impl TestDown {
    const KEY: KeyCode = KeyCode::Digit1;
}

#[derive(InputAction)]
#[action_output(bool)]
struct TestPress;

impl TestPress {
    const KEY: KeyCode = KeyCode::Digit2;
}

#[derive(InputAction)]
#[action_output(bool)]
struct TestHold;

impl TestHold {
    const KEY: KeyCode = KeyCode::Digit3;
}

#[derive(InputAction)]
#[action_output(bool)]
struct TestHoldAndRelease;

impl TestHoldAndRelease {
    const KEY: KeyCode = KeyCode::Digit4;
}

#[derive(InputAction)]
#[action_output(bool)]
struct TestPulse;

impl TestPulse {
    const KEY: KeyCode = KeyCode::Digit5;
}

#[derive(InputAction)]
#[action_output(bool)]
struct TestRelease;

impl TestRelease {
    const KEY: KeyCode = KeyCode::Digit6;
}

#[derive(InputAction)]
#[action_output(bool)]
struct TestTap;

impl TestTap {
    const KEY: KeyCode = KeyCode::Digit7;
}

#[derive(InputAction)]
#[action_output(bool)]
struct ChordMember1;

impl ChordMember1 {
    const KEY: KeyCode = KeyCode::Digit8;
}

#[derive(InputAction)]
#[action_output(bool)]
struct ChordMember2;

impl ChordMember2 {
    const KEY: KeyCode = KeyCode::Digit9;
}

#[derive(InputAction)]
#[action_output(bool)]
struct TestChord;

#[derive(InputAction)]
#[action_output(bool)]
struct Blocker;

impl Blocker {
    const KEY: KeyCode = KeyCode::Digit0;
}

#[derive(InputAction)]
#[action_output(bool)]
struct TestBlockBy;

impl TestBlockBy {
    const KEY: KeyCode = KeyCode::Minus;
}
