pub mod block_by;
pub mod chord;
pub mod down;
pub mod fns;
pub mod hold;
pub mod hold_and_release;
pub mod press;
pub mod pulse;
pub mod release;
pub mod tap;
pub mod cooldown;

use core::fmt::Debug;

use crate::prelude::*;

/// Default actuation threshold for all conditions.
pub const DEFAULT_ACTUATION: f32 = 0.5;

/// Defines how input activates.
///
/// Conditions analyze the input, checking for minimum actuation values
/// and validating patterns like short taps, prolonged holds, or the typical "press"
/// or "release" events.
///
/// Can be attached both to bindings and actions.
///
/// If you create a custom condition, it needs to be registered using
/// [`InputConditionAppExt::add_input_condition`].
pub trait InputCondition: Debug {
    /// Returns calculates state.
    ///
    /// `actions` is a state of other actions within the currently evaluating context.
    fn evaluate(
        &mut self,
        actions: &ActionsQuery,
        time: &ContextTime,
        value: ActionValue,
    ) -> ActionState;

    /// Returns how the condition is combined with others.
    fn kind(&self) -> ConditionKind {
        ConditionKind::Explicit
    }
}

/// Determines how a condition contributes to the final [`ActionState`].
///
/// If no conditions are provided, the state will be set to [`ActionState::Fired`]
/// on any non-zero value, functioning similarly to a [`Down`] condition
/// with a zero actuation threshold.
///
/// For details about how actions are combined, see [`Actions`].
pub enum ConditionKind {
    /// The most significant [`ActionState`] from all explicit conditions will be the
    /// resulting state.
    Explicit,
    /// Like [`Self::Explicit`], but [`ActionState::Fired`] will be set only if all
    /// implicit conditions return it.
    ///
    /// Otherwise, the most significant state will be capped at [`ActionState::Ongoing`].
    Implicit,
    /// Any blocking condition that returns [`ActionState::None`] will override
    /// the state with [`ActionState::None`].
    ///
    /// Doesn't contribute to the state on its own.
    Blocker,
}
