use bevy::prelude::*;

use super::DEFAULT_ACTUATION;
use crate::prelude::*;

/// Like [`super::press::Down`] but returns [`TriggerState::Fired`] only once until the next actuation.
///
/// Holding the input will not cause further triggers.
///
/// Note that both `bevy::prelude::*` and `bevy_enhanced_input::prelude::*` export a type with this name.
/// To disambiguate, import `bevy_enhanced_input::prelude::{*, Press}`.
#[derive(Component, Debug, Clone, Copy)]
#[cfg_attr(
    feature = "reflect",
    derive(Reflect),
    reflect(Clone, Component, Debug, Default)
)]
pub struct Press {
    /// Trigger threshold.
    pub actuation: f32,
    actuated: bool,
}

impl Press {
    #[must_use]
    pub const fn new(actuation: f32) -> Self {
        Self {
            actuation,
            actuated: false,
        }
    }
}

impl Default for Press {
    fn default() -> Self {
        Self::new(DEFAULT_ACTUATION)
    }
}

impl InputCondition for Press {
    fn evaluate(
        &mut self,
        _actions: &ActionsQuery,
        _time: &ContextTime,
        value: ActionValue,
    ) -> TriggerState {
        let previously_actuated = self.actuated;
        self.actuated = value.is_actuated(self.actuation);

        if self.actuated && !previously_actuated {
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
    fn press() {
        let (world, mut state) = context::init_world();
        let (time, actions) = state.get(&world);

        let mut condition = Press::default();
        assert_eq!(
            condition.evaluate(&actions, &time, 0.0.into()),
            TriggerState::None
        );
        assert_eq!(
            condition.evaluate(&actions, &time, 1.0.into()),
            TriggerState::Fired,
        );
    }
}
