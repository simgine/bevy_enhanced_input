use bevy::{ecs::spawn::SpawnWith, input::InputPlugin, prelude::*};
use bevy_enhanced_input::prelude::{Release, *};
use test_log::test;

#[test]
fn explicit() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, InputPlugin, EnhancedInputPlugin))
        .add_input_context::<TestContext>()
        .finish();

    app.world_mut().spawn((
        TestContext,
        actions!(TestContext[(Action::<Test>::new(), Down::default(), bindings![Test::KEY])]),
    ));

    app.update();

    let mut actions = app.world_mut().query::<(&Action<Test>, &ActionState)>();

    let (&action, &state) = actions.single(app.world()).unwrap();
    assert!(!*action);
    assert_eq!(state, ActionState::None);

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(Test::KEY);

    app.update();

    let (&action, &state) = actions.single(app.world()).unwrap();
    assert!(*action);
    assert_eq!(state, ActionState::Fired);

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .release(Test::KEY);

    app.update();

    let (&action, &state) = actions.single(app.world()).unwrap();
    assert!(!*action);
    assert_eq!(state, ActionState::None);
}

#[test]
fn implicit() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, InputPlugin, EnhancedInputPlugin))
        .add_input_context::<TestContext>()
        .finish();

    app.world_mut().spawn((
        TestContext,
        Actions::<TestContext>::spawn(SpawnWith(|context: &mut ActionSpawner<_>| {
            let release = context
                .spawn((
                    Action::<OnRelease>::new(),
                    Release::default(),
                    bindings![OnRelease::KEY],
                ))
                .id();
            context.spawn((Action::<Test>::new(), Chord::single(release)));
        })),
    ));

    app.update();

    let mut release_actions = app
        .world_mut()
        .query::<(&Action<OnRelease>, &ActionState)>();

    let (&release_action, &release_state) = release_actions.single(app.world()).unwrap();
    assert!(!*release_action);
    assert_eq!(release_state, ActionState::None);

    let mut test_actions = app.world_mut().query::<(&Action<Test>, &ActionState)>();

    let (&test_action, &test_state) = test_actions.single(app.world()).unwrap();
    assert!(!*test_action);
    assert_eq!(test_state, ActionState::None);

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(OnRelease::KEY);

    app.update();

    let (&release_action, &release_state) = release_actions.single(app.world()).unwrap();
    assert!(*release_action);
    assert_eq!(release_state, ActionState::Ongoing);

    let (&test_action, &test_state) = test_actions.single(app.world()).unwrap();
    assert!(!*test_action);
    assert_eq!(test_state, ActionState::Ongoing);

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .release(OnRelease::KEY);

    app.update();

    let (&release_action, &release_state) = release_actions.single(app.world()).unwrap();
    assert!(!*release_action);
    assert_eq!(release_state, ActionState::Fired);

    let (&test_action, &test_state) = test_actions.single(app.world()).unwrap();
    assert!(!*test_action);
    assert_eq!(test_state, ActionState::Fired);

    app.update();

    let (&release_action, &release_state) = release_actions.single(app.world()).unwrap();
    assert!(!*release_action);
    assert_eq!(release_state, ActionState::None);

    let (&test_action, &test_state) = test_actions.single(app.world()).unwrap();
    assert!(!*test_action);
    assert_eq!(test_state, ActionState::None);
}

#[test]
fn blocker() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, InputPlugin, EnhancedInputPlugin))
        .add_input_context::<TestContext>()
        .finish();

    app.world_mut().spawn((
        TestContext,
        Actions::<TestContext>::spawn(SpawnWith(|context: &mut ActionSpawner<_>| {
            let release = context
                .spawn((
                    Action::<OnRelease>::new(),
                    Release::default(),
                    bindings![OnRelease::KEY],
                ))
                .id();
            context.spawn((
                Action::<Test>::new(),
                BlockBy::single(release),
                bindings![Test::KEY],
            ));
        })),
    ));

    app.update();

    let mut release_actions = app
        .world_mut()
        .query::<(&Action<OnRelease>, &ActionState)>();

    let (&release_action, &release_state) = release_actions.single(app.world()).unwrap();
    assert!(!*release_action);
    assert_eq!(release_state, ActionState::None);

    let mut test_actions = app.world_mut().query::<(&Action<Test>, &ActionState)>();

    let (&test_action, &test_state) = test_actions.single(app.world()).unwrap();
    assert!(!*test_action);
    assert_eq!(test_state, ActionState::None);

    let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
    keys.press(OnRelease::KEY);
    keys.press(Test::KEY);

    app.update();

    let (&release_action, &release_state) = release_actions.single(app.world()).unwrap();
    assert!(*release_action);
    assert_eq!(release_state, ActionState::Ongoing);

    let (&test_action, &test_state) = test_actions.single(app.world()).unwrap();
    assert!(*test_action);
    assert_eq!(test_state, ActionState::Fired);

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .release(OnRelease::KEY);

    app.update();

    let (&release_action, &release_state) = release_actions.single(app.world()).unwrap();
    assert!(!*release_action);
    assert_eq!(release_state, ActionState::Fired);

    let (&test_action, &test_state) = test_actions.single(app.world()).unwrap();
    assert!(*test_action);
    assert_eq!(test_state, ActionState::None);

    app.update();

    let (&release_action, &release_state) = release_actions.single(app.world()).unwrap();
    assert!(!*release_action);
    assert_eq!(release_state, ActionState::None);

    let (&test_action, &test_state) = test_actions.single(app.world()).unwrap();
    assert!(*test_action);
    assert_eq!(test_state, ActionState::Fired);
}

#[derive(Component)]
struct TestContext;

#[derive(InputAction)]
#[action_output(bool)]
struct Test;

impl Test {
    const KEY: KeyCode = KeyCode::KeyA;
}

#[derive(InputAction)]
#[action_output(bool)]
struct OnRelease;

impl OnRelease {
    const KEY: KeyCode = KeyCode::KeyB;
}
