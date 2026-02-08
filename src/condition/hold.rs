use bevy::prelude::*;

use super::DEFAULT_ACTUATION;
use crate::prelude::*;

/// Returns [`TriggerState::Ongoing`] when the input becomes actuated and
/// [`TriggerState::Fired`] when input remained actuated for the defined hold time.
///
/// Returns [`TriggerState::None`] when the input stops being actuated earlier than the defined hold time.
/// May optionally fire once, or repeatedly fire.
#[derive(Component, Reflect, Debug, Clone)]
pub struct Hold {
    /// Should this trigger fire only once, or fire every frame once the hold time threshold is met?
    pub one_shot: bool,

    /// Trigger threshold.
    pub actuation: f32,

    /// The type of time used to advance the timer.
    pub time_kind: TimeKind,

    timer: Timer,
}

impl Hold {
    /// Creates a new instance with the given hold time in seconds.
    #[must_use]
    pub fn new(hold_time: f32) -> Self {
        Self {
            one_shot: false,
            actuation: DEFAULT_ACTUATION,
            time_kind: Default::default(),
            timer: Timer::from_seconds(hold_time, TimerMode::Once),
        }
    }

    #[must_use]
    pub fn one_shot(mut self, one_shot: bool) -> Self {
        self.one_shot = one_shot;
        self
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

impl InputCondition for Hold {
    fn evaluate(
        &mut self,
        _actions: &ActionsQuery,
        time: &ContextTime,
        value: ActionValue,
    ) -> TriggerState {
        let actuated = value.is_actuated(self.actuation);
        if actuated {
            self.timer.tick(time.delta_kind(self.time_kind));
        } else {
            self.timer.reset();
        }

        if self.timer.is_finished() {
            if self.timer.just_finished() || !self.one_shot {
                TriggerState::Fired
            } else {
                TriggerState::None
            }
        } else if actuated {
            TriggerState::Ongoing
        } else {
            TriggerState::None
        }
    }
}

#[cfg(test)]
mod tests {
    use core::time::Duration;

    use super::*;
    use crate::context;

    #[test]
    fn hold() {
        let (mut world, mut state) = context::init_world();
        let (time, actions) = state.get(&world);

        let mut condition = Hold::new(1.0);

        assert_eq!(
            condition.evaluate(&actions, &time, 1.0.into()),
            TriggerState::Ongoing,
        );

        world
            .resource_mut::<Time<Real>>()
            .advance_by(Duration::from_secs(1));
        let (time, actions) = state.get(&world);

        assert_eq!(
            condition.evaluate(&actions, &time, 1.0.into()),
            TriggerState::Fired,
        );
        assert_eq!(
            condition.evaluate(&actions, &time, 1.0.into()),
            TriggerState::Fired,
        );
        assert_eq!(
            condition.evaluate(&actions, &time, 0.0.into()),
            TriggerState::None
        );

        world
            .resource_mut::<Time<Real>>()
            .advance_by(Duration::ZERO);
        let (time, actions) = state.get(&world);

        assert_eq!(
            condition.evaluate(&actions, &time, 1.0.into()),
            TriggerState::Ongoing,
        );
    }

    #[test]
    fn one_shot() {
        let (mut world, mut state) = context::init_world();
        world
            .resource_mut::<Time<Real>>()
            .advance_by(Duration::from_secs(1));
        let (time, actions) = state.get(&world);

        let mut condition = Hold::new(1.0).one_shot(true);
        assert_eq!(
            condition.evaluate(&actions, &time, 1.0.into()),
            TriggerState::Fired
        );
        assert_eq!(
            condition.evaluate(&actions, &time, 1.0.into()),
            TriggerState::None
        );
    }
}
