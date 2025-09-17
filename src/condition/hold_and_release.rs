use bevy::prelude::*;

use super::DEFAULT_ACTUATION;
use crate::prelude::*;

/// Returns [`ActionState::Ongoing`] when input becomes actuated and [`ActionState::Fired`]
/// when the input is released after having been actuated for the defined hold time.
///
/// Returns [`ActionState::None`] when the input stops being actuated earlier than the defined hold time.
#[derive(Component, Reflect, Debug, Clone)]
pub struct HoldAndRelease {
    /// Trigger threshold.
    pub actuation: f32,

    /// The type of time used to advance the timer.
    pub time_kind: TimeKind,

    timer: Timer,
}

impl HoldAndRelease {
    /// Creates a new instance with the given hold time in seconds.
    #[must_use]
    pub fn new(hold_time: f32) -> Self {
        Self {
            actuation: DEFAULT_ACTUATION,
            time_kind: Default::default(),
            timer: Timer::from_seconds(hold_time, TimerMode::Once),
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

impl InputCondition for HoldAndRelease {
    fn evaluate(
        &mut self,
        _actions: &ActionsQuery,
        time: &ContextTime,
        value: ActionValue,
    ) -> ActionState {
        self.timer.tick(time.delta_kind(self.time_kind));

        if value.is_actuated(self.actuation) {
            ActionState::Ongoing
        } else {
            let is_finished = self.timer.is_finished();
            self.timer.reset();

            // Trigger if we've passed the threshold and released.
            if is_finished {
                ActionState::Fired
            } else {
                ActionState::None
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use core::time::Duration;

    use super::*;
    use crate::context;

    #[test]
    fn hold_and_release() {
        let (mut world, mut state) = context::init_world();
        let (time, actions) = state.get(&world);

        let mut condition = HoldAndRelease::new(1.0);

        assert_eq!(
            condition.evaluate(&actions, &time, 1.0.into()),
            ActionState::Ongoing,
        );

        world
            .resource_mut::<Time>()
            .advance_by(Duration::from_secs(1));
        let (time, actions) = state.get(&world);

        assert_eq!(
            condition.evaluate(&actions, &time, 0.0.into()),
            ActionState::Fired
        );

        world.resource_mut::<Time>().advance_by(Duration::ZERO);
        let (time, actions) = state.get(&world);

        assert_eq!(
            condition.evaluate(&actions, &time, 1.0.into()),
            ActionState::Ongoing,
        );
        assert_eq!(
            condition.evaluate(&actions, &time, 0.0.into()),
            ActionState::None
        );
    }

    #[test]
    fn exact_time() {
        let mut condition = HoldAndRelease::new(1.0);
        let (mut world, mut state) = context::init_world();
        let (time, actions) = state.get(&world);

        assert_eq!(
            condition.evaluate(&actions, &time, 1.0.into()),
            ActionState::Ongoing,
        );

        world
            .resource_mut::<Time>()
            .advance_by(Duration::from_secs(1));
        let (time, actions) = state.get(&world);

        assert_eq!(
            condition.evaluate(&actions, &time, 1.0.into()),
            ActionState::Ongoing
        );

        world.resource_mut::<Time>().advance_by(Duration::ZERO);
        let (time, actions) = state.get(&world);

        assert_eq!(
            condition.evaluate(&actions, &time, 1.0.into()),
            ActionState::Ongoing,
        );
        assert_eq!(
            condition.evaluate(&actions, &time, 0.0.into()),
            ActionState::Fired
        );
        assert_eq!(
            condition.evaluate(&actions, &time, 0.0.into()),
            ActionState::None
        );
    }

    #[test]
    fn delayed() {
        let mut condition = HoldAndRelease::new(1.0);
        let (mut world, mut state) = context::init_world();
        let (time, actions) = state.get(&world);

        assert_eq!(
            condition.evaluate(&actions, &time, 1.0.into()),
            ActionState::Ongoing,
        );

        world
            .resource_mut::<Time>()
            .advance_by(Duration::from_secs(1));
        let (time, actions) = state.get(&world);

        assert_eq!(
            condition.evaluate(&actions, &time, 1.0.into()),
            ActionState::Ongoing
        );

        world
            .resource_mut::<Time>()
            .advance_by(Duration::from_nanos(1));
        let (time, actions) = state.get(&world);

        assert_eq!(
            condition.evaluate(&actions, &time, 1.0.into()),
            ActionState::Ongoing,
        );
        assert_eq!(
            condition.evaluate(&actions, &time, 0.0.into()),
            ActionState::Fired
        );
        assert_eq!(
            condition.evaluate(&actions, &time, 0.0.into()),
            ActionState::None
        );
    }
}
