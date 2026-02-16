use bevy::prelude::*;

use super::DEFAULT_ACTUATION;
use crate::prelude::*;

/// Returns [`TriggerState::Fired`] when toggled on, [`TriggerState::None`] when toggled off.
///
/// When the input is pressed:
/// - If currently off, turns on and fires continuously every frame
/// - If currently on, turns off and stops firing
///
/// This is useful for modes that should persist until toggled off,
/// like entering a "select target" mode, toggling crouch, or any other
/// action that represents a persistent state rather than a momentary input.
///
/// ```
/// # use bevy::prelude::*;
/// # use bevy_enhanced_input::prelude::*;
/// # #[derive(Component)]
/// # struct Player;
/// # #[derive(InputAction)]
/// # #[action_output(bool)]
/// # struct SelectTarget;
/// # let mut world = World::new();
/// world.spawn((
///     Player,
///     actions!(Player[
///         (
///             Action::<SelectTarget>::new(),
///             Toggle::default(),
///             bindings![KeyCode::KeyF],
///         ),
///     ]),
/// ));
/// ```
#[derive(Component, Debug, Clone, Copy)]
#[cfg_attr(
    feature = "reflect",
    derive(Reflect),
    reflect(Clone, Component, Debug, Default)
)]
pub struct Toggle {
    /// Trigger threshold.
    pub actuation: f32,

    /// Whether we're currently in the "on" state.
    ///
    /// This can be directly mutated from game logic if you need to
    /// programmatically control the toggle state (e.g., force it off when
    /// certain conditions are met).
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
    ) -> TriggerState {
        let previously_actuated = self.actuated;
        self.actuated = value.is_actuated(self.actuation);

        if self.actuated && !previously_actuated {
            self.toggled = !self.toggled;
        }

        if self.toggled {
            TriggerState::Fired
        } else {
            TriggerState::None
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

        assert_eq!(
            condition.evaluate(&actions, &time, 0.0.into()),
            TriggerState::None
        );
        assert_eq!(
            condition.evaluate(&actions, &time, 1.0.into()),
            TriggerState::Fired,
            "should toggle on"
        );
        assert_eq!(
            condition.evaluate(&actions, &time, 1.0.into()),
            TriggerState::Fired,
            "should stay on while held"
        );
        assert_eq!(
            condition.evaluate(&actions, &time, 0.0.into()),
            TriggerState::Fired,
            "should stay on after release"
        );
        assert_eq!(
            condition.evaluate(&actions, &time, 1.0.into()),
            TriggerState::None,
            "should toggle off"
        );
        assert_eq!(
            condition.evaluate(&actions, &time, 0.0.into()),
            TriggerState::None,
        );
        assert_eq!(
            condition.evaluate(&actions, &time, 1.0.into()),
            TriggerState::Fired,
            "should toggle on again"
        );
    }

    #[test]
    fn actuation_threshold() {
        let (world, mut state) = context::init_world();
        let (time, actions) = state.get(&world);

        let mut condition = Toggle::new(0.7);

        assert_eq!(
            condition.evaluate(&actions, &time, 0.5.into()),
            TriggerState::None,
            "below threshold should not toggle"
        );
        assert_eq!(
            condition.evaluate(&actions, &time, 0.8.into()),
            TriggerState::Fired,
        );
    }
}
