use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;
use test_log::test;

#[test]
fn custom_source_value_reaches_action() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, EnhancedInputPlugin))
        .add_binding_source::<StagedSource>()
        .add_input_context::<TestContext>()
        .finish();

    app.world_mut().spawn((
        TestContext,
        actions!(TestContext[(
            Action::<TestValue>::new(),
            bindings![(Binding::Custom, StagedSource(0.75))],
        )]),
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
fn binding_entity_modifier_applies_to_custom_source() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, EnhancedInputPlugin))
        .add_binding_source::<StagedSource>()
        .add_input_context::<TestContext>()
        .finish();

    app.world_mut().spawn((
        TestContext,
        actions!(TestContext[(
            Action::<TestValue>::new(),
            bindings![(Binding::Custom, StagedSource(0.5), Negate::all())],
        )]),
    ));

    app.update();

    let mut q = app.world_mut().query::<&Action<TestValue>>();
    let &value = q.single(app.world()).unwrap();
    assert_eq!(*value, -0.5);
}

#[test]
fn unstaged_custom_source_reads_zero() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, EnhancedInputPlugin))
        .add_input_context::<TestContext>()
        .finish();

    app.world_mut().spawn((
        TestContext,
        actions!(TestContext[(
            Action::<TestValue>::new(),
            bindings![Binding::Custom],
        )]),
    ));

    app.update();

    let mut q = app.world_mut().query::<&Action<TestValue>>();
    let &value = q.single(app.world()).unwrap();
    assert_eq!(*value, 0.0);
}

#[derive(Component, Debug)]
struct StagedSource(f32);

impl BindingSource for StagedSource {
    fn read(&mut self, _: &ActionsQuery, _: &ContextTime) -> ActionValue {
        ActionValue::Axis1D(self.0)
    }
}

#[derive(Component)]
struct TestContext;

#[derive(InputAction)]
#[action_output(f32)]
struct TestValue;
