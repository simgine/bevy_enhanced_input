use bevy::prelude::*;
use log::warn;

use crate::prelude::*;

/// Gradually steps the input value toward the target value at a constant linear rate.
///
/// [`ActionValue::Bool`] will be transformed into [`ActionValue::Axis1D`]
#[derive(Component, Debug, Clone, Copy)]
#[cfg_attr(feature = "reflect", derive(Reflect), reflect(Clone, Component, Debug))]
pub struct LinearStep {
    /// The fraction of the distance to step per frame while accelerating.
    ///
    /// Must be between `0.0` and `1.0`, where `0.0` results
    /// in no movement and `1.0` snaps directly to the target value.
    pub accel_step_rate: f32,

    /// Like [`Self::accel_step_rate`], but for deceleration.
    pub decel_step_rate: f32,

    current_value: Vec3,
}

impl LinearStep {
    /// Creates a new instance with acceleration and deceleration rates set to `step_rate`.
    #[must_use]
    pub const fn splat(step_rate: f32) -> Self {
        Self::new(step_rate, step_rate)
    }

    #[must_use]
    pub const fn new(accel_step_rate: f32, decel_step_rate: f32) -> Self {
        Self {
            accel_step_rate,
            decel_step_rate,
            current_value: Vec3::ZERO,
        }
    }
}

impl InputModifier for LinearStep {
    fn transform(
        &mut self,
        _actions: &ActionsQuery,
        _time: &ContextTime,
        value: ActionValue,
    ) -> ActionValue {
        if let ActionValue::Bool(value) = value {
            let value = if value { 1.0 } else { 0.0 };
            return self.transform(_actions, _time, value.into());
        }

        let target_value = value.as_axis3d();
        let diff = target_value.length() - self.current_value.length();
        let step_rate = if diff > 0.0 {
            self.accel_step_rate
        } else {
            self.decel_step_rate
        };

        if !(0.0..=1.0).contains(&step_rate) {
            // TODO: use `warn_once` when `bevy_log` becomes `no_std` compatible.
            warn!("step rate can't be outside 0.0..=1.0: {step_rate}");
            return value;
        }

        // Snap if distance is less than one step.
        let distance = self.current_value.distance(target_value);
        if distance <= step_rate {
            self.current_value = target_value;
            return value;
        }

        if diff == 0.0 {
            return value;
        }
        if diff > 0.0 {
            self.current_value += step_rate * target_value;
        } else {
            self.current_value -= step_rate * self.current_value.signum();
        }

        ActionValue::Axis3D(self.current_value).convert(value.dim())
    }
}

#[cfg(test)]
mod tests {
    use core::time::Duration;

    use super::*;
    use crate::context;

    #[test]
    fn stepping() {
        let (mut world, mut state) = context::init_world();
        world
            .resource_mut::<Time<Real>>()
            .advance_by(Duration::from_millis(100));
        let (time, actions) = state.get(&world);

        let mut modifier = LinearStep::splat(0.1);
        // Forward
        assert_eq!(modifier.transform(&actions, &time, 1.0.into()), 0.1.into());
        assert_eq!(modifier.transform(&actions, &time, 1.0.into()), 0.2.into());

        // No movement
        assert_eq!(modifier.transform(&actions, &time, 0.2.into()), 0.2.into());

        // Backward
        assert_eq!(modifier.transform(&actions, &time, 0.0.into()), 0.1.into());
        assert_eq!(modifier.transform(&actions, &time, 0.0.into()), 0.0.into());
    }

    #[test]
    fn invalid_step_rate() {
        let (mut world, mut state) = context::init_world();
        world
            .resource_mut::<Time<Real>>()
            .advance_by(Duration::from_millis(100));
        let (time, actions) = state.get(&world);

        let mut modifier = LinearStep::splat(2.0);
        assert_eq!(modifier.transform(&actions, &time, 1.0.into()), 1.0.into());
        assert_eq!(
            modifier.transform(&actions, &time, (-1.0).into()),
            (-1.0).into()
        );
    }

    #[test]
    fn bool_as_axis1d() {
        let (mut world, mut state) = context::init_world();
        world
            .resource_mut::<Time<Real>>()
            .advance_by(Duration::from_millis(100));
        let (time, actions) = state.get(&world);

        let mut modifier = LinearStep::splat(0.1);
        assert_eq!(
            modifier.transform(&actions, &time, false.into()),
            0.0.into()
        );
        assert_eq!(modifier.transform(&actions, &time, true.into()), 0.1.into());
    }

    #[test]
    fn snapping() {
        let (mut world, mut state) = context::init_world();
        world
            .resource_mut::<Time<Real>>()
            .advance_by(Duration::from_millis(100));
        let (time, actions) = state.get(&world);

        let mut modifier = LinearStep {
            current_value: Vec3::X * 0.95,
            accel_step_rate: 0.1,
            decel_step_rate: 0.1,
        };
        assert_eq!(modifier.transform(&actions, &time, 1.0.into()), 1.0.into());
    }
}
