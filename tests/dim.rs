use bevy::{input::InputPlugin, prelude::*};
use bevy_enhanced_input::prelude::*;
use test_log::test;

#[test]
fn bool() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, InputPlugin, EnhancedInputPlugin))
        .add_input_context::<TestContext>()
        .finish();

    app.world_mut().spawn((
        TestContext,
        actions!(TestContext[(Action::<Bool>::new(), bindings![Bool::KEY])]),
    ));

    app.update();

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(Bool::KEY);

    app.update();

    let mut actions = app.world_mut().query::<&Action<Bool>>();

    let action = *actions.single(app.world()).unwrap();
    assert!(*action);

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .release(Bool::KEY);

    app.update();

    let action = *actions.single(app.world()).unwrap();
    assert!(!*action);
}

#[test]
fn axis1d() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, InputPlugin, EnhancedInputPlugin))
        .add_input_context::<TestContext>()
        .finish();

    app.world_mut().spawn((
        TestContext,
        actions!(TestContext[(Action::<Axis1D>::new(), bindings![Axis1D::KEY])]),
    ));

    app.update();

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(Axis1D::KEY);

    app.update();

    let mut actions = app.world_mut().query::<&Action<Axis1D>>();

    let action = *actions.single(app.world()).unwrap();
    assert_eq!(*action, 1.0);

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .release(Axis1D::KEY);

    app.update();

    let action = *actions.single(app.world()).unwrap();
    assert_eq!(*action, 0.0);
}

#[test]
fn axis2d() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, InputPlugin, EnhancedInputPlugin))
        .add_input_context::<TestContext>()
        .finish();

    app.world_mut().spawn((
        TestContext,
        actions!(TestContext[(Action::<Axis2D>::new(), bindings![Axis2D::KEY])]),
    ));

    app.update();

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(Axis2D::KEY);

    app.update();

    let mut actions = app.world_mut().query::<&Action<Axis2D>>();

    let action = *actions.single(app.world()).unwrap();
    assert_eq!(*action, (1.0, 0.0).into());

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .release(Axis2D::KEY);

    app.update();

    let action = *actions.single(app.world()).unwrap();
    assert_eq!(*action, Vec2::ZERO);
}

#[test]
fn axis3d() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, InputPlugin, EnhancedInputPlugin))
        .add_input_context::<TestContext>()
        .finish();

    app.world_mut().spawn((
        TestContext,
        actions!(TestContext[(Action::<Axis3D>::new(), bindings![Axis3D::KEY])]),
    ));

    app.update();

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(Axis3D::KEY);

    app.update();

    let mut actions = app.world_mut().query::<&Action<Axis3D>>();

    let action = *actions.single(app.world()).unwrap();
    assert_eq!(*action, (1.0, 0.0, 0.0).into());

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .release(Axis3D::KEY);

    app.update();

    let action = *actions.single(app.world()).unwrap();
    assert_eq!(*action, Vec3::ZERO);
}

#[derive(Component)]
struct TestContext;

#[derive(InputAction)]
#[action_output(bool)]
struct Bool;

impl Bool {
    const KEY: KeyCode = KeyCode::KeyA;
}

#[derive(InputAction)]
#[action_output(f32)]
struct Axis1D;

impl Axis1D {
    const KEY: KeyCode = KeyCode::KeyB;
}

#[derive(InputAction)]
#[action_output(Vec2)]
struct Axis2D;

impl Axis2D {
    const KEY: KeyCode = KeyCode::KeyC;
}

#[derive(InputAction)]
#[action_output(Vec3)]
struct Axis3D;

impl Axis3D {
    const KEY: KeyCode = KeyCode::KeyD;
}
