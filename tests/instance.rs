use bevy::{ecs::entity_disabling::Disabled, input::InputPlugin, prelude::*};
use bevy_enhanced_input::prelude::{Release, *};
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
            actions!(TestContext[(Action::<Test>::new(), bindings![Test::KEY1])]),
        ))
        .id();

    app.update();

    app.world_mut()
        .entity_mut(context)
        .remove_with_requires::<TestContext>();

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(Test::KEY1);

    app.world_mut().add_observer(|_: On<Fire<Test>>| {
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
            bindings![Test::KEY1],
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
            actions!(
                TestContext[(
                    Action::<Test>::new(),
                    Release::default(),
                    Scale::splat(2.0),
                    bindings![Test::KEY1]
                )]
            ),
        ))
        .id();

    let with_disabled_action = app
        .world_mut()
        .spawn((
            TestContext,
            actions!(
                TestContext[(
                    Action::<Test>::new(),
                    Disabled,
                    // Add at least one condition and modifier to ensure
                    // they register and unregister properly on a disabled entity.
                    Release::default(),
                    Scale::splat(2.0),
                    bindings![Test::KEY1]
                )]
            ),
        ))
        .id();

    app.update();

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(Test::KEY1);

    app.world_mut().add_observer(|_: On<Fire<Test>>| {
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
            actions!(
                TestContext[(
                    Action::<Test>::new(),
                    Release::default(),
                    Scale::splat(2.0),
                    bindings![Test::KEY1]
                )]
            ),
        ))
        .id();

    app.update();

    // Re-enable the context entity.
    app.world_mut().entity_mut(context).remove::<Disabled>();

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(Test::KEY1);

    app.update();

    let mut actions = app.world_mut().query::<(&Action<Test>, &ActionState)>();

    let (&action, &state) = actions.single(app.world()).unwrap();
    assert_eq!(*action, 2.0, "scale should work after re-enabling");
    assert_eq!(
        state,
        ActionState::Ongoing,
        "release should work after re-enabling"
    );
}

#[test]
fn same_action_different_bindings() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, InputPlugin, EnhancedInputPlugin))
        .add_input_context::<TestContext>()
        .finish();

    app.world_mut().spawn((
        TestContext,
        actions!(TestContext[
            (Action::<Test>::new(), bindings![Test::KEY1]),
            (Action::<Test>::new(), bindings![Test::KEY2]),
        ]),
    ));

    app.update();

    let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
    keys.press(Test::KEY1);
    keys.press(Test::KEY2);

    app.update();

    let mut actions = app.world_mut().query::<&Action<Test>>();

    assert_eq!(actions.iter(app.world()).map(|&a| *a).sum::<f32>(), 2.0);

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .release(Test::KEY1);

    app.update();

    assert_eq!(actions.iter(app.world()).map(|&a| *a).sum::<f32>(), 1.0);

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .release(Test::KEY2);

    app.update();

    assert_eq!(actions.iter(app.world()).map(|&a| *a).sum::<f32>(), 0.0);
}

#[derive(Component)]
struct TestContext;

#[derive(InputAction)]
#[action_output(f32)]
struct Test;

impl Test {
    const KEY1: KeyCode = KeyCode::KeyA;
    const KEY2: KeyCode = KeyCode::KeyB;
}
