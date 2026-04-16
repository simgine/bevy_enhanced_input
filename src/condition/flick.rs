use bevy::prelude::*;

use super::DEFAULT_ACTUATION;
use crate::prelude::*;

/// Returns [`TriggerState::Fired`] when the input transitions from a rest threshold
/// to an actuation threshold in a specified amount of time, and
/// [`TriggerState::Ongoing`] when between the rest and actuation threshold
/// and the flick timer hasn't finished
#[derive(Component, Debug, Clone)]
#[cfg_attr(feature = "reflect", derive(Reflect), reflect(Clone, Component, Debug))]
pub struct Flick {
    /// The threshold the input must reach to activate.
    pub actuation: f32,

    /// The threshold the input must exit to start the timer and the threshold
    /// the stick must enter to begin a flick.
    ///
    /// By default it's set to 0.2.
    pub rest_threshold: f32,

    /// The type of time used to advance the timer.
    pub time_kind: TimeKind,

    timer: Timer,

    fired: bool,
}

impl Flick {
    /// Creates a new instance where the input must be in between the enter actuation
    /// and exit actuation for less than `flick_time` in seconds in order to fire.
    #[must_use]
    pub fn new(flick_time: f32) -> Self {
        Self {
            actuation: DEFAULT_ACTUATION,
            rest_threshold: 0.2,
            time_kind: Default::default(),
            timer: Timer::from_seconds(flick_time, TimerMode::Once),
            fired: false,
        }
    }

    #[must_use]
    pub fn with_actuation(mut self, enter_actuation: f32) -> Self {
        self.actuation = enter_actuation;
        self
    }

    #[must_use]
    pub fn with_time_kind(mut self, kind: TimeKind) -> Self {
        self.time_kind = kind;
        self
    }

    #[must_use]
    pub fn with_rest_threshold(mut self, rest_threshold: f32) -> Self {
        self.rest_threshold = rest_threshold;
        self
    }
}

impl InputCondition for Flick {
    fn evaluate(
        &mut self,
        _actions: &ActionsQuery,
        time: &ContextTime,
        value: ActionValue,
    ) -> TriggerState {
        let exit_actuated = value.is_actuated(self.rest_threshold);
        let enter_actuated = value.is_actuated(self.actuation);

        if !exit_actuated {
            // In "dead zone". Reset to allow for another flick.
            self.fired = false;
            self.timer.reset();
            return TriggerState::None;
        }

        if !enter_actuated {
            // In "middle zone". Count up
            self.timer.tick(time.delta_kind(self.time_kind));
            return if self.timer.is_finished() {
                TriggerState::None
            } else {
                TriggerState::Ongoing
            };
        }

        let finished = self.timer.is_finished();

        if finished || self.fired {
            // Flick took too long
            return TriggerState::None;
        }

        // Only fire one time
        self.fired = true;
        TriggerState::Fired
    }
}

#[cfg(test)]
mod tests {
    use core::time::Duration;

    use super::*;
    use crate::context;

    #[test]
    fn flick() {
        let (mut world, mut state) = context::init_world();
        let mut condition = Flick::new(0.5).with_rest_threshold(0.4).with_actuation(0.6);

        // Run twice to check for reset.
        for _ in 1..2 {
            let (time, actions) = state.get(&world);
            assert_eq!(
                condition.evaluate(&actions, &time, 0.0.into()),
                TriggerState::None,
            );

            world
                .resource_mut::<Time<Real>>()
                .advance_by(Duration::from_secs_f32(0.1));
            let (time, actions) = state.get(&world);

            assert_eq!(
                condition.evaluate(&actions, &time, 0.3.into()),
                TriggerState::None,
            );

            world
                .resource_mut::<Time<Real>>()
                .advance_by(Duration::from_secs_f32(0.1));
            let (time, actions) = state.get(&world);

            assert_eq!(
                condition.evaluate(&actions, &time, 0.4.into()),
                TriggerState::Ongoing,
            );

            world
                .resource_mut::<Time<Real>>()
                .advance_by(Duration::from_secs_f32(0.1));
            let (time, actions) = state.get(&world);

            assert_eq!(
                condition.evaluate(&actions, &time, 0.6.into()),
                TriggerState::Fired,
            );

            world
                .resource_mut::<Time<Real>>()
                .advance_by(Duration::from_secs(1));
            let (time, actions) = state.get(&world);

            assert_eq!(
                condition.evaluate(&actions, &time, 0.6.into()),
                TriggerState::None,
            );
        }
    }

    #[test]
    fn timeout() {
        let (mut world, mut state) = context::init_world();
        let (time, actions) = state.get(&world);

        let mut condition = Flick::new(0.5).with_rest_threshold(0.4).with_actuation(0.6);

        assert_eq!(
            condition.evaluate(&actions, &time, 0.5.into()),
            TriggerState::Ongoing,
        );

        world
            .resource_mut::<Time<Real>>()
            .advance_by(Duration::from_secs(1));
        let (time, actions) = state.get(&world);

        assert_eq!(
            condition.evaluate(&actions, &time, 0.4.into()),
            TriggerState::None,
        );
    }
}
