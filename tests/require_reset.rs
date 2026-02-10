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

    let contexts = app
        .world_mut()
        .spawn((
            First,
            actions!(First[(Action::<OnFirst>::new(), bindings![KEY])]),
        ))
        .id();

    app.update();

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KEY);

    app.update();

    let mut first_actions = app.world_mut().query::<&Action<OnFirst>>();

    let on_first = *first_actions.single(app.world()).unwrap();
    assert!(*on_first);

    app.world_mut().entity_mut(contexts).insert((
        Second,
        ContextPriority::<Second>::new(1),
        actions!(
            Second[(
                Action::<OnSecond>::new(),
                ActionSettings {
                    consume_input: true,
                    require_reset: true,
                    ..Default::default()
                },
                bindings![KEY]
            )]
        ),
    ));

    app.update();

    let on_first = *first_actions.single(app.world()).unwrap();
    assert!(
        !*on_first,
        "shouldn't fire because consumed by the second action"
    );

    let mut second_actions = app.world_mut().query::<&Action<OnSecond>>();

    let on_second = *second_actions.single(app.world()).unwrap();
    assert!(
        !*on_second,
        "shouldn't fire because the input should stop actuating first"
    );

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .release(KEY);

    app.update();

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KEY);

    app.update();

    let on_fist = *first_actions.single(app.world()).unwrap();
    assert!(!*on_fist);

    let on_second = *second_actions.single(app.world()).unwrap();
    assert!(*on_second);
}

#[test]
fn switching_by_removal() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, InputPlugin, EnhancedInputPlugin))
        .add_input_context::<First>()
        .add_input_context::<Second>()
        .finish();

    let context = app
        .world_mut()
        .spawn((
            First,
            actions!(
                First[(
                    Action::<OnFirst>::new(),
                    ActionSettings {
                        require_reset: true,
                        ..Default::default()
                    },
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

    let mut actions = app.world_mut().query::<&TriggerState>();

    let first_state = *actions.single(app.world()).unwrap();
    assert_eq!(first_state, TriggerState::Fired);

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
        TriggerState::None,
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
    assert_eq!(second_state, TriggerState::Fired);
}

#[test]
fn switching_by_activation() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, InputPlugin, EnhancedInputPlugin))
        .add_input_context::<First>()
        .add_input_context::<Second>()
        .finish();

    let contexts = app
        .world_mut()
        .spawn((
            First,
            actions!(First[(Action::<OnFirst>::new(), bindings![KEY])]),
            Second,
            ContextActivity::<Second>::INACTIVE,
            actions!(
                Second[(
                    Action::<OnSecond>::new(),
                    ActionSettings {
                        require_reset: true,
                        ..Default::default()
                    },
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

    let mut first_actions = app.world_mut().query::<&Action<OnFirst>>();

    let on_first = *first_actions.single(app.world()).unwrap();
    assert!(*on_first);

    app.world_mut().entity_mut(contexts).insert((
        ContextActivity::<First>::INACTIVE,
        ContextActivity::<Second>::ACTIVE,
    ));

    app.update();

    let on_first = *first_actions.single(app.world()).unwrap();
    assert!(
        !*on_first,
        "shouldn't fire because consumed by the second action"
    );

    let mut second_actions = app.world_mut().query::<&Action<OnSecond>>();

    let on_second = *second_actions.single(app.world()).unwrap();
    assert!(
        !*on_second,
        "shouldn't fire because the input should stop actuating first"
    );

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .release(KEY);

    app.update();

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KEY);

    app.update();

    let on_first = *first_actions.single(app.world()).unwrap();
    assert!(!*on_first);

    let on_second = *second_actions.single(app.world()).unwrap();
    assert!(*on_second);
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
