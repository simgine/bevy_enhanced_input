use bevy::{ecs::entity_disabling::Disabled, input::InputPlugin, prelude::*};
use bevy_enhanced_input::prelude::*;
use test_log::test;

#[test]
fn removal() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, InputPlugin, EnhancedInputPlugin))
        .add_input_context::<TestContext>()
        .finish();

    let context = app
        .world_mut()
        .spawn((
            TestContext,
            actions!(TestContext[(Action::<Test>::new(), bindings![Test::KEY])]),
        ))
        .id();

    app.update();

    app.world_mut()
        .entity_mut(context)
        .remove_with_requires::<TestContext>();

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(Test::KEY);

    app.world_mut().add_observer(|_: Trigger<Fired<Test>>| {
        panic!("action shouldn't trigger");
    });

    app.update();
}

#[test]
fn invalid_hierarchy() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, InputPlugin, EnhancedInputPlugin))
        .add_input_context::<TestContext>()
        .finish();

    app.world_mut().spawn((
        TestContext,
        actions!(TestContext[
            (
                // Action without bindings.
                Action::<Test>::new(),
                Bindings::spawn((Spawn(Down::default()), Spawn(Scale::splat(1.0))))
            ),
            // Bindings without action.
            bindings![Test::KEY],
        ]),
    ));

    app.update();
}

#[test]
fn disabled() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, InputPlugin, EnhancedInputPlugin))
        .add_input_context::<TestContext>()
        .finish();

    let disabled = app
        .world_mut()
        .spawn((
            TestContext,
            Disabled,
            actions!(TestContext[(Action::<Test>::new(), bindings![Test::KEY])]),
        ))
        .id();

    let with_disabled_action = app
        .world_mut()
        .spawn((
            TestContext,
            actions!(TestContext[(Action::<Test>::new(), Disabled, bindings![Test::KEY])]),
        ))
        .id();

    app.update();

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(Test::KEY);

    app.world_mut().add_observer(|_: Trigger<Fired<Test>>| {
        panic!("action shouldn't trigger");
    });

    app.update();

    app.world_mut().despawn(disabled);
    app.world_mut().despawn(with_disabled_action);
}

#[test]
fn reenabling() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, InputPlugin, EnhancedInputPlugin))
        .add_input_context::<TestContext>()
        .finish();

    let context = app
        .world_mut()
        .spawn((
            TestContext,
            Disabled,
            actions!(TestContext[(Action::<Test>::new(), bindings![Test::KEY])]),
        ))
        .id();

    app.update();

    // Re-enable the context entity.
    app.world_mut().entity_mut(context).remove::<Disabled>();

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(Test::KEY);

    app.update();

    let mut actions = app
        .world_mut()
        .query_filtered::<&ActionState, With<Action<Test>>>();

    let state = *actions.single(app.world()).unwrap();
    assert_eq!(state, ActionState::Fired);
}

#[derive(Component)]
struct TestContext;

#[derive(InputAction)]
#[action_output(bool)]
struct Test;

impl Test {
    const KEY: KeyCode = KeyCode::KeyA;
}
