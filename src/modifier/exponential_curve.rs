use bevy::prelude::*;

use crate::prelude::*;

/// Response curve exponential.
///
/// Apply a simple exponential response curve to input values, per axis.
///
/// [`ActionValue::Bool`] will be transformed into [`ActionValue::Axis1D`].
#[derive(Component, Debug, Clone, Copy)]
#[cfg_attr(feature = "reflect", derive(Reflect), reflect(Clone, Component, Debug))]
pub struct ExponentialCurve {
    /// Curve exponent.
    pub exp: Vec3,
}

impl ExponentialCurve {
    /// Creates a new exponential curve with all axes set to `value`
    #[must_use]
    pub const fn splat(value: f32) -> Self {
        Self::new(Vec3::splat(value))
    }

    #[must_use]
    pub const fn new(exp: Vec3) -> Self {
        Self { exp }
    }
}

impl InputModifier for ExponentialCurve {
    fn transform(
        &mut self,
        _actions: &ActionsQuery,
        _time: &ContextTime,
        value: ActionValue,
    ) -> ActionValue {
        match value {
            ActionValue::Bool(value) => {
                let value = if value { 1.0 } else { 0.0 };
                apply_exp(value, self.exp.x).into()
            }
            ActionValue::Axis1D(value) => apply_exp(value, self.exp.x).into(),
            ActionValue::Axis2D(mut value) => {
                value.x = apply_exp(value.x, self.exp.x);
                value.y = apply_exp(value.y, self.exp.y);
                value.into()
            }
            ActionValue::Axis3D(mut value) => {
                value.x = apply_exp(value.x, self.exp.x);
                value.y = apply_exp(value.y, self.exp.y);
                value.z = apply_exp(value.z, self.exp.z);
                value.into()
            }
        }
    }
}

fn apply_exp(value: f32, exp: f32) -> f32 {
    ops::powf(value.abs(), exp).copysign(value)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context;

    #[test]
    fn exp() {
        let (world, mut state) = context::init_world();
        let (time, actions) = state.get(&world);

        let mut modifier = ExponentialCurve::splat(2.0);
        assert_eq!(modifier.transform(&actions, &time, true.into()), 1.0.into());
        assert_eq!(
            modifier.transform(&actions, &time, false.into()),
            0.0.into()
        );
        assert_eq!(
            modifier.transform(&actions, &time, (-0.5).into()),
            (-0.25).into()
        );
        assert_eq!(modifier.transform(&actions, &time, 0.5.into()), 0.25.into());
        assert_eq!(
            modifier.transform(&actions, &time, (Vec2::ONE * 2.0).into()),
            (Vec2::ONE * 4.0).into()
        );
        assert_eq!(
            modifier.transform(&actions, &time, (Vec3::ONE * 2.0).into()),
            (Vec3::ONE * 4.0).into()
        );
    }
}
