#![cfg(feature = "state")]

use bevy::{input::InputPlugin, prelude::*, state::app::StatesPlugin};
use bevy_enhanced_input::prelude::*;
use test_log::test;

#[test]
fn transition() {
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
    .sync_context_to_state::<ContextA, TestState>()
    .sync_context_to_state::<ContextB, TestState>()
    .finish();

    app.world_mut().spawn((
        ContextA,
        ContextActivity::<ContextA>::INACTIVE,
        ActiveInStates::<ContextA, _>::single(TestState::A),
        actions!(ContextA[(Action::<TestAction>::new(), bindings![KeyCode::KeyA])]),
    ));

    app.world_mut().spawn((
        ContextB,
        ContextActivity::<ContextB>::INACTIVE,
        ActiveInStates::<ContextB, _>::single(TestState::B),
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
fn multiple_states() {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        InputPlugin,
        StatesPlugin,
        EnhancedInputPlugin,
    ))
    .init_state::<TestState>()
    .add_input_context::<ContextA>()
    .sync_context_to_state::<ContextA, TestState>()
    .finish();

    app.world_mut().spawn((
        ContextA,
        ContextActivity::<ContextA>::INACTIVE,
        ActiveInStates::<ContextA, _>::new([TestState::A, TestState::B]),
        actions!(ContextA[(Action::<TestAction>::new(), bindings![KeyCode::KeyA])]),
    ));

    app.update();

    let mut activities = app
        .world_mut()
        .query_filtered::<&ContextActivity<ContextA>, With<ContextA>>();

    assert!(
        **activities.single(app.world()).unwrap(),
        "should be active in state A"
    );

    app.world_mut()
        .resource_mut::<NextState<TestState>>()
        .set(TestState::B);
    app.update();

    assert!(
        **activities.single(app.world()).unwrap(),
        "should be active in state B"
    );

    app.world_mut()
        .resource_mut::<NextState<TestState>>()
        .set(TestState::C);
    app.update();

    assert!(
        !**activities.single(app.world()).unwrap(),
        "should be inactive in state C"
    );
}

#[test]
fn on_spawn() {
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
    .sync_context_to_state::<ContextA, TestState>()
    .sync_context_to_state::<ContextB, TestState>()
    .finish();

    app.world_mut()
        .resource_mut::<NextState<TestState>>()
        .set(TestState::B);
    app.update();

    app.world_mut().spawn((
        ContextA,
        ContextActivity::<ContextA>::INACTIVE,
        ActiveInStates::<ContextA, _>::single(TestState::A),
        actions!(ContextA[(Action::<TestAction>::new(), bindings![KeyCode::KeyA])]),
    ));

    app.world_mut().spawn((
        ContextB,
        ContextActivity::<ContextB>::INACTIVE,
        ActiveInStates::<ContextB, _>::single(TestState::B),
        actions!(ContextB[(Action::<TestAction>::new(), bindings![KeyCode::KeyB])]),
    ));

    app.update();

    let activity_a = **app
        .world_mut()
        .query_filtered::<&ContextActivity<ContextA>, With<ContextA>>()
        .single(app.world())
        .unwrap();

    let activity_b = **app
        .world_mut()
        .query_filtered::<&ContextActivity<ContextB>, With<ContextB>>()
        .single(app.world())
        .unwrap();

    assert!(
        !activity_a,
        "context A should be inactive when spawned in state B"
    );
    assert!(
        activity_b,
        "context B should be active when spawned in state B"
    );
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
