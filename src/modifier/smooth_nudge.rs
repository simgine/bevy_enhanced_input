use bevy::prelude::*;

use crate::prelude::*;

/// Produces a smoothed value of the current and previous input value.
///
/// See [`StableInterpolate::smooth_nudge`] for details.
///
/// [`ActionValue::Bool`] will be transformed into [`ActionValue::Axis1D`].
#[derive(Component, Reflect, Debug, Clone, Copy)]
pub struct SmoothNudge {
    /// Multiplier for delta time, determines the rate of smoothing.
    ///
    /// By default set to 8.0, an ad-hoc value that usually produces nice results.
    pub decay_rate: f32,

    /// The type of time used to advance the timer.
    pub time_kind: TimeKind,

    current_value: Vec3,
}

impl SmoothNudge {
    #[must_use]
    pub fn new(decay_rate: f32) -> Self {
        Self {
            decay_rate,
            time_kind: Default::default(),
            current_value: Vec3::ZERO,
        }
    }

    #[must_use]
    pub fn with_time_kind(mut self, kind: TimeKind) -> Self {
        self.time_kind = kind;
        self
    }
}

impl Default for SmoothNudge {
    fn default() -> Self {
        Self::new(8.0)
    }
}

impl InputModifier for SmoothNudge {
    fn transform(
        &mut self,
        _actions: &ActionsQuery,
        time: &ContextTime,
        value: ActionValue,
    ) -> ActionValue {
        if let ActionValue::Bool(value) = value {
            let value = if value { 1.0 } else { 0.0 };
            return self.transform(_actions, time, value.into());
        }

        let target_value = value.as_axis3d();
        if self.current_value.distance_squared(target_value) < 1e-4 {
            // Snap to the target if the distance is too small.
            self.current_value = target_value;
            return value;
        }

        self.current_value.smooth_nudge(
            &target_value,
            self.decay_rate,
            time.delta_kind(self.time_kind).as_secs_f32(),
        );

        ActionValue::Axis3D(self.current_value).convert(value.dim())
    }
}

#[cfg(test)]
mod tests {
    use core::time::Duration;

    use super::*;
    use crate::context;

    #[test]
    fn lerp() {
        let (mut world, mut state) = context::init_world();
        world
            .resource_mut::<Time<Real>>()
            .advance_by(Duration::from_millis(100));
        let (time, actions) = state.get(&world);

        let mut modifier = SmoothNudge::default();
        assert_eq!(
            modifier.transform(&actions, &time, 0.5.into()),
            0.27533552.into()
        );
        assert_eq!(
            modifier.transform(&actions, &time, 1.0.into()),
            0.6743873.into()
        );
    }

    #[test]
    fn bool_as_axis1d() {
        let (mut world, mut state) = context::init_world();
        world
            .resource_mut::<Time<Real>>()
            .advance_by(Duration::from_millis(100));
        let (time, actions) = state.get(&world);

        let mut modifier = SmoothNudge::default();
        assert_eq!(
            modifier.transform(&actions, &time, false.into()),
            0.0.into()
        );
        assert_eq!(
            modifier.transform(&actions, &time, true.into()),
            0.55067104.into()
        );
    }

    #[test]
    fn snapping() {
        let (mut world, mut state) = context::init_world();
        world
            .resource_mut::<Time<Real>>()
            .advance_by(Duration::from_millis(100));
        let (time, actions) = state.get(&world);

        let mut modifier = SmoothNudge {
            current_value: Vec3::X * 0.99,
            ..Default::default()
        };
        assert_eq!(modifier.transform(&actions, &time, 1.0.into()), 1.0.into());
        modifier.current_value = Vec3::X * 0.98;
        assert_ne!(modifier.transform(&actions, &time, 1.0.into()), 1.0.into());
    }
}
