/*!
Instead of hardcoded states like "pressed" or "released", all actions use an abstract [`ActionState`] component
(which is a required component of [`Action<C>`]). Its meaning depends on the assigned [input conditions](crate::condition),
which determine when the action is triggered. This allows you to define flexible behaviors, such as "hold for 1 second".

Input conditions are components that implement [`InputCondition`] trait. Similar to modifiers, you can attach them to
both actions and bindings. They also evaluated during [`EnhancedInputSet::Update`] right after modifiers in their insertion
order and update [`ActionState`] on the associated action entity.

If no conditions are attached, the action behaves like with [`Down`] condition with a zero actuation threshold,
meaning it will trigger on any non-zero input value.

# Examples

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

let mut world = World::new();
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
```
*/

pub mod block_by;
pub mod chord;
pub mod cooldown;
pub mod down;
pub mod fns;
pub mod hold;
pub mod hold_and_release;
pub mod press;
pub mod pulse;
pub mod release;
pub mod tap;

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
