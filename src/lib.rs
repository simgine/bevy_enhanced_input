/*!
A powerful observer-based input manager for Bevy.

The design of this crate is heavily inspired by
[Unreal Engine Enhanced Input](https://dev.epicgames.com/documentation/en-us/unreal-engine/enhanced-input-in-unreal-engine),
but adapted to Bevy's ECS architecture and idioms.

# Core Concepts

This crate introduces three main concepts:

- **Actions** represent something a player can do, like "Jump", "Movement", or "Open Menu". They are not tied to specific input.
- **Bindings** connect those actions to real input sources such as keyboard keys, mouse buttons, gamepad axes, etc.
- **Contexts** represent a certain input state the player can be in, such as "On foot" or "In car". They associate actions with
  entities and define when those actions are evaluated.

In short, actions are mapped to inputs via bindings, and contexts control which actions are active.

## [Actions](action)

Each action represents a different in-game behavior.
To create a new action, you will need to define a new struct and implement the [`InputAction`] trait for it,
typically using the provided [`InputAction`] derive macro.

The action's output type is defined by the [`InputAction::Output`] associated type,
which can be one of [`bool`], [`f32`], [`Vec2`], or [`Vec3`]. This type determines
the kind of value the action will produce when triggered.
For example, a "Jump" action might produce a `bool` indicating whether the jump button is pressed,
while a "Movement" action might produce a `Vec2` representing the direction and magnitude of movement input.

Actions are stored as entities with the [`Action<A>`] component, where `A` is your [`InputAction`] type.
These are associated to contexts via the [`ActionOf<C>`] relationship, where `C` is your context type,
and can be quickly bound to them using the [`actions!`] macro.

By default, when actions are triggered, they "consume" the input values from their bindings.
This means that if multiple actions are bound to the same input source (e.g., the same key),
the action that is evaluated first will take precedence, and the others will not receive the input value.
This behavior (and other action-specific configuration) can be further customized using the [`ActionSettings`] component.

## [Bindings](binding)

Bindings define how actions are triggered by input sources (e.g. mouse movement or keyboard buttons) that your player might press, like keyboard keys or gamepad buttons.
We provide support for a variety of input sources out of the box: see the [`Binding`] enum for a full list.

Bindings are represented by entities with the [`Binding`] component.
Bindings associated with actions via [`BindingOf`] relationship. Similar to [`actions!`],
we provide the [`bindings!`] macro to spawn related bindings.

By default, input is read from all connected gamepads. You can customize this by adding the [`GamepadDevice`] component to the
context entity.

## [Contexts](context)

Contexts define when actions are evaluated. They are associated with action entities via the [`Actions<C>`] relationship mentioned earlier.
Depending on your type of game, you may have a single global context
or multiple contexts for different gameplay states. For games with multiple entities driven by a single context it's
common to create a "controller" entity which applies the input to the desired entity.

Contexts are stored using regular components, commonly on an entity for which the input is associated (player, button, dialog, etc.).

Contexts can be activated or deactivated using the [`ContextActivity`] component.
By default, contexts are active when the component is present.
When active, all actions associated with the context are evaluated.

By default, contexts are evaluated in reverse spawn order, meaning the most recently spawned context is evaluated first.
This behavior can be controlled with [`ContextPriority`].
To register a component as an input context, you need to call [`InputContextAppExt::add_input_context`]. By default, contexts are
evaluated during [`PreUpdate`], but you can customize this by using [`InputContextAppExt::add_input_context_to`] instead.

## Putting it all together

Let's summarize how contexts, actions, and bindings relate to each other in the ECS world.

You start with an entity that has a context component, such as `Player` with `OnFoot` context.
This context is an ordinary component, registered as an input context using
[`App::add_input_context`](InputContextAppExt::add_input_context).

Then, for each of the actions that you might want the player to be able to take while "on foot",
you define a new action type `A` that implements the [`InputAction`] trait.
These actions are represented as entities with the [`Action<A>`] component,
and are associated with the context entity via the [`ActionOf<C>`] relationship,
spawned using the [`actions!`] macro.

Finally, for each action, you define one or more bindings that specify which input sources
will trigger the action. These bindings are represented as entities with the [`Binding`] component,
and are associated with the action entity via the [`BindingOf`] relationship,
spawned using the [`bindings!`] macro.

Here's a complete example that demonstrates these concepts in action.

```
use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;

#[derive(Component)]
struct Player;

#[derive(InputAction)]
#[action_output(bool)]
struct Jump;

#[derive(InputAction)]
#[action_output(bool)]
struct Fire;

let mut app = App::new();
app.add_plugins(EnhancedInputPlugin)
    .add_input_context::<Player>()
    .finish();

app.world_mut().spawn((
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
```

If we wanted to add a new context for when the player is in a vehicle, we would create a new context component `InVehicle`
and register it as an input context. We would then define new actions specific to vehicle control,
such as `Accelerate` and `Brake`, and associate them with the `InVehicle` context entity.

If we wanted to add another action to the `OnFoot` context, such as `Crouch`, we would define a new action type `Crouch`
and associate it with the `OnFoot` context for our player entity.

And if we wanted to add a new key binding for the `Jump` action, such as the "J" key,
we would simply add a new binding to the existing `Jump` action on our player entity.

These patterns make it easy to manage complex input schemes in a structured but flexible way,
and support complex scenarios like multiple players, different gameplay states, customizable controls,
and computer-controlled entities that take the same actions as players.

## More sophisticated input patterns

While we can bind actions directly to input sources like buttons, further customization or preprocessing is often needed.
Not all inputs are "buttonlike": we may want to only trigger an action when a button is held for a certain duration,
or when a joystick is moved beyond a certain threshold.

There are two main ways to achieve this: using input conditions and input modifiers:

- Input conditions define when an action is considered to be triggered based on the state of its bindings.
    - For example, the [`DeadZone`] modifier can be used to ignore small movements of a joystick.
    - When no input conditions are attached to an action or its bindings, the action behaves as if it has a [`Down`] condition with a zero actuation threshold.
- Input modifiers transform the raw input values from bindings before they are processed by the action.
    - For example, the [`Hold`] condition can be used to trigger an action only when a button is held down for a specified duration.
    - When no input modifiers are attached, the raw input value is passed through unchanged.

See the module docs for [input conditions](crate::condition) and [input modifiers](crate::modifier) for more details.

These complex input patterns can be tedious to set up manually, especially for common use cases like character movement.
To simplify this, we provide a number of [presets](crate::preset) that bundle common bindings and modifiers together.

## Reacting to actions

Up to this point, we've explained how to define actions and link them to users inputs,
but haven't explained how you might actually react to those actions in your game logic.

We provide two flavors of API for this: a push-style API based on observers and a pull-style API based on querying action components.

Most users find the push-style API more ergonomic and easier to reason about,
but the pull-style API can allow for more complex checks and interactions between the state of multiple actions.

Ultimately, the choice between these two approaches depends on your specific use case and preferences,
with performance playing a relatively minor role unless you have a very large number of acting entities or if you have a complex logic for your action reaction.

### Push-style: responding to action events

When an action is triggered, we can notify your game logic using Bevy's [`Event`] system.
These triggers are driven by changes (including transitions from a state to itself) in the action's [`ActionState`],
updated during [`EnhancedInputSystems::Apply`].

There are a number of different [action events](crate::action::events), but the most commonly used are:
- [`Start<A>`]: The action has started triggering (e.g. button pressed).
- [`Fire<A>`]: The action is currently triggering (e.g. button held).
- [`Complete<A>`]: The action has stopped triggering (e.g. button released after being held).

The exact meaning of each [action event](crate::action::events) depends on the attached [input conditions](crate::condition).
For example, with the [`Down`] condition, [`Fire<A>`] triggers when the user holds the button.
But with [`HoldAndRelease`] it will trigger when user releases the button after holding it for the specified amount of time.

These events are targeted at the entity with the context component,
and will include information about the input values based on the [`InputAction::Output`],
as well as additional metadata such as timing information.
See the documentation for each [event type](crate::action::events) for more details.

Each of these events can be observed using the [`On<E>`] system parameter in a Bevy observer,
responding to changes as they occur during command flushes.

```
use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;

let mut app = App::new();
app.add_observer(apply_movement);

#[derive(InputAction)]
#[action_output(Vec2)]
struct Movement;

/// Apply movement when `Movement` action considered fired.
fn apply_movement(movement: On<Fire<Movement>>, mut players: Query<&mut Transform>) {
    // Read transform from the context entity.
    let mut transform = players.get_mut(movement.context).unwrap();

    // We defined the output of `Movement` as `Vec2`,
    // but since translation expects `Vec3`, we extend it to 3 axes.
    transform.translation += movement.value.extend(0.0);
}
```

The event system is highly flexible. For example, you can use the [`Hold`] condition for an attack action, triggering strong attacks on
[`Complete`] events and regular attacks on [`Cancel`] events.

This approach can be mixed with the pull-style API if you need to access values of other actions in your observer.

### Pull-style: polling action state

Sometimes you may want to access multiple actions at the same time, or check an action state
during other gameplay logic. For cases like you can use pull-style API.

Since actions just entities, you can query [`Action<C>`] in a system to get the action value in a strongly typed form.
Alternatively, you can query [`ActionValue`] in its dynamically typed form.

To access the action state, use the [`ActionState`] component. State transitions from the last action evaluation are recorded
in the [`ActionEvents`] component, which lets you detect when an action has just started or stopped triggering.

Timing information provided via [`ActionTime`] component.

You can also use Bevy's change detection - these components marked as changed only if their values actually change.

For single-player games you can use [`Single`] for convenient access:

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
fn apply_input(
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

# Next steps

While this is enough to allow you to understand the examples and get started, there are a number of other useful features to learn about.
Each of these is complex to deserve their own section:

- [input modifiers](crate::modifier) for combining and transforming input values (e.g. applying dead zones or sensitivity or creating chords)
- [input conditions](crate::condition) for defining when actions are triggered (e.g. on press, release, hold, tap, etc.)
- [presets](crate::preset) for common bindings and modifiers (e.g. WASD keys and gamepad sticks for movement)
- [mocking](crate::action::ActionMock) for simulating input in tests, cutscenes or as part of replicated network state
- [the details of working with contexts](crate::context) (e.g. managing multiple players or gameplay states)

# Input and UI

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
#[cfg(feature = "bevy_state")]
pub mod state;

pub mod prelude {
    #[cfg(feature = "bevy_state")]
    pub use super::state::{ActiveInState, ActiveInStates, StateContextAppExt};
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
            ConditionKind, InputCondition, block_by::*, chord::*, combo::*, cooldown::*, down::*,
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
/// See also [`EnhancedInputSystems`].
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
            .add_input_condition::<Combo>()
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
