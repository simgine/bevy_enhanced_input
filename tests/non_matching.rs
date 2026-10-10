use bevy::{input::InputPlugin, prelude::*};
use bevy_enhanced_input::prelude::*;
use test_log::test;

#[test]
fn action() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, InputPlugin, EnhancedInputPlugin))
        .add_input_context::<TestContext>()
        .finish();

    app.world_mut().spawn((
        TestContext,
        actions!(TestContext[
            Name::new("Not an action"),
            (Action::<Test>::new(), bindings![KEY]),
        ]),
    ));

    app.update();

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KEY);

    app.update();

    let mut actions = app.world_mut().query::<&Action<Test>>();

    let action = *actions.single(app.world()).unwrap();
    assert!(
        *action,
        "should fire even if an earlier entity of the context is not an action"
    );
}

#[test]
fn binding() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, InputPlugin, EnhancedInputPlugin))
        .add_input_context::<TestContext>()
        .finish();

    app.world_mut().spawn((
        TestContext,
        actions!(
            TestContext[(
                Action::<Test>::new(),
                Bindings::spawn((Spawn(Name::new("Not a binding")), Spawn(Binding::from(KEY)))),
            )]
        ),
    ));

    app.update();

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KEY);

    app.update();

    let mut actions = app.world_mut().query::<&Action<Test>>();

    let action = *actions.single(app.world()).unwrap();
    assert!(
        *action,
        "should fire even if an earlier entity of the action is not a binding"
    );
}

#[derive(Component)]
struct TestContext;

/// A key used by all actions.
const KEY: KeyCode = KeyCode::KeyA;

#[derive(InputAction)]
#[action_output(bool)]
struct Test;
