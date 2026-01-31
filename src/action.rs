//! Actions represent high-level user intents, such as "Jump" or "Move Forward".
//!
//! Each action is represented by a new type that implements the [`InputAction`] trait.
//! The trait defines the output type of the action, via the [`InputAction::Output`] associated type.
//! Actions can output different types of values, such as `bool` for button-like actions
//! (e.g., "Jump"), `f32` for single-axis actions (e.g., "Zoom"), or `Vec2`/`Vec3` for multi-axis actions
//! (like "Movement").
//!
//! Actions belong to [contexts](crate::context) that group related actions together,
//! allowing you to enable and disable actions based on the current game state.
//!
//! They are spawned as entities with the [`Action<C>`] component, where `C` is the action type,
//! and related to the context entity via the [`ActionOf<C>`] relationship.
//! The [`actions!`] macro can be used to conveniently spawn multiple actions at once.
//!
//! In turn, actions have input mappings defined by [binding](crate::binding) entities,
//! which are related to the action entity via the [`BindingOf`] relationship.
//!
//! # Responding to actions
//!
//! When an action is evaluated, it produces various [action events](events) that indicate
//! changes in the action's state.
//! See the section on [push-style action handling](crate#push-style-responding-to-action-events)
//! in the library documentation for more details.
//!
//! Similarly, you can check the current state and value of an action at any time using the
//! [`Action<C>`], [`ActionState`], [`ActionValue`] and [`ActionTime`] components.
//! See the section on [pull-style action handling](crate#pull-style-polling-action-state)
//! in the library documentation for more details.
//!
//! # Configuring actions
//!
//! The behavior of actions can be customized using the [`ActionSettings`] component,
//! which allows you to define accumulation behavior, input consumption, and reset requirements.
//!
//! The behavior of actions can also be modified via
//! [modifiers](crate::modifier) that transform the action value during evaluation,
//! or by using [input conditions](crate::condition) to control when actions are triggered.
//!
//! # Manually firing actions
//!
//! In addition to responding to user input, you can also manually set the state and value of actions
//! using the [`ActionMock`] component or by directly modifying various components before [`EnhancedInputSystems`] are run.
//!
//! This is useful for simulating input during cutscenes,
//! testing, networked replication, AI-controlled players, game replays, or other scenarios where you want to control the action state directly.

pub mod events;
pub mod fns;
pub mod relationship;
pub mod value;

use core::{any, fmt::Debug, time::Duration};

use bevy::prelude::*;
#[cfg(feature = "serialize")]
use serde::{Deserialize, Serialize};

use crate::prelude::*;
use fns::ActionFns;

/// Component that represents a user action.
///
/// Entities with this component needs to be spawned with [`ActionOf<C>`]
/// relationship in order to be evaluated.
///
/// Holds value defined by [`InputAction::Output`].
///
/// See the required components for other data associated with the action
/// that can be accessed without static typing.
#[derive(Component, Deref, DerefMut)]
#[require(
    Name::new(any::type_name::<A>()),
    ActionFns::new::<A>(),
    ActionValue::zero(A::Output::DIM),
    ActionSettings,
    ActionState,
    ActionEvents,
    ActionTime,
)]
pub struct Action<A: InputAction>(A::Output);

impl<A: InputAction> Clone for Action<A> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<A: InputAction> Copy for Action<A> {}

impl<A: InputAction> Default for Action<A> {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<A: InputAction> PartialEq for Action<A> {
    fn eq(&self, other: &Self) -> bool {
        **self == **other
    }
}

impl<A: InputAction> Action<A> {
    pub fn new() -> Self {
        Self::default()
    }
}

/// A gameplay-related action which can be bound to inputs.
///
/// Used to statically define the type for [`Action<C>`] and [`events`].
///
/// To implement the trait you can use the [`InputAction`](bevy_enhanced_input_macros::InputAction)
/// derive to reduce boilerplate. Just specify `action_output` attribute with the type.
///
/// ```
/// # use bevy::prelude::*;
/// # use bevy_enhanced_input::prelude::*;
/// #[derive(InputAction)]
/// #[action_output(Vec2)]
/// struct Movement;
/// ```
pub trait InputAction: 'static {
    /// What type of value this action will output.
    ///
    /// - Use [`bool`] for button-like actions (e.g., `Jump`).
    /// - Use [`f32`] for single-axis actions (e.g., `Zoom`).
    /// - For multi-axis actions, like `Movement`, use [`Vec2`] or [`Vec3`].
    type Output: ActionOutput;
}

/// Type which can be used as [`InputAction::Output`].
pub trait ActionOutput:
    From<ActionValue> + Default + Send + Sync + Debug + Clone + Copy + PartialEq
{
    /// Dimension of this output.
    ///
    /// Used for [`ActionValue`] initialization.
    const DIM: ActionValueDim;
}

impl ActionOutput for bool {
    const DIM: ActionValueDim = ActionValueDim::Bool;
}

impl ActionOutput for f32 {
    const DIM: ActionValueDim = ActionValueDim::Axis1D;
}

impl ActionOutput for Vec2 {
    const DIM: ActionValueDim = ActionValueDim::Axis2D;
}

impl ActionOutput for Vec3 {
    const DIM: ActionValueDim = ActionValueDim::Axis3D;
}

/// Behavior configuration for [`Action<C>`].
#[derive(Component, Reflect, Default, Debug, Clone, Copy)]
#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serialize", reflect(Serialize, Deserialize))]
pub struct ActionSettings {
    /// Accumulation behavior.
    ///
    /// By default set to [`Accumulation::default`].
    pub accumulation: Accumulation,

    /// Require inputs to be inactive before the first activation and continue to consume them
    /// even after context removal or deactivation until inputs become inactive again.
    ///
    /// This way new instances won't react to currently held inputs until they are released.
    /// This prevents unintended behavior where switching or layering contexts using the same key
    /// could cause an immediate switch back, as buttons are rarely pressed for only a single frame.
    ///
    /// By default set to `false`.
    pub require_reset: bool,

    /// Specifies whether this action should swallow any [`Bindings`]
    /// bound to it or allow them to pass through to affect actions that evaluated later.
    ///
    /// Actions are ordered by the maximum number of [`ModKeys`] in their bindings.
    /// For example, an action with a `Ctrl + C` binding is evaluated before one with just
    /// a `C` binding. If actions have the same modifier count, they are ordered by their
    /// spawn order.
    ///
    /// Consuming is global and affect actions in all contexts. Importantly, this does
    /// **not** affect the underlying Bevy input - only the action evaluation logic is impacted.
    ///
    /// Inputs are consumed only when the action state is not equal to
    /// [`ActionState::None`].
    ///
    /// By default set to `false`.
    pub consume_input: bool,
}

/// Defines how [`ActionValue`] is calculated when multiple inputs are evaluated with the
/// same most significant [`ActionState`] (excluding [`ActionState::None`]).
///
/// Stored inside [`ActionSettings`].
#[derive(Reflect, Debug, Default, Clone, Copy)]
#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serialize", reflect(Serialize, Deserialize))]
pub enum Accumulation {
    /// Cumulatively add the key values for each mapping.
    ///
    /// For example, given values of 0.5 and -0.3, the input action's value would be 0.2.
    ///
    /// Usually used for things like WASD movement, when you want pressing W and S to cancel each other out.
    #[default]
    Cumulative,
    /// Take the value from the mapping with the highest absolute value.
    ///
    /// For example, given values of 0.5 and -1.5, the input action's value would be -1.5.
    MaxAbs,
}

/// State for [`Action<C>`].
///
/// Updated from [`Bindings`] and associated [`conditions`](crate::condition),
/// or overridden by [`ActionMock`] if present.
///
/// During evaluation, [`ActionEvents`] are derived from the previous and current state.
#[derive(Component, Reflect, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serialize", reflect(Serialize, Deserialize))]
pub enum ActionState {
    /// Condition is not triggered.
    #[default]
    None,
    /// Condition has started triggering, but has not yet finished.
    ///
    /// For example, with the [`Hold`] condition, this state is set while
    /// the key is held down, until the required duration is met.
    Ongoing,
    /// The condition has been met.
    ///
    /// For example, with the [`Down`] condition, this state remains active
    /// as long as the key is held down. If you want to respond only
    /// on the first or last frame this state is active, see [`ActionEvents::START`]
    /// or [`ActionEvents::COMPLETE`] respectively. For this condition,
    /// these correspond to "just pressed" and "just released".
    Fired,
}

/// Timing information for [`Action<C>`].
#[derive(Component, Reflect, Debug, Default, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serialize", reflect(Serialize, Deserialize))]
pub struct ActionTime {
    /// Time the action was in [`ActionState::Ongoing`] and [`ActionState::Fired`] states.
    pub elapsed_secs: f32,

    /// Time the action was in [`ActionState::Fired`] state.
    pub fired_secs: f32,
}

impl ActionTime {
    /// Updates the timers based on the given delta time and action state.
    pub fn update(&mut self, delta_secs: f32, state: ActionState) {
        match state {
            ActionState::None => {
                self.elapsed_secs = 0.0;
                self.fired_secs = 0.0;
            }
            ActionState::Ongoing => {
                self.elapsed_secs += delta_secs;
                self.fired_secs = 0.0;
            }
            ActionState::Fired => {
                self.elapsed_secs += delta_secs;
                self.fired_secs += delta_secs;
            }
        }
    }
}

/// Mocks the state and value of [`Action<C>`] for a specified span.
///
/// While active, input reading, conditions, and modifiers are skipped. Instead,
/// the action reports the provided state and value. All state transition events
/// (e.g., [`Start<A>`], [`Fire<A>`]) will still be triggered as usual.
///
/// Once the span expires, [`Self::enabled`] is set to `false`, and the action resumes
/// the regular evaluation. The component is not removed automatically, allowing you
/// to reuse it for future mocking.
///
/// Mocking does not take effect immediately - it is applied during the next context evaluation.
/// For more details, see the [evaluation](crate#evaluation) section in the quick start guide.
///
/// See also [`ExternallyMocked`](crate::context::ExternallyMocked) to manually control the action data.
///
/// If you only need mocking, you can disable [`InputPlugin`](bevy::input::InputPlugin) entirely.
/// However, `bevy_input` is a required dependency because we use its input types elsewhere in this crate.
///
/// # Examples
///
/// Spawn and move up for 2 seconds:
///
/// ```
/// # use core::time::Duration;
/// # use bevy::prelude::*;
/// # use bevy_enhanced_input::prelude::*;
/// # let mut world = World::new();
/// world.spawn((
///     Player,
///     actions!(Player[
///         (
///             Action::<Movement>::new(),
///             ActionMock::new(ActionState::Fired, Vec2::Y, Duration::from_secs(2)),
///             Bindings::spawn(Cardinal::wasd_keys()), // Bindings will be ignored while mocked.
///         ),
///     ]),
/// ));
/// # #[derive(Component)]
/// # struct Player;
/// # #[derive(InputAction)]
/// # #[action_output(Vec2)]
/// # struct Movement;
/// ```
///
/// Mock previously spawned jump action for the next frame:
///
/// ```
/// # use bevy::prelude::*;
/// # use bevy_enhanced_input::prelude::*;
/// fn mock_jump(mut commands: Commands, jump: Single<Entity, With<Action<Jump>>>) {
///     commands.entity(*jump).insert(ActionMock::once(ActionState::Fired, true));
/// }
/// # #[derive(InputAction)]
/// # #[action_output(bool)]
/// # struct Jump;
#[derive(Component, Reflect, Debug, Clone, Copy)]
#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serialize", reflect(Serialize, Deserialize))]
pub struct ActionMock {
    pub state: ActionState,
    pub value: ActionValue,
    pub span: MockSpan,
    pub enabled: bool,
}

impl ActionMock {
    /// Creates a new instance that will mock state and value only for a single context evaluation.
    #[must_use]
    pub fn once(state: ActionState, value: impl Into<ActionValue>) -> Self {
        Self::new(state, value, MockSpan::Updates(1))
    }

    /// Creates a new instance that will mock state and value for the given span.
    #[must_use]
    pub fn new(
        state: ActionState,
        value: impl Into<ActionValue>,
        span: impl Into<MockSpan>,
    ) -> Self {
        Self {
            state,
            value: value.into(),
            span: span.into(),
            enabled: true,
        }
    }
}

/// Specifies how long [`ActionMock`] should remain active.
#[derive(Reflect, Debug, Clone, Copy)]
#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serialize", reflect(Serialize, Deserialize))]
pub enum MockSpan {
    /// Active for a fixed number of context evaluations.
    Updates(u32),
    /// Active for a real-time [`Duration`].
    Duration(Duration),
    /// Remains active until [`ActionMock::enabled`] is manually set to `false`,
    /// or the [`ActionMock`] component is removed from the action entity.
    Manual,
}

impl From<Duration> for MockSpan {
    fn from(value: Duration) -> Self {
        Self::Duration(value)
    }
}
