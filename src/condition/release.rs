use bevy::prelude::*;

use super::DEFAULT_ACTUATION;
use crate::prelude::*;

/// Returns [`TriggerState::Ongoing`] when the input exceeds the actuation threshold and
/// [`TriggerState::Fired`] once when the input drops back below the actuation threshold.
///
/// Note that both `bevy::prelude::*` and `bevy_enhanced_input::prelude::*` export a type with this name.
/// To disambiguate, import `bevy_enhanced_input::prelude::{*, Release}`.
#[derive(Component, Reflect, Debug, Clone, Copy)]
pub struct Release {
    /// Trigger threshold.
    pub actuation: f32,
    actuated: bool,
}

impl Release {
    #[must_use]
    pub const fn new(actuation: f32) -> Self {
        Self {
            actuation,
            actuated: false,
        }
    }
}

impl Default for Release {
    fn default() -> Self {
        Self::new(DEFAULT_ACTUATION)
    }
}

impl InputCondition for Release {
    fn evaluate(
        &mut self,
        _actions: &ActionsQuery,
        _time: &ContextTime,
        value: ActionValue,
    ) -> TriggerState {
        let previously_actuated = self.actuated;
        self.actuated = value.is_actuated(self.actuation);

        if self.actuated {
            // Ongoing on hold.
            TriggerState::Ongoing
        } else if previously_actuated {
            // Fired on release.
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
    fn release() {
        let (world, mut state) = context::init_world();
        let (time, actions) = state.get(&world);

        let mut condition = Release::default();
        assert_eq!(
            condition.evaluate(&actions, &time, 0.0.into()),
            TriggerState::None
        );
        assert_eq!(
            condition.evaluate(&actions, &time, 1.0.into()),
            TriggerState::Ongoing
        );
        assert_eq!(
            condition.evaluate(&actions, &time, 0.0.into()),
            TriggerState::Fired
        );
    }
}
