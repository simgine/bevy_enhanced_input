use bevy::{input::InputPlugin, prelude::*, time::TimeUpdateStrategy};
use bevy_enhanced_input::prelude::*;
use test_log::test;

#[test]
fn once_in_two_frames() {
    let time_step = Time::<Fixed>::default().timestep() / 2;

    let mut app = App::new();
    app.add_plugins((MinimalPlugins, InputPlugin, EnhancedInputPlugin))
        .insert_resource(TimeUpdateStrategy::ManualDuration(time_step))
        .add_input_context_to::<FixedPreUpdate, TestContext>()
        .finish();

    app.world_mut().spawn((
        TestContext,
        actions!(TestContext[(Action::<Test>::new(), bindings![Test::KEY])]),
    ));

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(Test::KEY);

    let mut actions = app.world_mut().query::<&ActionEvents>();

    for frame in 0..2 {
        app.update();

        let events = *actions.single(app.world()).unwrap();
        assert!(events.is_empty(), "shouldn't fire on frame {frame}");
    }

    for frame in 2..4 {
        app.update();

        let events = *actions.single(app.world()).unwrap();
        assert_eq!(
            events,
            ActionEvents::START | ActionEvents::FIRE,
            "should maintain start-firing on frame {frame}"
        );
    }
}

#[test]
fn twice_in_one_frame() {
    let time_step = Time::<Fixed>::default().timestep() * 2;

    let mut app = App::new();
    app.add_plugins((MinimalPlugins, InputPlugin, EnhancedInputPlugin))
        .insert_resource(TimeUpdateStrategy::ManualDuration(time_step))
        .add_input_context_to::<FixedPreUpdate, TestContext>()
        .finish();

    app.world_mut().spawn((
        TestContext,
        actions!(TestContext[(Action::<Test>::new(), bindings![Test::KEY])]),
    ));

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(Test::KEY);

    app.update();

    let mut actions = app.world_mut().query::<&ActionEvents>();
    let events = *actions.single(app.world()).unwrap();
    assert!(
        events.is_empty(),
        "`FixedMain` should never run on the first frame"
    );

    app.update();

    let events = *actions.single(app.world()).unwrap();
    assert_eq!(
        events,
        ActionEvents::FIRE,
        "should run twice, so it shouldn't be started on the second run"
    );
}

#[derive(Component, TypePath)]
struct TestContext;

#[derive(InputAction)]
#[action_output(bool)]
struct Test;

impl Test {
    const KEY: KeyCode = KeyCode::KeyA;
}
