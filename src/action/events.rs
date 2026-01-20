use core::fmt::{self, Debug, Formatter};

use bevy::prelude::*;
use bitflags::bitflags;
#[cfg(feature = "serialize")]
use serde::{Deserialize, Serialize};

use crate::prelude::*;

/// Bitset with events caused by state transitions of [`ActionState`].
///
/// During [`EnhancedInputSystems::Apply`], events that correspond to bitflags will be triggered.
///
/// You can use this component directly in systems or react on corresponding events in observers.
///
/// Table of state transitions:
///
/// | Last state                  | New state                | Events                    |
/// | --------------------------- | ------------------------ | ------------------------- |
/// | [`ActionState::None`]       | [`ActionState::None`]    | No events                 |
/// | [`ActionState::None`]       | [`ActionState::Ongoing`] | [`Start`] + [`Ongoing`] |
/// | [`ActionState::None`]       | [`ActionState::Fired`]   | [`Start`] + [`Fire`]   |
/// | [`ActionState::Ongoing`]    | [`ActionState::None`]    | [`Cancel`]              |
/// | [`ActionState::Ongoing`]    | [`ActionState::Ongoing`] | [`Ongoing`]               |
/// | [`ActionState::Ongoing`]    | [`ActionState::Fired`]   | [`Fire`]                 |
/// | [`ActionState::Fired`]      | [`ActionState::Fired`]   | [`Fire`]                 |
/// | [`ActionState::Fired`]      | [`ActionState::Ongoing`] | [`Ongoing`]               |
/// | [`ActionState::Fired`]      | [`ActionState::None`]    | [`Complete`]             |
///
/// The meaning of each kind depends on the assigned [`InputCondition`]s. The events are
/// triggered in the action evaluation order.
#[derive(Component, Reflect, Default, Debug, PartialEq, Eq, Clone, Copy)]
#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serialize", reflect(Serialize, Deserialize))]
pub struct ActionEvents(u8);

impl ActionEvents {
    #[deprecated(since = "0.23.0", note = "Replaced by `START`")]
    pub const STARTED: Self = Self::START;
    #[deprecated(since = "0.23.0", note = "Replaced by `FIRE`")]
    pub const FIRED: Self = Self::FIRE;
    #[deprecated(since = "0.23.0", note = "Replaced by `CANCEL`")]
    pub const CANCELLED: Self = Self::CANCEL;
    #[deprecated(since = "0.23.0", note = "Replaced by `COMPLETE`")]
    pub const COMPLETED: Self = Self::COMPLETE;
}

bitflags! {
    impl ActionEvents: u8 {
        /// Corresponds to [`Start`].
        const START = 0b00000001;
        /// Corresponds to [`Ongoing`].
        const ONGOING = 0b00000010;
        /// Corresponds to [`Fire`].
        const FIRE = 0b00000100;
        /// Corresponds to [`Cancel`].
        const CANCEL = 0b00001000;
        /// Corresponds to [`Complete`].
        const COMPLETE = 0b00010000;
    }
}

impl ActionEvents {
    /// Creates a new instance based on state transition.
    pub fn new(previous: ActionState, current: ActionState) -> ActionEvents {
        match (previous, current) {
            (ActionState::None, ActionState::None) => ActionEvents::empty(),
            (ActionState::None, ActionState::Ongoing) => {
                ActionEvents::START | ActionEvents::ONGOING
            }
            (ActionState::None, ActionState::Fired) => ActionEvents::START | ActionEvents::FIRE,
            (ActionState::Ongoing, ActionState::None) => ActionEvents::CANCEL,
            (ActionState::Ongoing, ActionState::Ongoing) => ActionEvents::ONGOING,
            (ActionState::Ongoing, ActionState::Fired) => ActionEvents::FIRE,
            (ActionState::Fired, ActionState::None) => ActionEvents::COMPLETE,
            (ActionState::Fired, ActionState::Ongoing) => ActionEvents::ONGOING,
            (ActionState::Fired, ActionState::Fired) => ActionEvents::FIRE,
        }
    }
}

/// Triggers when an action switches its state from [`ActionState::None`]
/// to [`ActionState::Fired`] or [`ActionState::Ongoing`].
///
/// Triggered before [`Fire`] and [`Ongoing`].
///
/// See [`ActionEvents`] for all transitions.
///
/// # Examples
///
/// Throw an item on the first frame when the button is pressed.
///
/// ```
/// # use bevy::prelude::*;
/// # use bevy_enhanced_input::prelude::*;
/// # let mut app = App::new();
/// app.add_observer(throw);
///
/// app.world_mut().spawn((
///     Player,
///     actions!(Player[Action::<Throw>::new(), bindings![KeyCode::KeyF]]),
/// ));
///
/// /// Triggered only once on the first press, similar to `just_pressed` in `bevy_input`.
/// ///
/// /// It will not trigger again until the key is released and pressed again.
/// fn throw(throw: On<Start<Throw>>, players: Query<(&Transform, &mut Health)>) {
///     // ...
/// }
/// # #[derive(Component)]
/// # struct Player;
/// # #[derive(Component)]
/// # struct Health;
/// # #[derive(InputAction)]
/// # #[action_output(bool)]
/// # struct Throw;
/// ```
#[derive(EntityEvent)]
pub struct Start<A: InputAction> {
    /// Entity with the context component on which this event was triggered.
    #[event_target]
    pub context: Entity,

    /// Action that triggered this event.
    pub action: Entity,

    /// Current action value.
    pub value: A::Output,

    /// Current action state.
    pub state: ActionState,
}

impl<A: InputAction> Debug for Start<A> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("Started")
            .field("value", &self.value)
            .field("state", &self.state)
            .finish()
    }
}

impl<A: InputAction> Clone for Start<A> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<A: InputAction> Copy for Start<A> {}

/// Triggers every frame when an action state is [`ActionState::Ongoing`].
///
///
/// Usually useful in combination with [`Complete`] to apply some
/// logic while the action condition is partially met, and additional
/// logic when the condition is fully met.
///
/// See [`ActionEvents`] for all transitions.
///
/// # Examples
///
/// Apply healing until the button is held down.
/// Can be paired with [`Complete`] to apply a bonus healing when the hold duration is met.
///
/// ```
/// # use bevy::prelude::*;
/// # use bevy_enhanced_input::prelude::*;
/// # let mut app = App::new();
/// app.add_observer(heal);
///
/// app.world_mut().spawn((
///     Player,
///     actions!(Player[
///         (
///             Action::<Heal>::new(),
///             Hold::new(2.0), // The action lasts for 2.0 seconds.
///             bindings![KeyCode::KeyH],
///         ),
///     ])
/// ));
///
/// /// Triggered continuously while the user is holding down the button,
/// /// until the specified duration is reached.
/// fn heal(heal: On<Ongoing<Heal>>, players: Query<&mut Health>) {
///     // ..
/// }
/// # #[derive(Component)]
/// # struct Player;
/// # #[derive(Component)]
/// # struct Health;
/// # #[derive(InputAction)]
/// # #[action_output(bool)]
/// # struct Heal;
/// ```
#[derive(EntityEvent)]
pub struct Ongoing<A: InputAction> {
    /// Entity with the context component on which this event was triggered.
    #[event_target]
    pub context: Entity,

    /// Action that triggered this event.
    pub action: Entity,

    /// Current action value.
    pub value: A::Output,

    /// Current action state.
    pub state: ActionState,

    /// Time that this action has been in [`ActionState::Ongoing`] state.
    pub elapsed_secs: f32,
}

impl<A: InputAction> Debug for Ongoing<A> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("Ongoing")
            .field("value", &self.value)
            .field("state", &self.state)
            .field("elapsed_secs", &self.elapsed_secs)
            .finish()
    }
}

impl<A: InputAction> Clone for Ongoing<A> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<A: InputAction> Copy for Ongoing<A> {}

/// Triggers every frame when an action state is [`ActionState::Fired`].
///
/// If you want to respond only on the first or last frame this state
/// is active, see [`Start`] or [`Complete`] respectively.
///
/// See [`ActionEvents`] for all transitions.
///
/// # Examples
///
/// Continuously fires while the button is held down.
///
/// ```
/// # use bevy::prelude::*;
/// # use bevy_enhanced_input::prelude::*;
/// # let mut app = App::new();
/// app.add_observer(primary_fire);
///
/// app.world_mut().spawn((
///     Player,
///     actions!(Player[Action::<PrimaryFire>::new(), bindings![MouseButton::Left]])
/// ));
///
/// /// Triggered every frame while the key is held down.
/// fn primary_fire(fire: On<Fire<PrimaryFire>>, players: Query<(&Transform, &mut Health)>) {
///     // ...
/// }
/// # #[derive(Component)]
/// # struct Player;
/// # #[derive(Component)]
/// # struct Health;
/// # #[derive(InputAction)]
/// # #[action_output(bool)]
/// # struct PrimaryFire;
/// ```
#[derive(EntityEvent)]
pub struct Fire<A: InputAction> {
    /// Entity with the context component on which this event was triggered.
    #[event_target]
    pub context: Entity,

    /// Action that triggered this event.
    pub action: Entity,

    /// Current action value.
    pub value: A::Output,

    /// Current action state.
    pub state: ActionState,

    /// Time that this action has been in [`ActionState::Fired`] state.
    pub fired_secs: f32,

    /// Total time this action has been in both [`ActionState::Ongoing`] and [`ActionState::Fired`].
    pub elapsed_secs: f32,
}

impl<A: InputAction> Debug for Fire<A> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("Fired")
            .field("value", &self.value)
            .field("state", &self.state)
            .field("fired_secs", &self.fired_secs)
            .field("elapsed_secs", &self.elapsed_secs)
            .finish()
    }
}

impl<A: InputAction> Clone for Fire<A> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<A: InputAction> Copy for Fire<A> {}

/// Triggers when action switches its state from [`ActionState::Ongoing`] to [`ActionState::None`].
///
/// Note that both `bevy::prelude::*` and `bevy_enhanced_input::prelude::*` export a type with this name.
/// To disambiguate, import `bevy_enhanced_input::prelude::{*, Cancel}`.
///
/// See [`ActionEvents`] for all transitions.
///
/// # Examples
///
/// Perform a weak attack when not holding the button enough with the [`Hold`] condition.
/// Can be paired with [`Complete`] to apply a strong attack when the hold duration is met.
///
///
/// ```
/// # use bevy::prelude::*;
/// # use bevy_enhanced_input::prelude::{*, Cancel};
/// # let mut app = App::new();
/// app.add_observer(weak_attack);
///
/// app.world_mut().spawn((
///     Player,
///     actions!(Player[
///         (
///             Action::<SecondaryAttack>::new(),
///             Hold::new(1.5), // User needs to hold the button for 1.5 seconds.
///             bindings![MouseButton::Left],
///         ),
///     ])
/// ));
///
/// /// Triggered if the user releases the key before 1.5 seconds.
/// fn weak_attack(attack: On<Cancel<SecondaryAttack>>, players: Query<(&Transform, &mut Health)>) {
///     // ...
/// }
/// # #[derive(Component)]
/// # struct Player;
/// # #[derive(Component)]
/// # struct Health;
/// # #[derive(InputAction)]
/// # #[action_output(bool)]
/// # struct SecondaryAttack;
/// ```
#[derive(EntityEvent)]
pub struct Cancel<A: InputAction> {
    /// Entity with the context component on which this event was triggered.
    #[event_target]
    pub context: Entity,

    /// Action that triggered this event.
    pub action: Entity,

    /// Current action value.
    pub value: A::Output,

    /// Current action state.
    pub state: ActionState,

    /// Time that this action has been in [`ActionState::Ongoing`] state.
    pub elapsed_secs: f32,
}

impl<A: InputAction> Debug for Cancel<A> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("Canceled")
            .field("value", &self.value)
            .field("state", &self.state)
            .field("elapsed_secs", &self.elapsed_secs)
            .finish()
    }
}

impl<A: InputAction> Clone for Cancel<A> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<A: InputAction> Copy for Cancel<A> {}

/// Triggers when action switches its state from [`ActionState::Fired`] to [`ActionState::None`].
///
/// See [`ActionEvents`] for all transitions.
///
/// # Examples
///
/// Perform a jump when the button is released.
///
/// ```
/// # use bevy::prelude::*;
/// # use bevy_enhanced_input::prelude::*;
/// # let mut app = App::new();
/// app.add_observer(jump);
///
/// app.world_mut().spawn((
///     Player,
///     actions!(Player[Action::<Jump>::new(), bindings![KeyCode::Space]]),
/// ));
///
/// /// Triggered only once when the user releases the key, similar to `just_released` in `bevy_input`.
/// fn jump(jump: On<Complete<Jump>>, players: Query<&mut Transform>) {
///     // ...
/// }
/// # #[derive(Component)]
/// # struct Player;
/// # #[derive(InputAction)]
/// # #[action_output(bool)]
/// # struct Jump;
/// ```
///
/// Perform a strong attack after holding the button enough with the [`Hold`] condition.
///
/// ```
/// # use bevy::prelude::*;
/// # use bevy_enhanced_input::prelude::*;
/// # let mut app = App::new();
/// app.add_observer(strong_attack);
///
/// app.world_mut().spawn((
///     Player,
///     actions!(Player[
///         (
///             Action::<SecondaryAttack>::new(),
///             Hold::new(1.5), // User needs to hold the button for 1.5 seconds.
///             bindings![MouseButton::Left],
///         ),
///     ])
/// ));
///
/// /// Triggered if the user releases the key before 1.5 seconds.
/// fn strong_attack(attack: On<Complete<SecondaryAttack>>, players: Query<(&Transform, &mut Health)>) {
///     // ...
/// }
/// # #[derive(Component)]
/// # struct Player;
/// # #[derive(Component)]
/// # struct Health;
/// # #[derive(InputAction)]
/// # #[action_output(bool)]
/// # struct SecondaryAttack;
/// ```
#[derive(EntityEvent)]
pub struct Complete<A: InputAction> {
    /// Entity with the context component on which this event was triggered.
    #[event_target]
    pub context: Entity,

    /// Action that triggered this event.
    pub action: Entity,

    /// Current action value.
    pub value: A::Output,

    /// Current action state.
    pub state: ActionState,

    /// Time that this action has been in [`ActionState::Fired`] state.
    pub fired_secs: f32,

    /// Total time this action has been in both [`ActionState::Ongoing`] and [`ActionState::Fired`].
    pub elapsed_secs: f32,
}

impl<A: InputAction> Debug for Complete<A> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("Completed")
            .field("value", &self.value)
            .field("state", &self.state)
            .field("fired_secs", &self.fired_secs)
            .field("elapsed_secs", &self.elapsed_secs)
            .finish()
    }
}

impl<A: InputAction> Clone for Complete<A> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<A: InputAction> Copy for Complete<A> {}
