use core::time::Duration;
use bevy::prelude::*;

use super::DEFAULT_ACTUATION;
use crate::prelude::*;

/// Returns [`ActionState::Fired`] when input is being pressed after a set duration passed or for the first time.
///
/// Returns [`ActionState::None`] when the action timer is not finished or already actuated last frame.
#[derive(Component, Reflect, Debug, Clone)]
pub struct Cooldown {
    /// Trigger threshold.
    pub actuation: f32,
    
    actuated: bool,

    /// The type of time used to advance the timer.
    pub time_kind: TimeKind,

    timer: Timer,
}

impl Cooldown {
    #[must_use]
    pub fn new(cd: f32) -> Self {
        let mut timer = Timer::from_seconds(cd, TimerMode::Once);
        // Allow the first press to fire immediately; cooldown gates later presses
        timer.set_elapsed(Duration::from_secs_f32(cd));
        Self {
            timer,
            actuated: false,
            actuation: DEFAULT_ACTUATION,
            time_kind: Default::default(),
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
}

impl InputCondition for Cooldown {
    fn evaluate(
        &mut self,
        _action: &ActionsQuery,
        time: &ContextTime,
        value: ActionValue,
    ) -> ActionState {
        self.timer.tick(time.delta_kind(self.time_kind));
        let last_actuated = self.actuated;
        self.actuated = value.is_actuated(self.actuation);

        if self.actuated             // Input is currently pressed
            && !last_actuated        // Input was NOT pressed last frame
            && self.timer.finished() // Cooldown has expired
        {
            self.timer.reset();
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
    use core::time::Duration;

    #[test]
    fn cooldown_basic() {
        let (mut world, mut state) = context::init_world();
        let (time, actions) = state.get(&world);

        let mut condition = Cooldown::new(1.0);

        // The first press should fire immediately (timer initially finished)
        assert_eq!(
            condition.evaluate(&actions, &time, 1.0.into()),
            ActionState::Fired,
        );

        // Holding should not fire again
        let (time, actions) = state.get(&world);
        assert_eq!(
            condition.evaluate(&actions, &time, 1.0.into()),
            ActionState::None,
        );

        // Release
        let (time, actions) = state.get(&world);
        assert_eq!(
            condition.evaluate(&actions, &time, 0.0.into()),
            ActionState::None,
        );

        // Immediate re-press before cooldown finishes should not fire
        let (time, actions) = state.get(&world);
        assert_eq!(
            condition.evaluate(&actions, &time, 1.0.into()),
            ActionState::None,
        );

        // Advance exactly the cooldown duration
        world
            .resource_mut::<Time>()
            .advance_by(Duration::from_secs(1));

        // Consume the elapsed time on a non-pressed frame to keep rising edge for the next press
        let (time, actions) = state.get(&world);
        assert_eq!(
            condition.evaluate(&actions, &time, 0.0.into()),
            ActionState::None,
        );

        // Now pressing should fire again
        let (time, actions) = state.get(&world);
        assert_eq!(
            condition.evaluate(&actions, &time, 1.0.into()),
            ActionState::Fired,
        );
    }

    #[test]
    fn exact_time() {
        let (mut world, mut state) = context::init_world();
        let (time, actions) = state.get(&world);

        let mut condition = Cooldown::new(1.0);

        // Initial fire
        assert_eq!(
            condition.evaluate(&actions, &time, 1.0.into()),
            ActionState::Fired,
        );

        // Release to allow rising edge next time
        let (time, actions) = state.get(&world);
        assert_eq!(
            condition.evaluate(&actions, &time, 0.0.into()),
            ActionState::None,
        );

        // Advance exactly 1 second
        world
            .resource_mut::<Time>()
            .advance_by(Duration::from_secs(1));

        // Keep unpressed to preserve the rising edge and tick the timer to finished
        let (time, actions) = state.get(&world);
        assert_eq!(
            condition.evaluate(&actions, &time, 0.0.into()),
            ActionState::None,
        );

        // Now press should fire exactly at the boundary
        let (time, actions) = state.get(&world);
        assert_eq!(
            condition.evaluate(&actions, &time, 1.0.into()),
            ActionState::Fired,
        );

        // Further frames without a cooldown won't fire
        let (time, actions) = state.get(&world);
        assert_eq!(
            condition.evaluate(&actions, &time, 1.0.into()),
            ActionState::None,
        );
    }

    #[test]
    fn delayed() {
        let (mut world, mut state) = context::init_world();
        let (time, actions) = state.get(&world);

        let mut condition = Cooldown::new(1.0);

        // Initial fire
        assert_eq!(
            condition.evaluate(&actions, &time, 1.0.into()),
            ActionState::Fired,
        );

        // Release to allow rising edge
        let (time, actions) = state.get(&world);
        assert_eq!(
            condition.evaluate(&actions, &time, 0.0.into()),
            ActionState::None,
        );

        // Advance cooldown + 1ns
        world
            .resource_mut::<Time>()
            .advance_by(Duration::from_secs(1));
        world
            .resource_mut::<Time>()
            .advance_by(Duration::from_nanos(1));

        // Tick on the unpressed frame
        let (time, actions) = state.get(&world);
        assert_eq!(
            condition.evaluate(&actions, &time, 0.0.into()),
            ActionState::None,
        );

        // Now press should fire
        let (time, actions) = state.get(&world);
        assert_eq!(
            condition.evaluate(&actions, &time, 1.0.into()),
            ActionState::Fired,
        );

        // Later press without a cooldown should not fire
        let (time, actions) = state.get(&world);
        assert_eq!(
            condition.evaluate(&actions, &time, 1.0.into()),
            ActionState::None,
        );
    }
}