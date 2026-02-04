//! Provides functionality for mocking actions. Mocking, in this context, means
//! activating an action without physically interacting with the input device.
//!
//! Could be useful for:
//!
//! - Automated testing.
//! - Applying inputs from AI systems.
//! - Driving cutscenes.
//! - Applying input over a network.

use core::time::Duration;

use bevy::{ecs::error::warn, prelude::*};
#[cfg(feature = "serialize")]
use serde::{Deserialize, Serialize};

use crate::prelude::*;

/// Mocks the state and value of [`Action<C>`] for a specified span.
///
/// You can simply insert this component on the action entity or use either [`MockEntityWorldMutExt`] or [`MockEntityCommandsExt`]
/// for a command-based API on the context entity.
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
    /// Like [`ActionMock::new`], but uses [`MockSpan::once`] to mock an action for a single update.
    ///
    /// See also [`mock_once`].
    #[must_use]
    pub fn once(state: ActionState, value: impl Into<ActionValue>) -> Self {
        Self::new(state, value, MockSpan::once())
    }

    /// Creates a new instance that will mock state and value for the given span.
    ///
    /// See also [`mock`].
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

impl Default for ActionMock {
    /// Creates a new disabled instance with some placeholder values.
    ///
    /// This is done to prevent archetype moves when the component is inserted as part of an action mock.
    /// The specific default values are unimportant, as a user is expected to either use [`mock`]
    /// or manually replace the entire component with their own values.
    fn default() -> Self {
        Self {
            state: ActionState::None,
            value: ActionValue::Bool(false),
            span: MockSpan::Manual,
            enabled: false,
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
    /// Remains active until [`ActionMock::enabled`] is manually set to `false`.
    Manual,
}

impl MockSpan {
    /// Active for a single context evaluation.
    ///
    /// Shorthand for `MockSpan::Updates(1)`.
    #[inline]
    #[must_use]
    pub fn once() -> Self {
        Self::Updates(1)
    }
}

impl From<Duration> for MockSpan {
    fn from(value: Duration) -> Self {
        Self::Duration(value)
    }
}

/// Extension trait for [`EntityWorldMut`] that provides methods for mocking actions.
pub trait MockEntityWorldMutExt {
    /// Mocks action `A` for the context `C` on the entity.
    ///
    /// See [`MockEntityCommandsExt::mock`] for more details.
    fn mock<C: Component, A: InputAction>(
        self,
        state: ActionState,
        value: impl Into<ActionValue>,
        span: impl Into<MockSpan>,
    ) -> Result<()>;

    /// Like [`Self::mock`], but uses [`MockSpan::once`] to mock an action for a single update.
    ///
    /// See also [`MockEntityCommandsExt::mock_once`].
    fn mock_once<C: Component, A: InputAction>(
        self,
        state: ActionState,
        value: impl Into<ActionValue>,
    ) -> Result<()>;
}

impl MockEntityWorldMutExt for EntityWorldMut<'_> {
    fn mock<C: Component, A: InputAction>(
        self,
        state: ActionState,
        value: impl Into<ActionValue>,
        span: impl Into<MockSpan>,
    ) -> Result<()> {
        mock::<C, A>(state, value, span).apply(self)
    }

    fn mock_once<C: Component, A: InputAction>(
        self,
        state: ActionState,
        value: impl Into<ActionValue>,
    ) -> Result<()> {
        mock_once::<C, A>(state, value).apply(self)
    }
}

/// Extension trait for [`EntityCommands`] that provides methods for mocking actions.
pub trait MockEntityCommandsExt {
    /// Searches for an entity with [`Action<A>`] in [`Actions<C>`] and inserts [`ActionMock`] to it with the given values.
    ///
    /// Convenience method to avoid manually searching for an action on a context entity to insert [`ActionMock`].
    /// This will emit a warning if the entity does not exist, does not have [`Actions<C>`], or those actions do not contain an [`Action<A>`].
    ///
    /// See also [`MockEntityWorldMutExt::mock`].
    ///
    /// # Examples
    ///
    /// Mocks a 2 second long press.
    ///
    /// ```
    /// # use bevy::prelude::*;
    /// # use bevy_enhanced_input::prelude::*;
    /// # use core::time::Duration;
    /// # let mut app = App::new();
    /// app.world_mut().spawn((
    ///     Player,
    ///     actions!(Player[Action::<PrimaryFire>::new(), bindings![MouseButton::Left]])
    /// ));
    ///
    /// fn mock_fire(mut commands: Commands, player: Single<Entity, With<Player>>) {
    ///     commands
    ///         .entity(player.into_inner())
    ///         .mock::<Player, PrimaryFire>(ActionState::Fired, true, Duration::from_secs(2));
    /// }
    /// # #[derive(Component)]
    /// # struct Player;
    /// # #[derive(InputAction)]
    /// # #[action_output(bool)]
    /// # struct PrimaryFire;
    /// ```
    fn mock<C: Component, A: InputAction>(
        &mut self,
        state: ActionState,
        value: impl Into<ActionValue>,
        span: impl Into<MockSpan>,
    ) -> &mut Self;

    /// Like [`Self::mock`], but will not emit a warning in case of failure.
    fn try_mock<C: Component, A: InputAction>(
        &mut self,
        state: ActionState,
        value: impl Into<ActionValue>,
        span: impl Into<MockSpan>,
    ) -> &mut Self;

    /// Like [`Self::mock`], but uses [`MockSpan::once`] to mock an action for a single update.
    ///
    /// See also [`MockEntityWorldMutExt::mock_once`].
    fn mock_once<C: Component, A: InputAction>(
        &mut self,
        state: ActionState,
        value: impl Into<ActionValue>,
    ) -> &mut Self;

    /// Like [`Self::mock_once`], but will not emit a warning in case of failure.
    fn try_mock_once<C: Component, A: InputAction>(
        &mut self,
        state: ActionState,
        value: impl Into<ActionValue>,
    ) -> &mut Self;
}

impl MockEntityCommandsExt for EntityCommands<'_> {
    fn mock<C: Component, A: InputAction>(
        &mut self,
        state: ActionState,
        value: impl Into<ActionValue>,
        span: impl Into<MockSpan>,
    ) -> &mut Self {
        self.queue_handled(mock::<C, A>(state, value, span), warn)
    }

    fn try_mock<C: Component, A: InputAction>(
        &mut self,
        state: ActionState,
        value: impl Into<ActionValue>,
        span: impl Into<MockSpan>,
    ) -> &mut Self {
        self.queue_silenced(mock::<C, A>(state, value, span))
    }

    fn mock_once<C: Component, A: InputAction>(
        &mut self,
        state: ActionState,
        value: impl Into<ActionValue>,
    ) -> &mut Self {
        self.queue_handled(mock_once::<C, A>(state, value), warn)
    }

    fn try_mock_once<C: Component, A: InputAction>(
        &mut self,
        state: ActionState,
        value: impl Into<ActionValue>,
    ) -> &mut Self {
        self.queue_silenced(mock_once::<C, A>(state, value))
    }
}

/// Mocks action `A` for the context `C` on the entity.
///
/// See also [`MockEntityCommandsExt::mock_once`] and [`MockEntityWorldMutExt::mock_once`].
pub fn mock<C: Component, A: InputAction>(
    state: ActionState,
    value: impl Into<ActionValue>,
    span: impl Into<MockSpan>,
) -> impl EntityCommand<Result<()>> {
    let value = value.into();
    let span = span.into();
    move |entity: EntityWorldMut| -> Result<()> {
        let context = entity.id();
        let actions = entity.get::<Actions<C>>().ok_or_else(|| {
            format!(
                "entity {} has no `{}`",
                context,
                ShortName::of::<Actions<C>>(),
            )
        })?;

        // Need to iterate over this immutably first
        // because otherwise we would have to borrow `World` mutably and immutably at the same time
        let action = actions
            .iter()
            .find(|&a| entity.world().get::<Action<A>>(a).is_some())
            .ok_or_else(|| {
                format!(
                    "entity {} has no `{}` in its `{}`",
                    context,
                    ShortName::of::<Action<A>>(),
                    ShortName::of::<Actions<C>>(),
                )
            })?;

        // Not an archetype move: `Action` requires `ActionMock`.
        let world = entity.into_world_mut();
        world
            .entity_mut(action)
            .insert(ActionMock::new(state, value, span));

        Ok(())
    }
}

/// Like [`mock`], but uses [`MockSpan::once`] to mock an action for a single update.
///
/// See also [`MockEntityCommandsExt::mock_once`] and [`MockEntityWorldMutExt::mock_once`].
pub fn mock_once<C: Component, A: InputAction>(
    state: ActionState,
    value: impl Into<ActionValue>,
) -> impl EntityCommand<Result<()>> {
    mock::<C, A>(state, value, MockSpan::once())
}
