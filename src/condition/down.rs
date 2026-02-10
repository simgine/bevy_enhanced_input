use bevy::prelude::*;

use super::DEFAULT_ACTUATION;
use crate::prelude::*;

/// Returns [`TriggerState::Fired`] when the input exceeds the actuation threshold.
#[derive(Component, Debug, Clone, Copy)]
#[cfg_attr(
    feature = "reflect",
    derive(Reflect),
    reflect(Clone, Component, Debug, Default)
)]
pub struct Down {
    /// Trigger threshold.
    pub actuation: f32,
}

impl Down {
    #[must_use]
    pub const fn new(actuation: f32) -> Self {
        Self { actuation }
    }
}

impl Default for Down {
    fn default() -> Self {
        Self::new(DEFAULT_ACTUATION)
    }
}

impl InputCondition for Down {
    fn evaluate(
        &mut self,
        _actions: &ActionsQuery,
        _time: &ContextTime,
        value: ActionValue,
    ) -> TriggerState {
        if value.is_actuated(self.actuation) {
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
    fn down() {
        let (world, mut state) = context::init_world();
        let (time, actions) = state.get(&world);

        let mut condition = Down::new(1.0);
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
