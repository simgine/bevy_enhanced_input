use bevy::{ecs::spawn::SpawnWith, input::InputPlugin, prelude::*};
use bevy_enhanced_input::prelude::*;
use test_log::test;

#[test]
fn input_level() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, InputPlugin, EnhancedInputPlugin))
        .add_input_context::<TestContext>()
        .finish();

    app.world_mut().spawn((
        TestContext,
        Actions::<TestContext>::spawn(SpawnWith(|context: &mut ActionSpawner<_>| {
            let chord_member = context
                .spawn((Action::<ChordMember>::new(), bindings![ChordMember::KEY]))
                .id();
            let blocker = context
                .spawn((Action::<Blocker>::new(), bindings![Blocker::KEY]))
                .id();
            context.spawn((
                Action::<Test>::new(),
                bindings![
                    (
                        Test::KEY1,
                        Chord::single(chord_member),
                        BlockBy::single(blocker),
                        Down::default(),
                        JustReleased::default(),
                        Scale::splat(2.0),
                        SwizzleAxis::YXZ
                    ),
                    (
                        Test::KEY2,
                        Chord::single(chord_member),
                        BlockBy::single(blocker),
                        Down::default(),
                        JustReleased::default(),
                        Negate::all(),
                        SwizzleAxis::YXZ
                    ),
                ],
            ));
        })),
    ));

    app.update();

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(Test::KEY1);

    app.update();

    let mut actions = app.world_mut().query::<(&Action<Test>, &ActionState)>();

    let (&action, &state) = actions.single(app.world()).unwrap();
    assert_eq!(*action, Vec2::Y * 2.0);
    assert_eq!(state, ActionState::Ongoing);

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(ChordMember::KEY);

    app.update();

    let (&action, &state) = actions.single(app.world()).unwrap();
    assert_eq!(*action, Vec2::Y * 2.0);
    assert_eq!(state, ActionState::Fired);

    let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
    keys.release(Test::KEY1);
    keys.press(Test::KEY2);

    app.update();

    let (&action, &state) = actions.single(app.world()).unwrap();
    assert_eq!(*action, Vec2::NEG_Y);
    assert_eq!(state, ActionState::Fired);

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(Test::KEY1);

    app.update();

    let (&action, &state) = actions.single(app.world()).unwrap();
    assert_eq!(*action, Vec2::Y);
    assert_eq!(state, ActionState::Fired);

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(Blocker::KEY);

    app.update();

    let (&action, &state) = actions.single(app.world()).unwrap();
    assert_eq!(*action, Vec2::ZERO);
    assert_eq!(
        state,
        ActionState::None,
        "if a blocker condition fails, it should override other conditions"
    );
}

#[test]
fn action_level() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, InputPlugin, EnhancedInputPlugin))
        .add_input_context::<TestContext>()
        .finish();

    app.world_mut().spawn((
        TestContext,
        Actions::<TestContext>::spawn(SpawnWith(|context: &mut ActionSpawner<_>| {
            let chord_member = context
                .spawn((Action::<ChordMember>::new(), bindings![ChordMember::KEY]))
                .id();
            let blocker = context
                .spawn((Action::<Blocker>::new(), bindings![Blocker::KEY]))
                .id();
            context.spawn((
                Action::<Test>::new(),
                Down::default(),
                JustReleased::default(),
                Chord::single(chord_member),
                BlockBy::single(blocker),
                SwizzleAxis::YXZ,
                Negate::all(),
                Scale::splat(2.0),
                bindings![Test::KEY1, Test::KEY2,],
            ));
        })),
    ));

    app.update();

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(Test::KEY1);

    app.update();

    let mut actions = app.world_mut().query::<(&Action<Test>, &ActionState)>();

    let (&action, &state) = actions.single(app.world()).unwrap();
    assert_eq!(*action, Vec2::NEG_Y * 2.0);
    assert_eq!(state, ActionState::Ongoing);

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(ChordMember::KEY);

    app.update();

    let (&action, &state) = actions.single(app.world()).unwrap();
    assert_eq!(*action, Vec2::NEG_Y * 2.0);
    assert_eq!(state, ActionState::Fired);

    let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
    keys.release(Test::KEY1);
    keys.press(Test::KEY2);

    app.update();

    let (&action, &state) = actions.single(app.world()).unwrap();
    assert_eq!(*action, Vec2::NEG_Y * 2.0);
    assert_eq!(state, ActionState::Fired);

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(Test::KEY1);

    app.update();

    let (&action, &state) = actions.single(app.world()).unwrap();
    assert_eq!(*action, Vec2::NEG_Y * 4.0);
    assert_eq!(state, ActionState::Fired);

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(Blocker::KEY);

    app.update();

    let (&action, &state) = actions.single(app.world()).unwrap();
    assert_eq!(*action, Vec2::NEG_Y * 4.0);
    assert_eq!(
        state,
        ActionState::None,
        "if a blocker condition fails, it should override other conditions"
    );
}

#[test]
fn both_levels() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, InputPlugin, EnhancedInputPlugin))
        .add_input_context::<TestContext>()
        .finish();

    app.world_mut().spawn((
        TestContext,
        Actions::<TestContext>::spawn(SpawnWith(|context: &mut ActionSpawner<_>| {
            let chord_member = context
                .spawn((Action::<ChordMember>::new(), bindings![ChordMember::KEY]))
                .id();
            let blocker = context
                .spawn((Action::<Blocker>::new(), bindings![Blocker::KEY]))
                .id();
            context.spawn((
                Action::<Test>::new(),
                JustReleased::default(),
                Chord::single(chord_member),
                BlockBy::single(blocker),
                SwizzleAxis::YXZ,
                bindings![
                    (Test::KEY1, Down::default(), Scale::splat(2.0)),
                    (Test::KEY2, Down::default(), Negate::all()),
                ],
            ));
        })),
    ));

    app.update();

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(Test::KEY1);

    app.update();

    let mut actions = app.world_mut().query::<(&Action<Test>, &ActionState)>();

    let (&action, &state) = actions.single(app.world()).unwrap();
    assert_eq!(*action, Vec2::Y * 2.0);
    assert_eq!(state, ActionState::Ongoing);

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(ChordMember::KEY);

    app.update();

    let (&action, &state) = actions.single(app.world()).unwrap();
    assert_eq!(*action, Vec2::Y * 2.0);
    assert_eq!(state, ActionState::Fired);

    let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
    keys.release(Test::KEY1);
    keys.press(Test::KEY2);

    app.update();

    let (&action, &state) = actions.single(app.world()).unwrap();
    assert_eq!(*action, Vec2::NEG_Y);
    assert_eq!(state, ActionState::Fired);

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(Test::KEY1);

    app.update();

    let (&action, &state) = actions.single(app.world()).unwrap();
    assert_eq!(*action, Vec2::Y);
    assert_eq!(state, ActionState::Fired);

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(Blocker::KEY);

    app.update();

    let (&action, &state) = actions.single(app.world()).unwrap();
    assert_eq!(*action, Vec2::Y);
    assert_eq!(
        state,
        ActionState::None,
        "if a blocker condition fails, it should override other conditions"
    );
}

#[derive(Component)]
struct TestContext;

#[derive(InputAction)]
#[action_output(Vec2)]
struct Test;

impl Test {
    const KEY1: KeyCode = KeyCode::KeyA;
    const KEY2: KeyCode = KeyCode::KeyB;
}

#[derive(InputAction)]
#[action_output(bool)]
struct ChordMember;

impl ChordMember {
    const KEY: KeyCode = KeyCode::KeyG;
}

#[derive(InputAction)]
#[action_output(bool)]
struct Blocker;

impl Blocker {
    const KEY: KeyCode = KeyCode::KeyH;
}
