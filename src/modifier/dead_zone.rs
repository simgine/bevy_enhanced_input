use bevy::prelude::*;

use crate::prelude::*;

/// Applies non-uniform normalization, suitable for both analog and digital inputs (e.g., keyboards and gamepad sticks).
///
/// Input values within the range [Self::lower_threshold] -> [Self::upper_threshold] will be remapped from 0 -> 1.
/// Values outside this range are clamped. For more details read
/// [Doing Thumbstick Dead Zones Right](https://web.archive.org/web/20210830071724/https://www.gamedeveloper.com/disciplines/doing-thumbstick-dead-zones-right)
///
/// Useful to ensure consistent diagonal movement speed.
///
/// [`ActionValue::Bool`] will be transformed into [`ActionValue::Axis1D`].
#[derive(Component, Debug, Clone, Copy)]
#[cfg_attr(feature = "reflect", derive(Reflect), reflect(Clone, Component, Debug))]
pub struct DeadZone {
    /// Defines how axes are processed.
    ///
    /// By default set to [`DeadZoneKind::Radial`].
    pub kind: DeadZoneKind,

    /// Threshold below which input is ignored.
    ///
    /// By default set to 0.2.
    pub lower_threshold: f32,

    /// Threshold above which input is clamped to 1.
    ///
    /// By default set to 1.0.
    pub upper_threshold: f32,
}

impl DeadZone {
    #[must_use]
    pub const fn new(kind: DeadZoneKind) -> Self {
        Self {
            kind,
            lower_threshold: 0.2,
            upper_threshold: 1.0,
        }
    }

    fn dead_zone(self, axis_value: f32) -> f32 {
        // Translate and scale the input to the +/- 1 range after removing the dead zone.
        let lower_bound = (axis_value.abs() - self.lower_threshold).max(0.0);
        let scaled_value = lower_bound / (self.upper_threshold - self.lower_threshold);
        scaled_value.min(1.0) * axis_value.signum()
    }
}

impl Default for DeadZone {
    fn default() -> Self {
        Self::new(Default::default())
    }
}

impl InputModifier for DeadZone {
    fn transform(
        &mut self,
        _actions: &ActionsQuery,
        _time: &ContextTime,
        value: ActionValue,
    ) -> ActionValue {
        match value {
            ActionValue::Bool(value) => {
                let value = if value { 1.0 } else { 0.0 };
                self.dead_zone(value).into()
            }
            ActionValue::Axis1D(value) => self.dead_zone(value).into(),
            ActionValue::Axis2D(mut value) => match self.kind {
                DeadZoneKind::Radial => {
                    (value.normalize_or_zero() * self.dead_zone(value.length())).into()
                }
                DeadZoneKind::Axial => {
                    value.x = self.dead_zone(value.x);
                    value.y = self.dead_zone(value.y);
                    value.into()
                }
            },
            ActionValue::Axis3D(mut value) => match self.kind {
                DeadZoneKind::Radial => {
                    (value.normalize_or_zero() * self.dead_zone(value.length())).into()
                }
                DeadZoneKind::Axial => {
                    value.x = self.dead_zone(value.x);
                    value.y = self.dead_zone(value.y);
                    value.z = self.dead_zone(value.z);
                    value.into()
                }
            },
        }
    }
}

/// Dead zone behavior.
#[derive(Default, Debug, Clone, Copy)]
#[cfg_attr(feature = "reflect", derive(Reflect), reflect(Clone, Debug, Default))]
pub enum DeadZoneKind {
    /// Apply dead zone logic to all axes simultaneously.
    ///
    /// This gives smooth input (circular/spherical coverage).
    /// For [`ActionValue::Axis1D`] and [`ActionValue::Bool`]
    /// this works identically to [`Self::Axial`].
    #[default]
    Radial,
    /// Apply dead zone to axes individually.
    ///
    /// This will result in input being chamfered at the corners
    /// for [`ActionValue::Axis2D`]/[`ActionValue::Axis2D`].
    Axial,
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use super::*;
    use crate::context;

    #[test]
    fn radial() {
        let (world, mut state) = context::init_world();
        let (time, actions) = state.get(&world);

        let mut modifier = DeadZone::new(DeadZoneKind::Radial);

        assert_eq!(modifier.transform(&actions, &time, true.into()), 1.0.into());
        assert_eq!(
            modifier.transform(&actions, &time, false.into()),
            0.0.into()
        );

        assert_eq!(modifier.transform(&actions, &time, 1.0.into()), 1.0.into());
        assert_eq!(
            modifier.transform(&actions, &time, 0.5.into()),
            0.375.into()
        );
        assert_eq!(modifier.transform(&actions, &time, 0.2.into()), 0.0.into());
        assert_eq!(modifier.transform(&actions, &time, 2.0.into()), 1.0.into());

        assert_eq!(
            modifier.transform(&actions, &time, (Vec2::ONE * 0.5).into()),
            (Vec2::ONE * 0.4482233).into()
        );
        assert_eq!(
            modifier.transform(&actions, &time, Vec2::ONE.into()),
            (Vec2::ONE * 0.70710677).into()
        );
        assert_eq!(
            modifier.transform(&actions, &time, (Vec2::ONE * 0.2).into()),
            (Vec2::ONE * 0.07322331).into()
        );

        assert_eq!(
            modifier.transform(&actions, &time, (Vec3::ONE * 0.5).into()),
            (Vec3::ONE * 0.48066244).into()
        );
        assert_eq!(
            modifier.transform(&actions, &time, Vec3::ONE.into()),
            (Vec3::ONE * 0.57735026).into()
        );
        assert_eq!(
            modifier.transform(&actions, &time, (Vec3::ONE * 0.2).into()),
            (Vec3::ONE * 0.105662435).into()
        );
    }

    #[test]
    fn axial() {
        let (world, mut state) = context::init_world();
        let (time, actions) = state.get(&world);

        let mut modifier = DeadZone::new(DeadZoneKind::Axial);

        assert_eq!(modifier.transform(&actions, &time, true.into()), 1.0.into());
        assert_eq!(
            modifier.transform(&actions, &time, false.into()),
            0.0.into()
        );
        assert_eq!(modifier.transform(&actions, &time, 1.0.into()), 1.0.into());
        assert_eq!(
            modifier.transform(&actions, &time, 0.5.into()),
            0.375.into()
        );
        assert_eq!(modifier.transform(&actions, &time, 0.2.into()), 0.0.into());
        assert_eq!(modifier.transform(&actions, &time, 2.0.into()), 1.0.into());
        assert_eq!(
            modifier.transform(&actions, &time, (Vec2::ONE * 0.5).into()),
            (Vec2::ONE * 0.375).into()
        );
        assert_eq!(
            modifier.transform(&actions, &time, Vec2::ONE.into()),
            Vec2::ONE.into()
        );
        assert_eq!(
            modifier.transform(&actions, &time, (Vec2::ONE * 0.2).into()),
            Vec2::ZERO.into()
        );
        assert_eq!(
            modifier.transform(&actions, &time, (Vec3::ONE * 0.5).into()),
            (Vec3::ONE * 0.375).into()
        );
        assert_eq!(
            modifier.transform(&actions, &time, Vec3::ONE.into()),
            Vec3::ONE.into()
        );
        assert_eq!(
            modifier.transform(&actions, &time, (Vec3::ONE * 0.2).into()),
            Vec3::ZERO.into()
        );
    }
}
