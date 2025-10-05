/*!
Input manager for [Bevy](https://bevyengine.org), inspired by
[Unreal Engine Enhanced Input](https://dev.epicgames.com/documentation/en-us/unreal-engine/enhanced-input-in-unreal-engine).

# Quick start

## Prelude

We provide a [`prelude`] module, which exports most of the typically used traits and types.

## Plugins

Add [`EnhancedInputPlugin`] to your app:

```
use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;

let mut app = App::new();
app.add_plugins((MinimalPlugins, EnhancedInputPlugin));
```

## Core Concepts

- **Actions** represent something a player can do, like "Jump", "Movement", or "Open Menu". They are not tied to specific input.
- **Bindings** connect those actions to real input sources such as keyboard keys, mouse buttons, gamepad axes, etc.
- **Contexts** represent a certain input state the player can be in, such as "On foot" or "In car". They associate actions with
  entities and define when those actions are evaluated.

Contexts are regular components. Depending on your type of game, you may have a single global context
or multiple contexts for different gameplay states. Contexts can be layered, and any number of them can be active at the same time.
To register a component as an input context, you need to call [`InputContextAppExt::add_input_context`]. By default, contexts are
evaluated during [`PreUpdate`], but you can customize this by using [`InputContextAppExt::add_input_context_to`] instead.

Actions are represented by entities with the [`Action<A>`] component, where `A` is a user-defined marker that implements the
[`InputAction`] trait, which defines [`InputAction::Output`] type - the value the action produces. It could be [`bool`], [`f32`],
[`Vec2`] or [`Vec3`]. Actions associated with contexts via [`ActionOf`] relationship. We provide the [`actions!`] macro, which is
similar to [`related!`], but for actions. The relationship is generic over `C` because a single entity can have multiple associated
contexts.

Bindings are represented by entities with the [`Binding`] component. It can be constructed from various input types, such as
[`KeyCode`], [`MouseButton`], [`GamepadAxis`], etc. Bindings associated with actions via [`BindingOf`] relationship. Similar to [`actions!`],
we provide the [`bindings!`] macro to spawn related bindings. But unlike [`ActionOf<C>`], it's not generic, since each action is represented
by a separate entity.


```
use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;

# let mut app = App::new();
# app.add_plugins(EnhancedInputPlugin);
app.add_input_context::<Player>();

# let mut world = World::new();
world.spawn((
    Player,
    actions!(Player[
        (
            Action::<Jump>::new(),
            bindings![KeyCode::Space, GamepadButton::South],
        ),
        (
            Action::<Fire>::new(),
            bindings![MouseButton::Left, GamepadButton::RightTrigger2],
        ),
    ])
));

#[derive(Component)]
struct Player;

#[derive(InputAction)]
#[action_output(bool)]
struct Jump;

#[derive(InputAction)]
#[action_output(bool)]
struct Fire;
```

By default, input is read from all connected gamepads. You can customize this by adding the [`GamepadDevice`] component to the
context entity.

Context actions will be evaluated in the schedule associated at context registration. Contexts registered in the same
schedule will be evaluated in their spawning order, but you can override it by adding the [`ContextPriority`] component.
You can also activate or deactivate contexts by inserting [`ContextActivity`] component.

Actions also have [`ActionSettings`] component that customizes their behavior.

## Input modifiers

Action values are stored in two forms:
- In a typed form, as the [`Action<C>`] component.
- In a dynamically typed form, as the [`ActionValue`], which is one of the required components of [`Action<C>`].
  Its variant depends on the [`InputAction::Output`].

During [`EnhancedInputSet::Update`], input is read for each [`Binding`] as an [`ActionValue`], with the variant depending
on the input source. This value is then converted into the [`ActionValue`] on the associated action entity. For example,
key inputs are captured as [`bool`], but if the action's output type is [`Vec2`], the value will be assigned to the X axis
as `0.0` or `1.0`. See [`Binding`] for details on how each source is captured, and [`ActionValue::convert`] for how values
are transformed.

Then, during [`EnhancedInputSet::Apply`], the value from [`ActionValue`] is written into [`Action<C>`].

However, you might want to apply preprocessing first - for example, invert values, apply sensitivity, or remap axes. This is
where [input modifiers](crate::modifier) come in. They are components that implement the [`InputModifier`] trait and can
be attached to both actions and bindings. Binding-level modifiers are applied first, followed by action-level modifiers.
Use action-level modifiers as global modifiers that are applied to all bindings of the action.

```
use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;

# let mut world = World::new();
world.spawn((
    Player,
    actions!(Player[
        (
            Action::<Movement>::new(),
            // Modifier components at the action level.
            DeadZone::default(),    // Applies non-uniform normalization.
            SmoothNudge::default(), // Smoothes movement.
            bindings![
                // Keyboard keys captured as `bool`, but the output of `Movement` is defined as `Vec2`,
                // so you need to assign keys to axes using swizzle to reorder them and negation.
                (KeyCode::KeyW, SwizzleAxis::YXZ),
                (KeyCode::KeyA, Negate::all()),
                (KeyCode::KeyS, Negate::all(), SwizzleAxis::YXZ),
                KeyCode::KeyD,
                // In Bevy sticks split by axes and captured as 1-dimensional inputs,
                // so Y stick needs to be sweezled into Y axis.
                GamepadAxis::LeftStickX,
                (GamepadAxis::LeftStickY, SwizzleAxis::YXZ),
            ]
        ),
    ]),
));

#[derive(Component)]
struct Player;

#[derive(InputAction)]
#[action_output(Vec2)]
struct Movement;
```

### Presets

Some bindings are very common. It would be inconvenient to bind WASD keys and analog sticks manually, like in the example above,
every time. To solve this, we provide [presets](crate::preset) - structs that implement [`SpawnableList`] and store bindings that
will be spawned with predefined modifiers. To spawn them, you need to to call [`SpawnRelated::spawn`] implemented for [`Bindings`]
directly instead of the [`bindings!`] macro.

For example, you can use [`Cardinal`] and [`Axial`] presets to simplify the example above.

```
# use bevy::prelude::*;
# use bevy_enhanced_input::prelude::*;
# let mut world = World::new();
world.spawn((
    Player,
    actions!(Player[
        (
            Action::<Movement>::new(),
            DeadZone::default(),
            SmoothNudge::default(),
            Bindings::spawn((
                Cardinal::wasd_keys(),
                Axial::left_stick(),
            )),
        ),
    ]),
));
# #[derive(Component)]
# struct Player;
# #[derive(InputAction)]
# #[action_output(Vec2)]
# struct Movement;
```

You can also assign custom bindings or attach additional modifiers, see the [preset] module for more details.

## Input conditions

Instead of hardcoded states like "pressed" or "released", all actions use an abstract [`ActionState`] component
(which is a required component of [`Action<C>`]). Its meaning depends on the assigned [input conditions](crate::condition),
which determine when the action is triggered. This allows you to define flexible behaviors, such as "hold for 1 second".

Input conditions are components that implement [`InputCondition`] trait. Similar to modifiers, you can attach them to
both actions and bindings. They also evaluated during [`EnhancedInputSet::Update`] right after modifiers and update
[`ActionState`] on the associated action entity.

If no conditions are attached, the action behaves like with [`Down`] condition with a zero actuation threshold,
meaning it will trigger on any non-zero input value.

```
# use bevy::prelude::*;
# use bevy_enhanced_input::prelude::*;
# let mut world = World::new();
world.spawn((
    Player,
    actions!(Player[
        (
            // The action will trigger only if held for 1 second.
            Action::<Jump>::new(),
            Hold::new(1.0),
            bindings![KeyCode::Space, GamepadButton::South],
        ),
        (
            Action::<Fire>::new(),
            Pulse::new(0.5), // The action will trigger every 0.5 seconds while held.
            bindings![
                (GamepadButton::RightTrigger2, Down::new(0.3)), // Additionally the right trigger only counts if its value is greater than 0.3.
                MouseButton::Left,
            ]
        ),
    ])
));
# #[derive(Component)]
# struct Player;
# #[derive(InputAction)]
# #[action_output(bool)]
# struct Jump;
# #[derive(InputAction)]
# #[action_output(bool)]
# struct Fire;
```

## Mocking

You can also mock actions using the [`ActionMock`] component. When it's present on an action with [`ActionMock::enabled`], it will drive
the [`ActionState`] and [`ActionValue`] for the specified [`MockSpan`] duration. During this time, all bindings for this action will be ignored.
For more details, see the [`ActionMock`] documentation.

If you only need mocking, you can disable [`InputPlugin`](bevy::input::InputPlugin) entirely. However, `bevy_input` is a required dependency
because we use its input types.

## Reacting on actions

Up to this point, we've only defined actions and contexts but haven't reacted to them yet.
We provide both push-style (via observers) and pull-style (by checking components) APIs.

### Push-style

For most cases it's better to use observer API, especially for actions that trigger rarely. Don't worry about losing parallelism - running
a system has its own overhead, so for small logic, it's actually faster to execute it outside a system. Just avoid heavy logic in
action observers.

During [`EnhancedInputSet::Apply`], events are triggered based on transitions of [`ActionState`], such as
[`Started<A>`], [`Fired<A>`], and others, where `A` is your action type. This includes transitions between identical states.
For a full list of transition events, see the [`ActionEvents`] component documentation.
Just like with the [`ActionState`], their meaning depends on the assigned [input conditions](crate::condition),

The event target will be the entity with the context component, and the output type will match the action's [`InputAction::Output`].
Events also include additional data, such as timings and state. See the documentation for each [event type](crate::action::events)
for more details.

```
# use bevy::prelude::*;
# use bevy_enhanced_input::prelude::*;
# let mut app = App::new();
app.add_observer(apply_movement);

/// Apply movement when `Movement` action considered fired.
fn apply_movement(movement: On<Fire<Movement>>, mut players: Query<&mut Transform>) {
    // Read transform from the context entity.
    let mut transform = players.get_mut(movement.context).unwrap();

    // We defined the output of `Movement` as `Vec2`,
    // but since translation expects `Vec3`, we extend it to 3 axes.
    transform.translation += movement.value.extend(0.0);
}
# #[derive(InputAction)]
# #[action_output(Vec2)]
# struct Movement;
```

The event system is highly flexible. For example, you can use the [`Hold`] condition for an attack action, triggering strong attacks on
[`Complete`] events and regular attacks on [`Cancel`] events.

This approach can be mixed with the pull-style API if you need to access values of other actions in your observer.

### Pull-style

Sometimes you may want to access multiple actions at the same time. Or check an action state
during other gameplay logic. For cases like you can use pull-style API.

Since actions just entities, you can query [`Action<C>`] in a system to get the action value in a strongly typed form.
Alternatively, you can query [`ActionValue`] in its dynamically typed form.

To access the action state, use the [`ActionState`] component. State transitions from the last action evaluation are recorded
in the [`ActionEvents`] component, which lets you detect when an action has just started or stopped triggering.

Timing information provided via [`ActionTime`] component.

You can also use Bevy's change detection - these components marked as changed only if their values actually change.

For singleplayer games you can use `Single` for a convenient access:

```
# use bevy::prelude::*;
# use bevy_enhanced_input::prelude::*;
fn apply_input(
    jump_events: Single<&ActionEvents, With<Action<Jump>>>,
    movement: Single<&Action<Movement>>,
    mut player_transform: Single<&mut Transform, With<Player>>,
) {
    // Jumped this frame
    if jump_events.contains(ActionEvents::STARTED) {
        // User logic...
    }

    // We defined the output of `Movement` as `Vec2`,
    // but since translation expects `Vec3`, we extend it to 3 axes.
    player_transform.translation = movement.extend(0.0);
}
# #[derive(Component)]
# struct Player;
# #[derive(InputAction)]
# #[action_output(bool)]
# struct Jump;
# #[derive(InputAction)]
# #[action_output(Vec2)]
# struct Movement;
```

For games with multiple contexts you can query for specific action or
iterate over action contexts.

```
# use bevy::prelude::*;
# use bevy_enhanced_input::prelude::*;
fn multiplayer_movement(
    jumps: Query<&ActionEvents, With<Action<Jump>>>,
    movements: Query<&Action<Movement>>,
    mut players: Query<(&mut Transform, &Actions<Player>)>,
) {
    for (mut transform, actions) in &mut players {
        let Some(jump_events) = jumps.iter_many(actions).next() else {
            continue;
        };
        let Some(movement) = movements.iter_many(actions).next() else {
            continue;
        };

        // Jumped this frame
        if jump_events.contains(ActionEvents::STARTED) {
            // User logic...
        }

        // We defined the output of `Movement` as `Vec2`,
        // but since translation expects `Vec3`, we extend it to 3 axes.
        transform.translation = movement.extend(0.0);
    }
}
# #[derive(Component)]
# struct Player;
# #[derive(InputAction)]
# #[action_output(bool)]
# struct Jump;
# #[derive(InputAction)]
# #[action_output(Vec2)]
# struct Movement;
```

## Removing contexts

If you despawn an entity with its context, the actions and bindings will also be despawned.
However, if you only want to remove a context from an entity, you must remove the required components
**and** manually despawn its actions.

```
# use bevy::prelude::*;
# use bevy_enhanced_input::prelude::*;
# let mut world = World::new();
let mut player = world.spawn((
    OnFoot,
    actions!(OnFoot[
        (Action::<Jump>::new(), bindings![KeyCode::Space, GamepadButton::South]),
        (Action::<Fire>::new(), bindings![MouseButton::Left, GamepadButton::RightTrigger2]),
    ])
));

player
    .remove_with_requires::<OnFoot>()
    .despawn_related::<Actions<OnFoot>>();

assert_eq!(world.entities().len(), 1, "only the player entity should be left");
# #[derive(Component)]
# struct OnFoot;
# #[derive(InputAction)]
# #[action_output(bool)]
# struct Jump;
# #[derive(InputAction)]
# #[action_output(bool)]
# struct Fire;
```

Actions aren't despawned automatically via [`EntityWorldMut::remove_with_requires`], since Bevy doesn't automatically
despawn related entities when their relationship targets (like [`Actions<C>`]) are removed. For this reason, [`Actions<C>`]
is not a required component for `C`. See [#20252](https://github.com/bevyengine/bevy/issues/20252) for more details.

When an action is despawned, it automatically transitions its state to [`ActionState::None`] with [`ActionValue::zero`],
triggering the corresponding events. Depending on your use case, using [`ContextActivity`] might be more convenient than removal.

## Input and UI

Currently, we don't integrate `bevy_input_focus` directly. But we provide [`ActionSources`] resource
that could be used to prevents actions from triggering during UI interactions. See its docs for details.

# Troubleshooting

If you face any issue, try to enable logging to see what is going on.
To enable logging, you can temporarily set `RUST_LOG` environment variable to `bevy_enhanced_input=debug`
(or `bevy_enhanced_input=trace` for more noisy output) like this:

```bash
RUST_LOG=bevy_enhanced_input=debug cargo run
```

The exact method depends on the OS shell.

Alternatively you can configure `LogPlugin` to make it permanent.

[`SpawnableList`]: bevy::ecs::spawn::SpawnableList
*/

#![no_std]

extern crate alloc;

// Required for the derive macro to work within the crate.
extern crate self as bevy_enhanced_input;

pub mod action;
pub mod binding;
pub mod condition;
pub mod context;
pub mod modifier;
pub mod preset;

pub mod prelude {
    pub use super::{
        EnhancedInputPlugin, EnhancedInputSystems,
        action::{
            Accumulation, Action, ActionMock, ActionOutput, ActionSettings, ActionState,
            ActionTime, InputAction, MockSpan,
            events::*,
            relationship::{ActionOf, ActionSpawner, ActionSpawnerCommands, Actions},
            value::{ActionValue, ActionValueDim},
        },
        actions,
        binding::{
            Binding, InputModKeys,
            mod_keys::ModKeys,
            relationship::{
                BindingOf, BindingSpawner, BindingSpawnerCommands, Bindings, IntoBindingBundle,
            },
        },
        bindings,
        condition::{
            ConditionKind, InputCondition, block_by::*, chord::*, cooldown::*, down::*,
            fns::InputConditionAppExt, hold::*, hold_and_release::*, press::*, pulse::*,
            release::*, tap::*,
        },
        context::{
            ActionsQuery, ContextActivity, ContextPriority, GamepadDevice, InputContextAppExt,
            input_reader::ActionSources,
            time::{ContextTime, TimeKind},
        },
        modifier::{
            InputModifier, accumulate_by::*, clamp::*, dead_zone::*, delta_scale::*,
            exponential_curve::*, fns::InputModifierAppExt, linear_step::*, negate::*, scale::*,
            smooth_nudge::*, swizzle_axis::*,
        },
        preset::{WithBundle, axial::*, bidirectional::*, cardinal::*, ordinal::*, spatial::*},
    };
    #[expect(
        deprecated,
        reason = "Exporting deprecated aliases to improve migration"
    )]
    pub use super::{
        EnhancedInputSet,
        action::events::{Cancelled, Completed, Fired, Started},
        condition::{press::Pressed, release::Released},
    };
    pub use bevy_enhanced_input_macros::InputAction;
}

use bevy::{input::InputSystems, prelude::*};

use condition::fns::ConditionRegistry;
use context::{
    ContextRegistry,
    input_reader::{self, ConsumedInputs, PendingBindings},
};
use modifier::fns::ModifierRegistry;
use prelude::{Press, Release, *};

/// Initializes contexts and feeds inputs to them.
///
/// See also [`EnhancedInputSet`].
pub struct EnhancedInputPlugin;

impl Plugin for EnhancedInputPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ContextRegistry>()
            .init_resource::<ConsumedInputs>()
            .init_resource::<PendingBindings>()
            .init_resource::<ActionSources>()
            .init_resource::<ConditionRegistry>()
            .init_resource::<ModifierRegistry>()
            .add_input_condition::<BlockBy>()
            .add_input_condition::<Chord>()
            .add_input_condition::<Down>()
            .add_input_condition::<Hold>()
            .add_input_condition::<HoldAndRelease>()
            .add_input_condition::<Press>()
            .add_input_condition::<Pulse>()
            .add_input_condition::<Release>()
            .add_input_condition::<Tap>()
            .add_input_condition::<Cooldown>()
            .add_input_modifier::<AccumulateBy>()
            .add_input_modifier::<Clamp>()
            .add_input_modifier::<DeadZone>()
            .add_input_modifier::<DeltaScale>()
            .add_input_modifier::<ExponentialCurve>()
            .add_input_modifier::<LinearStep>()
            .add_input_modifier::<Negate>()
            .add_input_modifier::<Scale>()
            .add_input_modifier::<SmoothNudge>()
            .add_input_modifier::<SwizzleAxis>()
            .configure_sets(
                PreUpdate,
                (EnhancedInputSystems::Prepare, EnhancedInputSystems::Update)
                    .chain()
                    .after(InputSystems),
            )
            .add_systems(
                PreUpdate,
                input_reader::update_pending.in_set(EnhancedInputSystems::Prepare),
            );
    }

    fn finish(&self, app: &mut App) {
        let context = app
            .world_mut()
            .remove_resource::<ContextRegistry>()
            .expect("contexts registry should be inserted in `build`");

        let conditions = app
            .world_mut()
            .remove_resource::<ConditionRegistry>()
            .expect("conditions registry should be inserted in `build`");

        let modifiers = app
            .world_mut()
            .remove_resource::<ModifierRegistry>()
            .expect("conditions registry should be inserted in `build`");

        for contexts in &*context {
            contexts.setup(app, &conditions, &modifiers);
        }
    }
}

/// Outdated alias for [`EnhancedInputSystems`].
#[doc(hidden)]
#[deprecated(since = "0.19.0", note = "Use `EnhancedInputSystems` instead")]
pub type EnhancedInputSet = EnhancedInputSystems;

/// Label for the system that updates input context instances.
#[derive(Debug, PartialEq, Eq, Clone, Hash, SystemSet)]
pub enum EnhancedInputSystems {
    /// Updates list of pending inputs to ignore.
    ///
    /// Runs in [`PreUpdate`].
    Prepare,
    /// Updates the state of the input contexts from inputs and mocks.
    ///
    /// Executes in every schedule where a context is registered.
    Update,
    /// Applies the value from [`ActionValue`] to [`Action`] and triggers
    /// events evaluated from [`Self::Update`].
    ///
    /// Executes in every schedule where a context is registered.
    Apply,
}
