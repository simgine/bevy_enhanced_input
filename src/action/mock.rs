//! Provides functionality for mocking actions. Mocking, in this context, means
//! activating an action without physically interacting with the input device.
//!
//! Could be useful for:
//!
//! - Automated testing.
//! - Applying inputs from AI systems.
//! - Driving cutscenes.
//! - Applying input over a network.

use core::{marker::PhantomData, time::Duration};

use bevy::prelude::*;

use crate::prelude::*;

/// Mocks the state and value of [`Action<C>`] for a specified span.
///
/// You can simply insert this component on the action entity or use [`MockCommandExt`] for a command-based API on the context entity.
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

impl Default for ActionMock {
    /// By default the component is initialized as disabled and holds some placeholder values.
    /// This is done to prevent archetype moves when the component is inserted as part of an action mock.
    /// The specific default values are unimportant, as a user is expected to either use [`MockCommandExt::mock`]
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
    /// Remains active until [`ActionMock::enabled`] is manually set to `false`,
    /// or the [`ActionMock`] component is removed from the action entity.
    Manual,
}

impl From<Duration> for MockSpan {
    fn from(value: Duration) -> Self {
        Self::Duration(value)
    }
}

/// Command used by [`MockCommandExt::mock`]. See that method for documentation.
pub struct MockCommand<T, U> {
    /// The action mock to use.
    pub action_mock: ActionMock,
    marker: PhantomData<(T, U)>,
}

impl<C: Component, A: InputAction + Send> EntityCommand<bevy::ecs::error::Result<()>>
    for MockCommand<C, A>
{
    fn apply(self, entity: EntityWorldMut) -> bevy::ecs::error::Result<()> {
        let context = entity.id();
        let world = entity.into_world_mut();

        let actions = world.get::<Actions<C>>(context).ok_or_else(|| {
            format!(
                "entity {} has no `{}`",
                context,
                ShortName::of::<Actions<C>>(),
            )
        })?;

        // Need to iterate over this immutably first
        // because otherwise we would have to borrow `World` mutably and immutably at the same time
        let action_entity = actions
            .iter()
            .find(|e| world.entity(*e).contains::<Action<A>>())
            .ok_or_else(|| {
                format!(
                    "entity {} has no `{}` in its `{}`",
                    context,
                    ShortName::of::<Action<A>>(),
                    ShortName::of::<Actions<C>>(),
                )
            })?;

        // Not an archetype move: `InputAction` requires `ActionMock`.
        world.entity_mut(action_entity).insert(self.action_mock);
        Ok(())
    }
}

/// Extension trait for [`EntityCommands`] that allows mocking actions.
pub trait MockCommandExt {
    /// Searches for an entity with [`Action<A>`] in [`Actions<C>`] and inserts [`ActionMock`] to it with the given values.
    ///
    /// Convenience method to avoid manually querying an [`ActionMock`].
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
    fn mock<C: Component, A: InputAction + Send>(
        &mut self,
        state: ActionState,
        value: impl Into<ActionValue>,
        span: impl Into<MockSpan>,
    ) -> &mut Self;

    /// Mocks an action for a single update. `C` is the action context, `A` is the [`InputAction`].
    /// Convenience method so we don't have to manually query an action's [`ActionMock`].
    ///
    /// # Examples
    ///
    /// Mocks a single press.
    ///
    /// ```
    /// # use bevy::prelude::*;
    /// # use bevy_enhanced_input::prelude::*;
    /// # let mut app = App::new();
    /// app.world_mut().spawn((
    ///     Player,
    ///     actions!(Player[Action::<PrimaryFire>::new(), bindings![MouseButton::Left]])
    /// ));
    ///
    /// fn mock_fire(mut commands: Commands, player: Single<Entity, With<Player>>) {
    ///     commands
    ///         .entity(player.into_inner())
    ///         .mock_once::<Player, PrimaryFire>(ActionState::Fired, true);
    /// }
    /// # #[derive(Component)]
    /// # struct Player;
    /// # #[derive(InputAction)]
    /// # #[action_output(bool)]
    /// # struct PrimaryFire;
    /// ```
    fn mock_once<C: Component, A: InputAction + Send>(
        &mut self,
        state: ActionState,
        value: impl Into<ActionValue>,
    ) -> &mut Self;
}

impl MockCommandExt for EntityCommands<'_> {
    fn mock<C: Component, A: InputAction + Send>(
        &mut self,
        state: ActionState,
        value: impl Into<ActionValue>,
        span: impl Into<MockSpan>,
    ) -> &mut Self {
        self.queue(MockCommand::<C, A> {
            action_mock: ActionMock::new(state, value, span),
            marker: PhantomData,
        })
    }

    fn mock_once<C: Component, A: InputAction + Send>(
        &mut self,
        state: ActionState,
        value: impl Into<ActionValue>,
    ) -> &mut Self {
        self.queue(MockCommand::<C, A> {
            action_mock: ActionMock::once(state, value),
            marker: PhantomData,
        })
    }
}
