use bevy::prelude::*;

use super::DEFAULT_ACTUATION;
use crate::prelude::*;

/// Returns [`TriggerState::Fired`] when the input transitions from a rest threshold
/// to an actuation threshold in an amount of time set by the user, then
/// [`TriggerState::Ongoing`] until the actuation threshold is exited.
#[derive(Component, Debug, Clone)]
#[cfg_attr(feature = "reflect", derive(Reflect), reflect(Clone, Component, Debug))]
pub struct Flick {
    /// The threshold the input must reach to activate
    pub actuation: f32,

    /// The threshold the input must exit to start the timer and the threshold
    /// the stick must enter to begin a flick
    pub max_rest_threshold: f32,

    /// The type of time used to advance the timer.
    pub time_kind: TimeKind,

    timer: Timer,

    fired: bool,
}

impl Flick {
    const DEFAULT_EXIT_ACTUATION: f32 = 0.3;

    /// Creates a new instance where the input must be in between the enter actuation
    /// and exit actuation for less than `flick_time` in seconds in order to fire.
    #[must_use]
    pub fn new(flick_time: f32) -> Self {
        Self {
            actuation: DEFAULT_ACTUATION,
            max_rest_threshold: Self::DEFAULT_EXIT_ACTUATION,
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
}

impl InputCondition for Flick {
    fn evaluate(
        &mut self,
        _actions: &ActionsQuery,
        time: &ContextTime,
        value: ActionValue,
    ) -> TriggerState {
        let exit_actuated = value.is_actuated(self.max_rest_threshold);
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
            return TriggerState::None;
        }

        // Reach this line only if both exit and enter regions are actuated
        let finished = self.timer.is_finished();

        if finished {
            // Flick took too long
            return TriggerState::None;
        }

        if !self.fired {
            // Only fire one time
            self.fired = true;
            TriggerState::Fired
        } else {
            // Ongoing until we exit the enter region
            TriggerState::Ongoing
        }
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
        let (time, actions) = state.get(&world);

        let mut condition = Flick::new(0.5).with_actuation(0.9);

        // Check successful flick
        assert_eq!(
            condition.evaluate(&actions, &time, 0.0.into()),
            TriggerState::None,
        );

        assert_eq!(
            condition.evaluate(&actions, &time, 0.4.into()),
            TriggerState::None,
        );

        world
            .resource_mut::<Time<Real>>()
            .advance_by(Duration::from_secs_f32(0.25));
        let (time, actions) = state.get(&world);

        assert_eq!(
            condition.evaluate(&actions, &time, 0.4.into()),
            TriggerState::None,
        );

        assert_eq!(
            condition.evaluate(&actions, &time, 1.0.into()),
            TriggerState::Fired,
        );

        // Check to see that Ongoing still continues
        world
            .resource_mut::<Time<Real>>()
            .advance_by(Duration::from_secs(1));
        let (time, actions) = state.get(&world);

        assert_eq!(
            condition.evaluate(&actions, &time, 1.0.into()),
            TriggerState::Ongoing,
        );

        // Check a flick that takes too long
        assert_eq!(
            condition.evaluate(&actions, &time, 0.0.into()),
            TriggerState::None,
        );

        assert_eq!(
            condition.evaluate(&actions, &time, 0.4.into()),
            TriggerState::None,
        );

        world
            .resource_mut::<Time<Real>>()
            .advance_by(Duration::from_secs(1));
        let (time, actions) = state.get(&world);

        assert_eq!(
            condition.evaluate(&actions, &time, 1.0.into()),
            TriggerState::None,
        );

        world
            .resource_mut::<Time<Real>>()
            .advance_by(Duration::from_secs_f32(0.1));
        let (time, actions) = state.get(&world);

        // Check successful flick again to ensure it reset properly
        assert_eq!(
            condition.evaluate(&actions, &time, 0.0.into()),
            TriggerState::None,
        );

        assert_eq!(
            condition.evaluate(&actions, &time, 0.4.into()),
            TriggerState::None,
        );

        world
            .resource_mut::<Time<Real>>()
            .advance_by(Duration::from_secs_f32(0.25));
        let (time, actions) = state.get(&world);

        assert_eq!(
            condition.evaluate(&actions, &time, 0.4.into()),
            TriggerState::None,
        );

        assert_eq!(
            condition.evaluate(&actions, &time, 1.0.into()),
            TriggerState::Fired,
        );
    }
}
