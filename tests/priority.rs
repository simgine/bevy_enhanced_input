use bevy::{input::InputPlugin, prelude::*, time::TimeUpdateStrategy};
use bevy_enhanced_input::prelude::*;
use test_log::test;

#[test]
fn same_schedule() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, InputPlugin, EnhancedInputPlugin))
        .add_input_context::<First>()
        .add_input_context::<Second>()
        .finish();

    app.world_mut().spawn((
        First,
        ContextPriority::<First>::new(1),
        actions!(First[
            (
                Action::<FirstConsume>::new(),
                ActionSettings { consume_input: true, ..Default::default() },
                bindings![CONSUME_KEY]
            ),
            (
                Action::<FirstPassthrough>::new(),
                ActionSettings { consume_input: false, ..Default::default() },
                bindings![PASSTHROUGH_KEY]
            )
        ]),
        Second,
        actions!(Second[
            (
                Action::<SecondConsume>::new(),
                ActionSettings { consume_input: true, ..Default::default() },
                bindings![CONSUME_KEY]
            ),
            (
                Action::<SecondPassthrough>::new(),
                ActionSettings { consume_input: false, ..Default::default() },
                bindings![PASSTHROUGH_KEY]
            )
        ]),
    ));

    app.update();

    let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
    keys.press(CONSUME_KEY);
    keys.press(PASSTHROUGH_KEY);

    app.update();

    let mut first_consume = app
        .world_mut()
        .query_filtered::<&TriggerState, With<Action<FirstConsume>>>();

    let first_consume_state = *first_consume.single(app.world()).unwrap();
    assert_eq!(first_consume_state, TriggerState::Fired);

    let mut first_passthrough = app
        .world_mut()
        .query_filtered::<&TriggerState, With<Action<FirstPassthrough>>>();

    let first_passthrough_state = *first_passthrough.single(app.world()).unwrap();
    assert_eq!(first_passthrough_state, TriggerState::Fired);

    let mut second_consume = app
        .world_mut()
        .query_filtered::<&TriggerState, With<Action<SecondConsume>>>();

    let second_consume_state = *second_consume.single(app.world()).unwrap();
    assert_eq!(
        second_consume_state,
        TriggerState::None,
        "input should be consumed from a context with a higher priority"
    );

    let mut second_passthrough = app
        .world_mut()
        .query_filtered::<&TriggerState, With<Action<SecondPassthrough>>>();

    let second_passthrough_state = *second_passthrough.single(app.world()).unwrap();
    assert_eq!(
        second_passthrough_state,
        TriggerState::Fired,
        "actions that doesn't consume inputs should still be triggered"
    );
}

#[test]
fn different_schedules() {
    let time_step = Time::<Fixed>::default().timestep() / 2;

    let mut app = App::new();
    app.add_plugins((MinimalPlugins, InputPlugin, EnhancedInputPlugin))
        .insert_resource(TimeUpdateStrategy::ManualDuration(time_step))
        .add_input_context::<First>()
        .add_input_context_to::<FixedPreUpdate, Second>()
        .finish();

    app.world_mut().spawn((
        First,
        actions!(First[
            (
                Action::<FirstConsume>::new(),
                ActionSettings { consume_input: true, ..Default::default() },
                bindings![CONSUME_KEY]
            ),
            (
                Action::<FirstPassthrough>::new(),
                ActionSettings { consume_input: false, ..Default::default() },
                bindings![PASSTHROUGH_KEY]
            )
        ]),
        Second,
        ContextPriority::<Second>::new(1),
        actions!(Second[
            (
                Action::<SecondConsume>::new(),
                ActionSettings { consume_input: true, ..Default::default() },
                bindings![CONSUME_KEY]
            ),
            (
                Action::<SecondPassthrough>::new(),
                ActionSettings { consume_input: false, ..Default::default() },
                bindings![PASSTHROUGH_KEY]
            )
        ]),
    ));

    let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
    keys.press(CONSUME_KEY);
    keys.press(PASSTHROUGH_KEY);

    let mut first_consume = app
        .world_mut()
        .query_filtered::<&TriggerState, With<Action<FirstConsume>>>();
    let mut first_passthrough = app
        .world_mut()
        .query_filtered::<&TriggerState, With<Action<FirstPassthrough>>>();
    let mut second_consume = app
        .world_mut()
        .query_filtered::<&TriggerState, With<Action<SecondConsume>>>();
    let mut second_passthrough = app
        .world_mut()
        .query_filtered::<&TriggerState, With<Action<SecondPassthrough>>>();

    for frame in 0..2 {
        app.update();

        let first_consume_state = *first_consume.single(app.world()).unwrap();
        assert_eq!(first_consume_state, TriggerState::Fired);

        let first_passthrough_state = *first_passthrough.single(app.world()).unwrap();
        assert_eq!(first_passthrough_state, TriggerState::Fired);

        let second_consume_state = *second_consume.single(app.world()).unwrap();
        assert_eq!(second_consume_state, TriggerState::None);

        let second_passthrough_state = *second_passthrough.single(app.world()).unwrap();
        assert_eq!(
            second_passthrough_state,
            TriggerState::None,
            "shouldn't fire on frame {frame} because the schedule hasn't run yet"
        );
    }

    for frame in 2..4 {
        app.update();

        let first_consume_state = *first_consume.single(app.world()).unwrap();
        assert_eq!(first_consume_state, TriggerState::Fired);

        let first_passthrough_state = *first_passthrough.single(app.world()).unwrap();
        assert_eq!(first_passthrough_state, TriggerState::Fired);

        let second_consume_state = *second_consume.single(app.world()).unwrap();
        assert_eq!(
            second_consume_state,
            TriggerState::None,
            "shouldn't fire on frame {frame} because of the schedule evaluation order"
        );

        let second_passthrough_state = *second_passthrough.single(app.world()).unwrap();
        assert_eq!(second_passthrough_state, TriggerState::Fired);
    }
}

#[test]
fn change() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, InputPlugin, EnhancedInputPlugin))
        .add_input_context::<First>()
        .add_input_context::<Second>()
        .finish();

    let contexts = app
        .world_mut()
        .spawn((
            First,
            ContextPriority::<First>::new(1),
            actions!(First[
                (
                    Action::<FirstConsume>::new(),
                    ActionSettings { consume_input: true, ..Default::default() },
                    bindings![CONSUME_KEY]
                ),
                (
                    Action::<FirstPassthrough>::new(),
                    ActionSettings { consume_input: false, ..Default::default() },
                    bindings![PASSTHROUGH_KEY]
                )
            ]),
            Second,
            actions!(Second[
                (
                    Action::<SecondConsume>::new(),
                    ActionSettings { consume_input: true, ..Default::default() },
                    bindings![CONSUME_KEY]
                ),
                (
                    Action::<SecondPassthrough>::new(),
                    ActionSettings { consume_input: false, ..Default::default() },
                    bindings![PASSTHROUGH_KEY]
                )
            ]),
        ))
        .id();

    app.update();

    app.world_mut()
        .entity_mut(contexts)
        .insert(ContextPriority::<Second>::new(2));

    let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
    keys.press(CONSUME_KEY);
    keys.press(PASSTHROUGH_KEY);

    app.update();

    let mut first_consume = app
        .world_mut()
        .query_filtered::<&TriggerState, With<Action<FirstConsume>>>();

    let first_consume_state = *first_consume.single(app.world()).unwrap();
    assert_eq!(first_consume_state, TriggerState::None);

    let mut first_passthrough = app
        .world_mut()
        .query_filtered::<&TriggerState, With<Action<FirstPassthrough>>>();

    let first_passthrough_state = *first_passthrough.single(app.world()).unwrap();
    assert_eq!(first_passthrough_state, TriggerState::Fired);

    let mut second_consume = app
        .world_mut()
        .query_filtered::<&TriggerState, With<Action<SecondConsume>>>();

    let second_consume_state = *second_consume.single(app.world()).unwrap();
    assert_eq!(second_consume_state, TriggerState::Fired);

    let mut second_passthrough = app
        .world_mut()
        .query_filtered::<&TriggerState, With<Action<SecondPassthrough>>>();

    let second_passthrough_state = *second_passthrough.single(app.world()).unwrap();
    assert_eq!(second_passthrough_state, TriggerState::Fired);
}

#[derive(Component, TypePath)]
struct First;

#[derive(Component, TypePath)]
struct Second;

/// A key used by all consume actions.
const CONSUME_KEY: KeyCode = KeyCode::KeyA;

/// A key used by all consume actions.
const PASSTHROUGH_KEY: KeyCode = KeyCode::KeyB;

#[derive(InputAction)]
#[action_output(bool)]
struct FirstConsume;

#[derive(InputAction)]
#[action_output(bool)]
struct FirstPassthrough;

#[derive(InputAction)]
#[action_output(bool)]
struct SecondConsume;

#[derive(InputAction)]
#[action_output(bool)]
struct SecondPassthrough;
