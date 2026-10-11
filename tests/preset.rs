use bevy::{input::InputPlugin, prelude::*, scene::ScenePlugin};
use bevy_enhanced_input::prelude::*;
use test_log::test;

#[test]
fn keys() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, InputPlugin, EnhancedInputPlugin))
        .add_input_context::<TestContext>()
        .finish();

    app.world_mut().spawn((
        TestContext,
        actions!(
            TestContext[(
                Action::<Test>::new(),
                Bindings::spawn((
                    Cardinal::wasd_keys(),
                    Cardinal::arrows(),
                    Bidirectional {
                        positive: Binding::from(KeyCode::NumpadAdd),
                        negative: Binding::from(KeyCode::NumpadSubtract),
                    },
                    Spatial {
                        forward: Binding::from(KeyCode::Digit0),
                        backward: Binding::from(KeyCode::Digit1),
                        left: Binding::from(KeyCode::Digit2),
                        right: Binding::from(KeyCode::Digit3),
                        up: Binding::from(KeyCode::Digit4),
                        down: Binding::from(KeyCode::Digit5),
                    },
                    Ordinal::numpad(),
                ))
            )]
        ),
    ));

    assert_keys(&mut app);
}

#[test]
fn keys_bsn() {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        AssetPlugin::default(),
        ScenePlugin,
        InputPlugin,
        EnhancedInputPlugin,
    ))
    .add_input_context::<TestContext>()
    .finish();

    app.world_mut()
        .spawn_scene(bsn! {
            TestContext
            Actions<TestContext> [
                Action::<Test>::new()
                Bindings [
                    {preset::cardinal::wasd_keys()}
                    --
                    {preset::cardinal::arrows()}
                    --
                    {preset::bidirectional::new(KeyCode::NumpadAdd, KeyCode::NumpadSubtract)}
                    --
                    {preset::spatial::new(
                        KeyCode::Digit0,
                        KeyCode::Digit1,
                        KeyCode::Digit2,
                        KeyCode::Digit3,
                        KeyCode::Digit4,
                        KeyCode::Digit5,
                    )}
                    --
                    {preset::ordinal::numpad()}
                ]
            ]
        })
        .unwrap();

    assert_keys(&mut app);
}

fn assert_keys(app: &mut App) {
    app.update();

    let mut actions = app.world_mut().query::<&Action<Test>>();
    for (key, dir) in [
        (KeyCode::KeyW, UP),
        (KeyCode::KeyA, LEFT),
        (KeyCode::KeyS, DOWN),
        (KeyCode::KeyD, RIGHT),
        (KeyCode::ArrowUp, UP),
        (KeyCode::ArrowLeft, LEFT),
        (KeyCode::ArrowDown, DOWN),
        (KeyCode::ArrowRight, RIGHT),
        (KeyCode::NumpadSubtract, LEFT),
        (KeyCode::NumpadAdd, RIGHT),
        (KeyCode::Digit0, FORWARD),
        (KeyCode::Digit1, BACKWARD),
        (KeyCode::Digit2, LEFT),
        (KeyCode::Digit3, RIGHT),
        (KeyCode::Digit4, UP),
        (KeyCode::Digit5, DOWN),
        (KeyCode::Numpad8, UP),
        (KeyCode::Numpad9, RIGHT_UP),
        (KeyCode::Numpad6, RIGHT),
        (KeyCode::Numpad3, RIGHT_DOWN),
        (KeyCode::Numpad2, DOWN),
        (KeyCode::Numpad1, LEFT_DOWN),
        (KeyCode::Numpad4, LEFT),
        (KeyCode::Numpad7, LEFT_UP),
    ] {
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(key);

        app.update();

        let action = *actions.single(app.world()).unwrap();
        assert_eq!(*action, dir, "`{key:?}` should result in `{dir}`");

        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .release(key);

        app.update();
    }
}

#[test]
fn dpad() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, InputPlugin, EnhancedInputPlugin))
        .add_input_context::<TestContext>()
        .finish();

    let gamepad_entity = app.world_mut().spawn(Gamepad::default()).id();
    app.world_mut().spawn((
        TestContext,
        GamepadDevice::Single(gamepad_entity),
        actions!(TestContext[(Action::<Test>::new(), Bindings::spawn(Cardinal::dpad()))]),
    ));

    assert_dpad(&mut app, gamepad_entity);
}

#[test]
fn dpad_bsn() {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        AssetPlugin::default(),
        ScenePlugin,
        InputPlugin,
        EnhancedInputPlugin,
    ))
    .add_input_context::<TestContext>()
    .finish();

    let gamepad_entity = app.world_mut().spawn(Gamepad::default()).id();
    app.world_mut()
        .spawn_scene(bsn! {
            TestContext
            ~GamepadDevice::Single(gamepad_entity)
            Actions<TestContext> [
                Action::<Test>::new()
                Bindings [
                    {preset::cardinal::dpad()}
                ]
            ]
        })
        .unwrap();

    assert_dpad(&mut app, gamepad_entity);
}

fn assert_dpad(app: &mut App, gamepad_entity: Entity) {
    app.update();

    let mut actions = app.world_mut().query::<&Action<Test>>();
    for (button, dir) in [
        (GamepadButton::DPadUp, UP),
        (GamepadButton::DPadLeft, LEFT),
        (GamepadButton::DPadDown, DOWN),
        (GamepadButton::DPadRight, RIGHT),
    ] {
        let mut gamepad = app.world_mut().get_mut::<Gamepad>(gamepad_entity).unwrap();
        gamepad.analog_mut().set(button, 1.0);

        app.update();

        let action = *actions.single(app.world()).unwrap();
        assert_eq!(*action, dir, "`{button:?}` should result in `{dir}`");

        let mut gamepad = app.world_mut().get_mut::<Gamepad>(gamepad_entity).unwrap();
        gamepad.analog_mut().set(button, 0.0);

        app.update();
    }
}

#[test]
fn sticks() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, InputPlugin, EnhancedInputPlugin))
        .add_input_context::<TestContext>()
        .finish();

    let gamepad_entity = app.world_mut().spawn(Gamepad::default()).id();
    app.world_mut().spawn((
        TestContext,
        GamepadDevice::Single(gamepad_entity),
        actions!(
            TestContext[(
                Action::<Test>::new(),
                Bindings::spawn((Axial::left_stick(), Axial::right_stick()))
            )]
        ),
    ));

    assert_sticks(&mut app, gamepad_entity);
}

#[test]
fn sticks_bsn() {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        AssetPlugin::default(),
        ScenePlugin,
        InputPlugin,
        EnhancedInputPlugin,
    ))
    .add_input_context::<TestContext>()
    .finish();

    let gamepad_entity = app.world_mut().spawn(Gamepad::default()).id();
    app.world_mut()
        .spawn_scene(bsn! {
            TestContext
            ~GamepadDevice::Single(gamepad_entity)
            Actions<TestContext> [
                Action::<Test>::new()
                Bindings [
                    {preset::axial::left_stick()}
                    --
                    {preset::axial::right_stick()}
                ]
            ]
        })
        .unwrap();

    assert_sticks(&mut app, gamepad_entity);
}

fn assert_sticks(app: &mut App, gamepad_entity: Entity) {
    app.update();

    let mut actions = app.world_mut().query::<&Action<Test>>();
    for (axis, dirs) in [
        (GamepadAxis::LeftStickX, [LEFT, RIGHT]),
        (GamepadAxis::RightStickX, [LEFT, RIGHT]),
        (GamepadAxis::LeftStickY, [DOWN, UP]),
        (GamepadAxis::RightStickY, [DOWN, UP]),
    ] {
        for (dir, value) in dirs.into_iter().zip([-1.0, 1.0]) {
            let mut gamepad = app.world_mut().get_mut::<Gamepad>(gamepad_entity).unwrap();
            gamepad.analog_mut().set(axis, value);

            app.update();

            let action = *actions.single(app.world()).unwrap();
            assert_eq!(*action, dir, "`{axis:?}` should result in `{dir}`");

            let mut gamepad = app.world_mut().get_mut::<Gamepad>(gamepad_entity).unwrap();
            gamepad.analog_mut().set(axis, 0.0);

            app.update();
        }
    }
}

const RIGHT: Vec3 = Vec3::X;
const LEFT: Vec3 = Vec3::NEG_X;
const BACKWARD: Vec3 = Vec3::Z;
const FORWARD: Vec3 = Vec3::NEG_Z;
const UP: Vec3 = Vec3::Y;
const DOWN: Vec3 = Vec3::NEG_Y;
const RIGHT_UP: Vec3 = Vec3::new(1.0, 1.0, 0.0);
const RIGHT_DOWN: Vec3 = Vec3::new(1.0, -1.0, 0.0);
const LEFT_DOWN: Vec3 = Vec3::new(-1.0, -1.0, 0.0);
const LEFT_UP: Vec3 = Vec3::new(-1.0, 1.0, 0.0);

#[derive(Component, Default, Clone)]
struct TestContext;

#[derive(InputAction)]
#[action_output(Vec3)]
struct Test;
