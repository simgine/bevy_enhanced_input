use bevy::{input::InputPlugin, prelude::*};
use bevy_enhanced_input::prelude::*;
use test_log::test;

#[test]
fn max_abs() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, InputPlugin, EnhancedInputPlugin))
        .add_input_context::<TestContext>()
        .finish();

    app.world_mut().spawn((
        TestContext,
        actions!(
            TestContext[(
                Action::<Test>::new(),
                ActionSettings {
                    accumulation: Accumulation::MaxAbs,
                    ..Default::default()
                },
                Bindings::spawn(Cardinal::wasd_keys())
            )]
        ),
    ));

    app.update();

    let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
    keys.press(KeyCode::KeyW);
    keys.press(KeyCode::KeyS);

    app.update();

    let mut actions = app.world_mut().query::<&Action<Test>>();
    let action = *actions.single(app.world()).unwrap();
    assert_eq!(*action, Vec2::Y);
}

#[test]
fn cumulative() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, InputPlugin, EnhancedInputPlugin))
        .add_input_context::<TestContext>()
        .finish();

    app.world_mut().spawn((
        TestContext,
        actions!(
            TestContext[(
                Action::<Test>::new(),
                ActionSettings {
                    accumulation: Accumulation::Cumulative,
                    ..Default::default()
                },
                Bindings::spawn(Cardinal::wasd_keys())
            )]
        ),
    ));

    app.update();

    let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
    keys.press(KeyCode::ArrowUp);
    keys.press(KeyCode::ArrowDown);

    app.update();

    let mut actions = app.world_mut().query::<&Action<Test>>();
    let action = *actions.single(app.world()).unwrap();
    assert_eq!(*action, Vec2::ZERO, "up and down should cancel each other");
}

#[derive(Component, TypePath)]
struct TestContext;

#[derive(InputAction)]
#[action_output(Vec2)]
struct Test;
