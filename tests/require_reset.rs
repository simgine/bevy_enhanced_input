use bevy::{input::InputPlugin, prelude::*};
use bevy_enhanced_input::prelude::*;
use test_log::test;

#[test]
fn layering() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, InputPlugin, EnhancedInputPlugin))
        .add_input_context::<First>()
        .add_input_context::<Second>()
        .finish();

    let context = app
        .world_mut()
        .spawn((
            Second,
            actions!(Second[(Action::<OnSecond>::new(), bindings![KEY])]),
        ))
        .id();

    app.update();

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KEY);

    app.update();

    let mut second_actions = app
        .world_mut()
        .query_filtered::<&ActionState, With<Action<OnSecond>>>();

    let second_state = *second_actions.single(app.world()).unwrap();
    assert_eq!(second_state, ActionState::Fired);

    app.world_mut().entity_mut(context).insert((
        First,
        ContextPriority::<First>::new(1),
        actions!(
            First[(
                Action::<OnFirst>::new(),
                ActionSettings::default().require_reset(true),
                bindings![KEY]
            )]
        ),
    ));

    app.update();

    let mut first_actions = app
        .world_mut()
        .query_filtered::<&ActionState, With<Action<OnFirst>>>();

    let first_state = *first_actions.single(app.world()).unwrap();
    assert_eq!(
        first_state,
        ActionState::None,
        "shouldn't fire because the input should stop actuating first"
    );

    let second_state = *second_actions.single(app.world()).unwrap();
    assert_eq!(
        second_state,
        ActionState::None,
        "shouldn't fire because consumed by the first action"
    );

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .release(KEY);

    app.update();

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KEY);

    app.update();

    let first_state = *first_actions.single(app.world()).unwrap();
    assert_eq!(first_state, ActionState::Fired);

    let second_state = *second_actions.single(app.world()).unwrap();
    assert_eq!(second_state, ActionState::None);
}

#[test]
fn switching() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, InputPlugin, EnhancedInputPlugin))
        .add_input_context::<First>()
        .add_input_context::<Second>()
        .finish();

    let context = app
        .world_mut()
        .spawn((
            First,
            ContextPriority::<First>::new(1),
            actions!(
                First[(
                    Action::<OnFirst>::new(),
                    ActionSettings::default().require_reset(true),
                    bindings![KEY]
                )]
            ),
        ))
        .id();

    app.update();

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KEY);

    app.update();

    let mut actions = app.world_mut().query::<&ActionState>();

    let first_state = *actions.single(app.world()).unwrap();
    assert_eq!(first_state, ActionState::Fired);

    app.world_mut()
        .entity_mut(context)
        .remove_with_requires::<First>()
        .despawn_related::<Actions<First>>()
        .insert((
            Second,
            actions!(Second[(Action::<OnSecond>::new(), bindings![KEY])]),
        ));

    app.update();

    let second_state = *actions.single(app.world()).unwrap();
    assert_eq!(
        second_state,
        ActionState::None,
        "action should still be consumed even after removal"
    );

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .release(KEY);

    app.update();

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KEY);

    app.update();

    let second_state = *actions.single(app.world()).unwrap();
    assert_eq!(second_state, ActionState::Fired);
}

#[derive(Component)]
struct First;

#[derive(Component)]
struct Second;

/// A key used by all actions.
const KEY: KeyCode = KeyCode::KeyA;

#[derive(InputAction)]
#[action_output(bool)]
struct OnFirst;

#[derive(InputAction)]
#[action_output(bool)]
struct OnSecond;
