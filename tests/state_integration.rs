#![cfg(feature = "bevy_state")]

use bevy::{input::InputPlugin, prelude::*, state::app::StatesPlugin};
use bevy_enhanced_input::prelude::*;
use test_log::test;

#[test]
fn initial_state_activates_context() {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        InputPlugin,
        StatesPlugin,
        EnhancedInputPlugin,
    ))
    .init_state::<TestState>()
    .add_input_context::<ContextA>()
    .sync_context_to_state::<TestState, ContextA>()
    .finish();

    app.world_mut().spawn((
        ContextA,
        ActiveInState::<TestState, ContextA>::new(TestState::A),
        actions!(ContextA[(Action::<TestAction>::new(), bindings![KeyCode::KeyA])]),
    ));

    app.update();

    let activity = app
        .world_mut()
        .query_filtered::<&ContextActivity<ContextA>, With<ContextA>>()
        .single(app.world())
        .unwrap();

    assert!(**activity, "context should be active in initial state");
}

#[test]
fn state_transition_activates_matching_context() {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        InputPlugin,
        StatesPlugin,
        EnhancedInputPlugin,
    ))
    .init_state::<TestState>()
    .add_input_context::<ContextA>()
    .add_input_context::<ContextB>()
    .sync_context_to_state::<TestState, ContextA>()
    .sync_context_to_state::<TestState, ContextB>()
    .finish();

    app.world_mut().spawn((
        ContextA,
        ActiveInState::<TestState, ContextA>::new(TestState::A),
        actions!(ContextA[(Action::<TestAction>::new(), bindings![KeyCode::KeyA])]),
    ));

    app.world_mut().spawn((
        ContextB,
        ActiveInState::<TestState, ContextB>::new(TestState::B),
        actions!(ContextB[(Action::<TestAction>::new(), bindings![KeyCode::KeyB])]),
    ));

    app.update();

    app.world_mut()
        .resource_mut::<NextState<TestState>>()
        .set(TestState::B);

    app.update();

    let activity_a = *app
        .world_mut()
        .query_filtered::<&ContextActivity<ContextA>, With<ContextA>>()
        .single(app.world())
        .unwrap();

    let activity_b = *app
        .world_mut()
        .query_filtered::<&ContextActivity<ContextB>, With<ContextB>>()
        .single(app.world())
        .unwrap();

    assert!(
        !*activity_a,
        "context A should be inactive after transition to B"
    );
    assert!(
        *activity_b,
        "context B should be active after transition to B"
    );
}

#[test]
fn active_in_states_matches_multiple() {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        InputPlugin,
        StatesPlugin,
        EnhancedInputPlugin,
    ))
    .init_state::<TestState>()
    .add_input_context::<ContextA>()
    .sync_context_to_state::<TestState, ContextA>()
    .finish();

    app.world_mut().spawn((
        ContextA,
        ActiveInStates::<TestState, ContextA>::new([TestState::A, TestState::B]),
        actions!(ContextA[(Action::<TestAction>::new(), bindings![KeyCode::KeyA])]),
    ));

    app.update();

    let get_activity = |app: &mut App| {
        **app
            .world_mut()
            .query_filtered::<&ContextActivity<ContextA>, With<ContextA>>()
            .single(app.world())
            .unwrap()
    };

    assert!(get_activity(&mut app), "should be active in state A");

    app.world_mut()
        .resource_mut::<NextState<TestState>>()
        .set(TestState::B);
    app.update();

    assert!(get_activity(&mut app), "should be active in state B");

    app.world_mut()
        .resource_mut::<NextState<TestState>>()
        .set(TestState::C);
    app.update();

    assert!(!get_activity(&mut app), "should be inactive in state C");
}

#[derive(States, Clone, PartialEq, Eq, Hash, Debug, Default)]
enum TestState {
    #[default]
    A,
    B,
    C,
}

#[derive(Component)]
struct ContextA;

#[derive(Component)]
struct ContextB;

#[derive(InputAction)]
#[action_output(bool)]
struct TestAction;
