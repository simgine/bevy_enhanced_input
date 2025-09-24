use bevy::prelude::*;

use super::DEFAULT_ACTUATION;
use crate::prelude::*;

/// Like [`super::press::Down`] but returns [`ActionState::Fired`] only once until the next actuation.
///
/// Holding the input will not cause further triggers.
#[derive(Component, Reflect, Debug, Clone, Copy)]
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
    ) -> ActionState {
        let previously_actuated = self.actuated;
        self.actuated = value.is_actuated(self.actuation);

        if self.actuated && !previously_actuated {
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
    fn press() {
        let (world, mut state) = context::init_world();
        let (time, actions) = state.get(&world);

        let mut condition = Press::default();
        assert_eq!(
            condition.evaluate(&actions, &time, 0.0.into()),
            ActionState::None
        );
        assert_eq!(
            condition.evaluate(&actions, &time, 1.0.into()),
            ActionState::Fired,
        );
    }
}
