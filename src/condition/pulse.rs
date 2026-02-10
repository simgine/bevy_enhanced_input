use core::time::Duration;

use bevy::prelude::*;

use super::DEFAULT_ACTUATION;
use crate::prelude::*;

/// Returns [`TriggerState::Ongoing`] when input becomes actuated and [`TriggerState::Fired`]
/// on the defined time interval.
///
/// Note: [`Complete`] only fires when the repeat limit is reached or when input is released
/// immediately after being triggered. Otherwise, [`Cancel`] is fired when input is released.
#[derive(Component, Debug, Clone)]
#[cfg_attr(feature = "reflect", derive(Reflect), reflect(Clone, Component, Debug))]
pub struct Pulse {
    /// Number of times the condition can be triggered (0 means no limit).
    pub trigger_limit: u32,

    /// Whether to trigger when the input first exceeds the actuation threshold or wait for the first interval.
    pub trigger_on_start: bool,

    /// Trigger threshold.
    pub actuation: f32,

    /// The type of time used to advance the timer.
    pub time_kind: TimeKind,

    /// Time in seconds that will be used instead of the [`Self::interval`] once.
    initial_delay: Option<f32>,

    /// Interval between pulses in seconds.
    interval: f32,

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
            initial_delay: None,
            interval,
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

    /// Sets a different pause before the first repeat.
    ///
    /// Further repeats will use the interval from [`Self::new`].
    ///
    /// For example, you could set a longer delay to simulate keyboard repeat:
    /// when you hold down a key, the first repeat takes longer to fire, and then
    /// it repeats at a faster, steady interval.
    #[must_use]
    pub fn with_initial_delay(mut self, initial_delay: f32) -> Self {
        self.initial_delay = Some(initial_delay);
        self.timer
            .set_duration(Duration::from_secs_f32(initial_delay));
        self
    }

    /// Returns the delay from [`Self::with_initial_delay`] if it was set.
    #[must_use]
    pub fn initial_delay(&self) -> Option<f32> {
        self.initial_delay
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
    ) -> TriggerState {
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
                    if self.initial_delay.is_some() && self.trigger_count >= 1 {
                        self.timer.reset();
                        self.timer
                            .set_duration(Duration::from_secs_f32(self.interval));
                    }
                    self.trigger_count += 1;
                    TriggerState::Fired
                } else {
                    TriggerState::Ongoing
                }
            } else {
                TriggerState::None
            }
        } else {
            if let Some(initial_delay) = self.initial_delay {
                self.timer
                    .set_duration(Duration::from_secs_f32(initial_delay));
            }
            self.timer.reset();
            self.trigger_count = 0;
            self.started_actuation = false;
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
    fn pulse() {
        let (mut world, mut state) = context::init_world();
        let (time, actions) = state.get(&world);

        let mut condition = Pulse::new(1.0);

        assert_eq!(
            condition.evaluate(&actions, &time, 1.0.into()),
            TriggerState::Fired,
        );

        world
            .resource_mut::<Time<Real>>()
            .advance_by(Duration::from_millis(500));
        let (time, actions) = state.get(&world);

        assert_eq!(
            condition.evaluate(&actions, &time, 1.0.into()),
            TriggerState::Ongoing,
        );
        assert_eq!(
            condition.evaluate(&actions, &time, 1.0.into()),
            TriggerState::Fired,
        );

        world
            .resource_mut::<Time<Real>>()
            .advance_by(Duration::ZERO);
        let (time, actions) = state.get(&world);

        assert_eq!(
            condition.evaluate(&actions, &time, 1.0.into()),
            TriggerState::Ongoing,
        );
        assert_eq!(
            condition.evaluate(&actions, &time, 0.0.into()),
            TriggerState::None
        );
    }

    #[test]
    fn fires_again_after_release() {
        let (mut world, mut state) = context::init_world();
        let (time, actions) = state.get(&world);

        let mut condition = Pulse::new(1.0);

        assert_eq!(
            condition.evaluate(&actions, &time, 1.0.into()),
            TriggerState::Fired,
        );

        world
            .resource_mut::<Time<Real>>()
            .advance_by(Duration::from_millis(500));
        let (time, actions) = state.get(&world);

        assert_eq!(
            condition.evaluate(&actions, &time, 0.0.into()),
            TriggerState::None,
        );
        assert_eq!(
            condition.evaluate(&actions, &time, 1.0.into()),
            TriggerState::Fired,
        );

        world
            .resource_mut::<Time<Real>>()
            .advance_by(Duration::ZERO);
        let (time, actions) = state.get(&world);

        assert_eq!(
            condition.evaluate(&actions, &time, 1.0.into()),
            TriggerState::Ongoing,
        );
        assert_eq!(
            condition.evaluate(&actions, &time, 0.0.into()),
            TriggerState::None
        );
    }

    #[test]
    fn not_trigger_on_start() {
        let (world, mut state) = context::init_world();
        let (time, actions) = state.get(&world);

        let mut condition = Pulse::new(1.0).trigger_on_start(false);
        assert_eq!(
            condition.evaluate(&actions, &time, 1.0.into()),
            TriggerState::Ongoing,
        );
    }

    #[test]
    fn initial_delay() {
        let (mut world, mut state) = context::init_world();
        let mut condition = Pulse::new(0.35).with_initial_delay(1.0);
        let (time, actions) = state.get(&world);
        assert_eq!(
            condition.evaluate(&actions, &time, 1.0.into()),
            TriggerState::Fired,
        );

        world
            .resource_mut::<Time<Real>>()
            .advance_by(Duration::from_millis(500));
        let (time, actions) = state.get(&world);
        assert_eq!(
            condition.evaluate(&actions, &time, 1.0.into()),
            TriggerState::Ongoing,
        );

        world
            .resource_mut::<Time<Real>>()
            .advance_by(Duration::from_millis(500));
        let (time, actions) = state.get(&world);
        assert_eq!(
            condition.evaluate(&actions, &time, 1.0.into()),
            TriggerState::Fired,
            "should fire after initial delay",
        );

        world
            .resource_mut::<Time<Real>>()
            .advance_by(Duration::from_millis(300));
        let (time, actions) = state.get(&world);
        assert_eq!(
            condition.evaluate(&actions, &time, 1.0.into()),
            TriggerState::Ongoing,
        );

        world
            .resource_mut::<Time<Real>>()
            .advance_by(Duration::from_millis(50));
        let (time, actions) = state.get(&world);
        assert_eq!(
            condition.evaluate(&actions, &time, 1.0.into()),
            TriggerState::Fired,
            "should fire after regular interval",
        );

        let (time, actions) = state.get(&world);
        assert_eq!(
            condition.evaluate(&actions, &time, 0.0.into()),
            TriggerState::None,
        );
        assert_eq!(condition.timer().duration().as_secs_f32(), 1.0);
    }

    #[test]
    fn trigger_limit() {
        let (world, mut state) = context::init_world();
        let (time, actions) = state.get(&world);

        let mut condition = Pulse::new(1.0).with_trigger_limit(1);
        assert_eq!(
            condition.evaluate(&actions, &time, 1.0.into()),
            TriggerState::Fired,
        );
        assert_eq!(
            condition.evaluate(&actions, &time, 1.0.into()),
            TriggerState::None
        );
    }
}
