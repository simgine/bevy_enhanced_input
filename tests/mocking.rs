use core::time::Duration;

use bevy::{prelude::*, time::TimeUpdateStrategy};
use bevy_enhanced_input::{context::ExternallyMocked, prelude::*};
use test_log::test;

#[test]
fn updates() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, EnhancedInputPlugin))
        .add_input_context::<TestContext>()
        .finish();

    app.world_mut().spawn((
        TestContext,
        actions!(
            TestContext[(
                Action::<Test>::new(),
                ActionMock::once(ActionState::Fired, true)
            )]
        ),
    ));

    app.update();

    let mut actions = app
        .world_mut()
        .query::<(&Action<Test>, &ActionState, &ActionEvents)>();

    let (&action, &state, &events) = actions.single(app.world()).unwrap();
    assert!(*action);
    assert_eq!(state, ActionState::Fired);
    assert_eq!(events, ActionEvents::FIRE | ActionEvents::START);

    app.update();

    let (&action, &state, &events) = actions.single(app.world()).unwrap();
    assert!(!*action);
    assert_eq!(state, ActionState::None);
    assert_eq!(events, ActionEvents::COMPLETE);
}

#[test]
fn updates_when_using_extension() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, EnhancedInputPlugin))
        .add_input_context::<TestContext>()
        .finish();

    let context = app
        .world_mut()
        // Using implicit `ActionMock`
        .spawn((TestContext, actions!(TestContext[Action::<Test>::new()])))
        .id();

    let mut actions = app
        .world_mut()
        .query::<(&Action<Test>, &ActionState, &ActionEvents)>();

    app.update();

    let (&action, &state, _events) = actions.single(app.world()).unwrap();
    assert!(!*action);
    assert_eq!(state, ActionState::None);

    app.world_mut()
        .commands()
        .entity(context)
        .mock::<TestContext, Test>(ActionMock::once(ActionState::Fired, true));

    app.update();

    let (&action, &state, &events) = actions.single(app.world()).unwrap();
    assert!(*action);
    assert_eq!(state, ActionState::Fired);
    assert_eq!(events, ActionEvents::FIRE | ActionEvents::START);

    app.update();

    let (&action, &state, &events) = actions.single(app.world()).unwrap();
    assert!(!*action);
    assert_eq!(state, ActionState::None);
    assert_eq!(events, ActionEvents::COMPLETE);
}

#[test]
fn duration() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, EnhancedInputPlugin))
        .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_millis(1)))
        .add_input_context::<TestContext>()
        .finish();

    // Update once to get a non-zero delta-time.
    app.update();

    app.world_mut().spawn((
        TestContext,
        actions!(
            TestContext[(
                Action::<Test>::new(),
                ActionMock::new(ActionState::Fired, true, Duration::from_millis(2))
            )]
        ),
    ));

    app.update();

    let mut actions = app
        .world_mut()
        .query::<(&Action<Test>, &ActionState, &ActionEvents)>();

    let (&action, &state, &events) = actions.single(app.world()).unwrap();
    assert!(*action);
    assert_eq!(state, ActionState::Fired);
    assert_eq!(events, ActionEvents::FIRE | ActionEvents::START);

    app.update();

    let (&action, &state, &events) = actions.single(app.world()).unwrap();
    assert!(*action);
    assert_eq!(state, ActionState::Fired);
    assert_eq!(events, ActionEvents::FIRE);

    app.update();

    let (&action, &state, &events) = actions.single(app.world()).unwrap();
    assert!(!*action);
    assert_eq!(state, ActionState::None);
    assert_eq!(events, ActionEvents::COMPLETE);
}

#[test]
fn manual() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, EnhancedInputPlugin))
        .add_input_context::<TestContext>()
        .finish();

    app.world_mut().spawn((
        TestContext,
        actions!(
            TestContext[(
                Action::<Test>::new(),
                ActionMock::new(ActionState::Fired, true, MockSpan::Manual),
            )]
        ),
    ));

    app.update();

    let mut actions = app
        .world_mut()
        .query::<(&Action<Test>, &ActionState, &ActionEvents, &mut ActionMock)>();

    let (&action, &state, &events, _) = actions.single(app.world()).unwrap();
    assert!(*action);
    assert_eq!(state, ActionState::Fired);
    assert_eq!(events, ActionEvents::FIRE | ActionEvents::START);

    app.update();

    let (&action, &state, &events, mut mock) = actions.single_mut(app.world_mut()).unwrap();
    assert!(*action);
    assert_eq!(state, ActionState::Fired);
    assert_eq!(events, ActionEvents::FIRE);

    mock.enabled = false;

    app.update();

    let (&action, &state, &events, _) = actions.single(app.world()).unwrap();
    assert!(!*action);
    assert_eq!(state, ActionState::None);
    assert_eq!(events, ActionEvents::COMPLETE);
}

#[test]
fn external_mock() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, EnhancedInputPlugin))
        .add_input_context::<TestContext>()
        .finish();

    app.world_mut().spawn((
        TestContext,
        actions!(
            TestContext[(
                Action::<Test>::new(),
                ExternallyMocked,
                ActionMock::once(ActionState::Fired, true)
            )]
        ),
    ));

    app.update();

    let mut actions = app
        .world_mut()
        .query::<(&Action<Test>, &ActionState, &ActionEvents)>();

    let (&action, &state, &events) = actions.single(app.world()).unwrap();
    assert!(
        !*action,
        "action shouldn't be updated because it marked as mocked externally"
    );
    assert_eq!(state, ActionState::None);
    assert_eq!(events, ActionEvents::empty());
}

#[derive(Component)]
struct TestContext;

#[derive(InputAction)]
#[action_output(bool)]
struct Test;
