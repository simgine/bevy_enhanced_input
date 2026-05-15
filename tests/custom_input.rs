use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;
use test_log::test;

#[test]
fn with_value() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, EnhancedInputPlugin))
        .add_input_context::<TestContext>()
        .finish();

    let mut custom_inputs = app.world_mut().resource_mut::<CustomInputs>();
    let test = custom_inputs.register_input();
    custom_inputs.insert(test, ActionValue::Axis1D(0.75));

    app.world_mut().spawn((
        TestContext,
        actions!(TestContext[(Action::<TestValue>::new(), bindings![Binding::Custom(test)])]),
    ));

    app.update();

    let mut q = app
        .world_mut()
        .query::<(&Action<TestValue>, &TriggerState)>();
    let (&value, &state) = q.single(app.world()).unwrap();
    assert_eq!(*value, 0.75);
    assert_eq!(state, TriggerState::Fired);
}

#[test]
fn no_value() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, EnhancedInputPlugin))
        .add_input_context::<TestContext>()
        .finish();

    let test = app
        .world_mut()
        .resource_mut::<CustomInputs>()
        .register_input();

    app.world_mut().spawn((
        TestContext,
        actions!(TestContext[(Action::<TestValue>::new(), bindings![Binding::Custom(test)])]),
    ));

    app.update();

    let mut q = app.world_mut().query::<&Action<TestValue>>();
    let &value = q.single(app.world()).unwrap();
    assert_eq!(*value, 0.0);
}

#[derive(Component)]
struct TestContext;

#[derive(InputAction)]
#[action_output(f32)]
struct TestValue;
