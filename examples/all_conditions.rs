//! Demonstrates all available input conditions.
//!
//! Input conditions determine when an action is considered "triggered" based on the input state.
//! Read more about them in the [condition module documentation](bevy_enhanced_input::condition).
//!
//! Press keys from the number row on the keyboard to trigger actions and observe the output in console.

use bevy::{log::LogPlugin, prelude::*};
use bevy_enhanced_input::prelude::*;

fn main() {
    // Setup logging to display triggered events.
    let mut log_plugin = LogPlugin::default();
    log_plugin.filter += ",bevy_enhanced_input=debug";

    App::new()
        .add_plugins((DefaultPlugins.set(log_plugin), EnhancedInputPlugin))
        .add_input_context::<TestContext>()
        .add_systems(Startup, spawn)
        .run();
}

fn spawn(mut commands: Commands) {
    commands.spawn_scene(bsn! {
        TestContext
        Actions<TestContext> [
            Action::<TestDown>::new()
            Down
            Bindings [
                Binding(TestDown::KEY)
            ]
            --
            Action::<TestPress>::new()
            Press
            Bindings [
                Binding(TestPress::KEY)
            ]
            --
            Action::<TestHold>::new()
            ~Hold::new(1.0)
            Bindings [
                Binding(TestHold::KEY)
            ]
            --
            Action::<TestHoldAndRelease>::new()
            ~HoldAndRelease::new(1.0)
            Bindings [
                Binding(TestHoldAndRelease::KEY)
            ]
            --
            Action::<TestPulse>::new()
            ~Pulse::new(1.0)
            Bindings [
                Binding(TestPulse::KEY)
            ]
            --
            Action::<TestRelease>::new()
            Release
            Bindings [
                Binding(TestRelease::KEY)
            ]
            --
            Action::<TestTap>::new()
            ~Tap::new(0.5)
            Bindings [
                Binding(TestTap::KEY)
            ]
            --
            Action::<TestToggle>::new()
            Toggle
            Bindings [
                Binding(TestToggle::KEY)
            ]
            --
            #member1
            Action::<ChordMember1>::new()
            Bindings [
                Binding(ChordMember1::KEY)
            ]
            --
            #member2
            Action::<ChordMember2>::new()
            Bindings [
                Binding(ChordMember2::KEY)
            ]
            --
            Action::<TestChord>::new()
            Chord::new([#member1, #member2])
            --
            #blocker
            Action::<Blocker>::new()
            Bindings [
                Binding(Blocker::KEY)
            ]
            --
            Action::<TestBlockBy>::new()
            BlockBy::single(#blocker)
            Bindings [
                Binding(TestBlockBy::KEY)
            ]
            --
            #combo_step
            Action::<ComboStep>::new()
            Bindings [
                Binding(ComboStep::KEY)
            ]
            --
            Action::<TestCombo>::new()
            Combo::new([#combo_step, #combo_step])
        ]
    });
}

#[derive(Component, Default, Clone)]
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
struct TestToggle;

impl TestToggle {
    const KEY: KeyCode = KeyCode::Backspace;
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

#[derive(InputAction)]
#[action_output(bool)]
struct ComboStep;

impl ComboStep {
    const KEY: KeyCode = KeyCode::Space;
}

#[derive(InputAction)]
#[action_output(bool)]
struct TestCombo;
