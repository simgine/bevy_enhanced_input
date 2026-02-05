use bevy::prelude::*;

use super::DEFAULT_ACTUATION;
use crate::prelude::*;

/// Returns [`ActionState::Fired`] when toggled on, [`ActionState::None`] when toggled off.
///
/// The toggle state flips each time the input transitions from non-actuated to actuated.
/// This is useful for actions that should remain "on" after the player triggers them,
/// and turn "off" when triggered again.
///
/// For example, if the player presses `F` you can use a toggle to make the `SelectingFireTarget`
/// action remain "on", and then when they press `F` again the action turns "off".
#[derive(Component, Reflect, Debug, Clone, Copy)]
pub struct Toggle {
    /// Trigger threshold.
    pub actuation: f32,
    /// Current toggle state (on/off).
    pub toggled: bool,
    actuated: bool,
}

impl Toggle {
    #[must_use]
    pub const fn new(actuation: f32) -> Self {
        Self {
            actuation,
            toggled: false,
            actuated: false,
        }
    }
}

impl Default for Toggle {
    fn default() -> Self {
        Self::new(DEFAULT_ACTUATION)
    }
}

impl InputCondition for Toggle {
    fn evaluate(
        &mut self,
        _actions: &ActionsQuery,
        _time: &ContextTime,
        value: ActionValue,
    ) -> ActionState {
        let previously_actuated = self.actuated;
        self.actuated = value.is_actuated(self.actuation);

        if self.actuated && !previously_actuated {
            self.toggled = !self.toggled;
        }

        if self.toggled {
            ActionState::Fired
        } else {
            ActionState::None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context;

    #[test]
    fn toggle() {
        let (world, mut state) = context::init_world();
        let (time, actions) = state.get(&world);

        let mut condition = Toggle::default();

        // Initially not actuated, should be None
        assert_eq!(
            condition.evaluate(&actions, &time, 0.0.into()),
            ActionState::None
        );

        // First actuation toggles on -> Fired
        assert_eq!(
            condition.evaluate(&actions, &time, 1.0.into()),
            ActionState::Fired,
        );

        // Continued actuation should maintain Fired
        assert_eq!(
            condition.evaluate(&actions, &time, 1.0.into()),
            ActionState::Fired,
        );

        // Release (still toggled on) should maintain Fired
        assert_eq!(
            condition.evaluate(&actions, &time, 0.0.into()),
            ActionState::Fired,
        );

        // Second actuation toggles off -> None
        assert_eq!(
            condition.evaluate(&actions, &time, 1.0.into()),
            ActionState::None,
        );

        // Continued non-actuation should maintain None
        assert_eq!(
            condition.evaluate(&actions, &time, 0.0.into()),
            ActionState::None,
        );

        // Third actuation toggles on again -> Fired
        assert_eq!(
            condition.evaluate(&actions, &time, 1.0.into()),
            ActionState::Fired,
        );
    }
}
