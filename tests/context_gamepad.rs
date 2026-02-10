use bevy::{input::InputPlugin, prelude::*};
use bevy_enhanced_input::prelude::*;
use test_log::test;

#[test]
fn any() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, InputPlugin, EnhancedInputPlugin))
        .add_input_context::<TestContext>()
        .finish();

    let gamepad_entity1 = app.world_mut().spawn(Gamepad::default()).id();
    let gamepad_entity2 = app.world_mut().spawn(Gamepad::default()).id();

    app.world_mut().spawn((
        TestContext,
        GamepadDevice::Any,
        actions!(TestContext[(Action::<Test>::new(), bindings![Test::BUTTON])]),
    ));

    app.update();

    let mut gamepad1 = app.world_mut().get_mut::<Gamepad>(gamepad_entity1).unwrap();
    gamepad1.analog_mut().set(Test::BUTTON, 1.0);

    app.update();

    let mut actions = app.world_mut().query::<&TriggerState>();
    let state = *actions.single(app.world()).unwrap();
    assert_eq!(state, TriggerState::Fired);

    let mut gamepad1 = app.world_mut().get_mut::<Gamepad>(gamepad_entity1).unwrap();
    gamepad1.analog_mut().set(Test::BUTTON, 0.0);

    let mut gamepad2 = app.world_mut().get_mut::<Gamepad>(gamepad_entity2).unwrap();
    gamepad2.analog_mut().set(Test::BUTTON, 1.0);

    app.update();

    let state = *actions.single(app.world()).unwrap();
    assert_eq!(state, TriggerState::Fired);
}

#[test]
fn by_id() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, InputPlugin, EnhancedInputPlugin))
        .add_input_context::<TestContext>()
        .finish();

    let gamepad_entity1 = app.world_mut().spawn(Gamepad::default()).id();
    let gamepad_entity2 = app.world_mut().spawn(Gamepad::default()).id();

    app.world_mut().spawn((
        TestContext,
        GamepadDevice::Single(gamepad_entity1),
        actions!(TestContext[(Action::<Test>::new(), bindings![Test::BUTTON])]),
    ));

    app.update();

    let mut gamepad1 = app.world_mut().get_mut::<Gamepad>(gamepad_entity1).unwrap();
    gamepad1.analog_mut().set(Test::BUTTON, 1.0);

    app.update();

    let mut actions = app.world_mut().query::<&TriggerState>();
    let state = *actions.single(app.world()).unwrap();
    assert_eq!(state, TriggerState::Fired);

    let mut gamepad1 = app.world_mut().get_mut::<Gamepad>(gamepad_entity1).unwrap();
    gamepad1.analog_mut().set(Test::BUTTON, 0.0);

    let mut gamepad2 = app.world_mut().get_mut::<Gamepad>(gamepad_entity2).unwrap();
    gamepad2.analog_mut().set(Test::BUTTON, 1.0);

    app.update();

    let state = *actions.single(app.world()).unwrap();
    assert_eq!(state, TriggerState::None);
}

#[derive(Component)]
struct TestContext;

#[derive(InputAction)]
#[action_output(bool)]
struct Test;

impl Test {
    const BUTTON: GamepadButton = GamepadButton::South;
}
