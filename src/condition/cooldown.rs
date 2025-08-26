use core::time::Duration;

use bevy::prelude::*;

use super::DEFAULT_ACTUATION;
use crate::prelude::*;

/// Returns [`ActionState::Fired`] when actuated, then [`ActionState::None`]
/// on subsequent actuations until the cooldown duration has elapsed.
#[derive(Component, Reflect, Debug, Clone)]
pub struct Cooldown {
    /// Trigger threshold.
    pub actuation: f32,

    /// The type of time used to advance the timer.
    pub time_kind: TimeKind,

    timer: Timer,

    actuated: bool,
}

impl Cooldown {
    /// Creates a new instance with the given cooldown time in seconds.
    #[must_use]
    pub fn new(duration: f32) -> Self {
        let mut timer = Timer::from_seconds(duration, TimerMode::Once);
        timer.tick(Duration::from_secs_f32(duration)); // Allow the first press to fire immediately.
        Self {
            actuation: DEFAULT_ACTUATION,
            time_kind: Default::default(),
            timer,
            actuated: false,
        }
    }

    #[must_use]
    pub fn with_actuation(mut self, actuation: f32) -> Self {
        self.actuation = actuation;
        self
    }

    #[must_use]
    pub fn with_time_kind(mut self, kind: TimeKind) -> Self {
        self.time_kind = kind;
        self
    }

    /// Returns the associated timer.
    #[must_use]
    pub fn timer(&self) -> &Timer {
        &self.timer
    }
}

impl InputCondition for Cooldown {
    fn evaluate(
        &mut self,
        _action: &ActionsQuery,
        time: &ContextTime,
        value: ActionValue,
    ) -> ActionState {
        let last_actuated = self.actuated;
        self.actuated = value.is_actuated(self.actuation);

        if !self.actuated {
            let finished_before = self.timer.finished();
            self.timer.tick(time.delta_kind(self.time_kind));

            // Start cooldown when the action stops actuating, but
            // only if it was already finished before this tick.
            // This avoids re-triggering cooldown when a hold started during cooldown
            // and is released exactly as the cooldown completes.
            if last_actuated && finished_before {
                self.timer.reset();
            }
        }

        if self.actuated && self.timer.finished() {
            ActionState::Fired
        } else {
            ActionState::None
        }
    }

    fn kind(&self) -> ConditionKind {
        ConditionKind::Implicit
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context;

    #[test]
    fn cooldown() {
        let (mut world, mut state) = context::init_world();
        let (time, actions) = state.get(&world);

        let mut condition = Cooldown::new(1.0);

        assert_eq!(
            condition.evaluate(&actions, &time, true.into()),
            ActionState::Fired,
            "should fire on the first actuation",
        );
        assert_eq!(
            condition.evaluate(&actions, &time, true.into()),
            ActionState::Fired,
            "should continue to fire while the input is actuated",
        );

        assert_eq!(
            condition.evaluate(&actions, &time, false.into()),
            ActionState::None,
        );

        assert_eq!(
            condition.evaluate(&actions, &time, true.into()),
            ActionState::None,
            "shouldn't fire due to cooldown"
        );

        world
            .resource_mut::<Time>()
            .advance_by(Duration::from_secs(1));
        let (time, actions) = state.get(&world);

        assert_eq!(
            condition.evaluate(&actions, &time, false.into()),
            ActionState::None,
            "should fire only when actuated"
        );

        assert_eq!(
            condition.evaluate(&actions, &time, true.into()),
            ActionState::Fired,
        );
    }
}
