use bevy::{input::InputPlugin, prelude::*};
use bevy_enhanced_input::prelude::*;
use test_log::test;

#[test]
fn consume() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, InputPlugin, EnhancedInputPlugin))
        .add_input_context::<TestContext>()
        .finish();

    app.world_mut().spawn((
        TestContext,
        actions!(
            TestContext[(
                Action::<First>::new(),
                ActionSettings {
                    consume_input: true,
                    ..Default::default()
                },
                bindings![KEY],
            )]
        ),
    ));
    app.world_mut().spawn((
        TestContext,
        actions!(
            TestContext[(
                Action::<Second>::new(),
                ActionSettings {
                    consume_input: true,
                    ..Default::default()
                },
                bindings![KEY],
            )]
        ),
    ));

    app.update();

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KEY);

    app.update();

    let mut first = app
        .world_mut()
        .query_filtered::<&ActionState, With<Action<First>>>();

    let first_state = *first.single(app.world()).unwrap();
    assert_eq!(
        first_state,
        ActionState::None,
        "only last spawned entity with the same bindings that consume inputs should receive them"
    );

    let mut second = app
        .world_mut()
        .query_filtered::<&ActionState, With<Action<Second>>>();

    let second_state = *second.single(app.world()).unwrap();
    assert_eq!(second_state, ActionState::Fired);
}

#[test]
fn passthrough() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, InputPlugin, EnhancedInputPlugin))
        .add_input_context::<TestContext>()
        .finish();

    app.world_mut().spawn((
        TestContext,
        actions!(
            TestContext[(
                Action::<First>::new(),
                ActionSettings {
                    consume_input: false,
                    ..Default::default()
                },
                bindings![KEY],
            )]
        ),
    ));
    app.world_mut().spawn((
        TestContext,
        actions!(
            TestContext[(
                Action::<Second>::new(),
                ActionSettings {
                    consume_input: false,
                    ..Default::default()
                },
                bindings![KEY],
            )]
        ),
    ));

    app.update();

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KEY);

    app.update();

    let mut first = app
        .world_mut()
        .query_filtered::<&ActionState, With<Action<First>>>();

    let first_state = *first.single(app.world()).unwrap();
    assert_eq!(first_state, ActionState::Fired);

    let mut second = app
        .world_mut()
        .query_filtered::<&ActionState, With<Action<Second>>>();

    let second_state = *second.single(app.world()).unwrap();
    assert_eq!(
        second_state,
        ActionState::Fired,
        "actions that doesn't consume inputs should still fire"
    );
}

#[test]
fn consume_then_passthrough() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, InputPlugin, EnhancedInputPlugin))
        .add_input_context::<TestContext>()
        .finish();

    app.world_mut().spawn((
        TestContext,
        actions!(
            TestContext[
            (
                Action::<First>::new(),
                ActionSettings {
                    consume_input: true,
                    ..Default::default()
                },
                bindings![KEY],
            ),
            (
                Action::<Second>::new(),
                ActionSettings {
                    consume_input: false,
                    ..Default::default()
                },
                bindings![KEY],
            ),
            ]
        ),
    ));

    app.update();

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KEY);

    app.update();

    let mut first = app
        .world_mut()
        .query_filtered::<&ActionState, With<Action<First>>>();

    let first_state = *first.single(app.world()).unwrap();
    assert_eq!(first_state, ActionState::Fired);

    let mut second = app
        .world_mut()
        .query_filtered::<&ActionState, With<Action<Second>>>();

    let second_state = *second.single(app.world()).unwrap();
    assert_eq!(second_state, ActionState::None, "action should be consumed");
}

#[test]
fn passthrough_then_consume() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, InputPlugin, EnhancedInputPlugin))
        .add_input_context::<TestContext>()
        .finish();

    app.world_mut().spawn((
        TestContext,
        actions!(
            TestContext[
            (
                Action::<First>::new(),
                ActionSettings {
                    consume_input: false,
                    ..Default::default()
                },
                bindings![KEY],
            ),
            (
                Action::<Second>::new(),
                ActionSettings {
                    consume_input: true,
                    ..Default::default()
                },
                bindings![KEY],
            ),
            ]
        ),
    ));

    app.update();

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KEY);

    app.update();

    let mut first = app
        .world_mut()
        .query_filtered::<&ActionState, With<Action<First>>>();

    let first_state = *first.single(app.world()).unwrap();
    assert_eq!(first_state, ActionState::Fired);

    let mut second = app
        .world_mut()
        .query_filtered::<&ActionState, With<Action<Second>>>();

    let second_state = *second.single(app.world()).unwrap();
    assert_eq!(second_state, ActionState::Fired);
}

#[test]
fn modifiers() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, InputPlugin, EnhancedInputPlugin))
        .add_input_context::<TestContext>()
        .finish();

    app.world_mut().spawn((
        TestContext,
        actions!(TestContext[
            (
                Action::<First>::new(),
                bindings![KEY],
            ),
            (
                Action::<Second>::new(),
                bindings![Binding::Keyboard { key: KEY, mod_keys: MOD }],
            )
        ]),
    ));

    app.update();

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KEY);

    app.update();

    let mut first = app
        .world_mut()
        .query_filtered::<&ActionState, With<Action<First>>>();

    let first_state = *first.single(app.world()).unwrap();
    assert_eq!(first_state, ActionState::Fired);

    let mut second = app
        .world_mut()
        .query_filtered::<&ActionState, With<Action<Second>>>();

    let second_state = *second.single(app.world()).unwrap();
    assert_eq!(second_state, ActionState::None);

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KeyCode::ControlLeft);

    app.update();

    let first_state = *first.single(app.world()).unwrap();
    assert_eq!(first_state, ActionState::None);

    let second_state = *second.single(app.world()).unwrap();
    assert_eq!(second_state, ActionState::Fired);
}

#[derive(Component, Clone, Copy)]
struct TestContext;

/// Keys used by all actions.
const KEY: KeyCode = KeyCode::KeyA;
const MOD: ModKeys = ModKeys::CONTROL;

#[derive(InputAction)]
#[action_output(bool)]
struct First;

#[derive(InputAction)]
#[action_output(bool)]
struct Second;
