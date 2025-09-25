use bevy::prelude::*;

use super::DEFAULT_ACTUATION;
use crate::prelude::*;

/// Returns [`ActionState::Ongoing`] when input becomes actuated and [`ActionState::Fired`]
/// on the defined time interval.
///
/// Note: [`Completed`] only fires when the repeat limit is reached or when input is released
/// immediately after being triggered. Otherwise, [`Cancel`] is fired when input is released.
#[derive(Component, Reflect, Debug, Clone)]
pub struct Pulse {
    /// Number of times the condition can be triggered (0 means no limit).
    pub trigger_limit: u32,

    /// Whether to trigger when the input first exceeds the actuation threshold or wait for the first interval.
    pub trigger_on_start: bool,

    /// Trigger threshold.
    pub actuation: f32,

    /// The type of time used to advance the timer.
    pub time_kind: TimeKind,

    timer: Timer,

    trigger_count: u32,

    /// Tracks if we're in an actuated state to detect the start.
    started_actuation: bool,
}

impl Pulse {
    /// Creates a new instance with the given interval in seconds.
    #[must_use]
    pub fn new(interval: f32) -> Self {
        Self {
            trigger_limit: 0,
            trigger_on_start: true,
            actuation: DEFAULT_ACTUATION,
            time_kind: Default::default(),
            timer: Timer::from_seconds(interval, TimerMode::Repeating),
            trigger_count: 0,
            started_actuation: false,
        }
    }

    #[must_use]
    pub fn with_trigger_limit(mut self, trigger_limit: u32) -> Self {
        self.trigger_limit = trigger_limit;
        self
    }

    #[must_use]
    pub fn trigger_on_start(mut self, trigger_on_start: bool) -> Self {
        self.trigger_on_start = trigger_on_start;
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

impl InputCondition for Pulse {
    fn evaluate(
        &mut self,
        _actions: &ActionsQuery,
        time: &ContextTime,
        value: ActionValue,
    ) -> ActionState {
        if value.is_actuated(self.actuation) {
            let mut should_fire = false;

            if !self.started_actuation {
                self.started_actuation = true;
                should_fire |= self.trigger_on_start;
            }

            self.timer.tick(time.delta_kind(self.time_kind));
            should_fire |= self.timer.just_finished();

            if self.trigger_limit == 0 || self.trigger_count < self.trigger_limit {
                if should_fire {
                    self.trigger_count += 1;
                    ActionState::Fired
                } else {
                    ActionState::Ongoing
                }
            } else {
                ActionState::None
            }
        } else {
            self.timer.reset();
            self.trigger_count = 0;
            self.started_actuation = false;
            ActionState::None
        }
    }
}

#[cfg(test)]
mod tests {
    use core::time::Duration;

    use super::*;
    use crate::context;

    #[test]
    fn pulse() {
        let (mut world, mut state) = context::init_world();
        let (time, actions) = state.get(&world);

        let mut condition = Pulse::new(1.0);

        assert_eq!(
            condition.evaluate(&actions, &time, 1.0.into()),
            ActionState::Fired,
        );

        world
            .resource_mut::<Time>()
            .advance_by(Duration::from_millis(500));
        let (time, actions) = state.get(&world);

        assert_eq!(
            condition.evaluate(&actions, &time, 1.0.into()),
            ActionState::Ongoing,
        );
        assert_eq!(
            condition.evaluate(&actions, &time, 1.0.into()),
            ActionState::Fired,
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
    fn fires_again_after_release() {
        let (mut world, mut state) = context::init_world();
        let (time, actions) = state.get(&world);

        let mut condition = Pulse::new(1.0);

        assert_eq!(
            condition.evaluate(&actions, &time, 1.0.into()),
            ActionState::Fired,
        );

        world
            .resource_mut::<Time>()
            .advance_by(Duration::from_millis(500));
        let (time, actions) = state.get(&world);

        assert_eq!(
            condition.evaluate(&actions, &time, 0.0.into()),
            ActionState::None,
        );
        assert_eq!(
            condition.evaluate(&actions, &time, 1.0.into()),
            ActionState::Fired,
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
    fn not_trigger_on_start() {
        let (world, mut state) = context::init_world();
        let (time, actions) = state.get(&world);

        let mut condition = Pulse::new(1.0).trigger_on_start(false);
        assert_eq!(
            condition.evaluate(&actions, &time, 1.0.into()),
            ActionState::Ongoing,
        );
    }

    #[test]
    fn trigger_limit() {
        let (world, mut state) = context::init_world();
        let (time, actions) = state.get(&world);

        let mut condition = Pulse::new(1.0).with_trigger_limit(1);
        assert_eq!(
            condition.evaluate(&actions, &time, 1.0.into()),
            ActionState::Fired,
        );
        assert_eq!(
            condition.evaluate(&actions, &time, 1.0.into()),
            ActionState::None
        );
    }
}
