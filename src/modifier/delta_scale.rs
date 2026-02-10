use bevy::prelude::*;

use crate::prelude::*;

/// Multiplies the input value by delta time for this frame.
///
/// [`ActionValue::Bool`] will be transformed into [`ActionValue::Axis1D`].
#[derive(Component, Default, Debug, Clone, Copy)]
#[cfg_attr(
    feature = "reflect",
    derive(Reflect),
    reflect(Clone, Component, Debug, Default)
)]
pub struct DeltaScale {
    /// The type of time used to advance the timer.
    pub time_kind: TimeKind,
}

impl InputModifier for DeltaScale {
    fn transform(
        &mut self,
        _actions: &ActionsQuery,
        time: &ContextTime,
        value: ActionValue,
    ) -> ActionValue {
        match value {
            ActionValue::Bool(value) => {
                let value = if value { 1.0 } else { 0.0 };
                (value * time.delta_kind(self.time_kind).as_secs_f32()).into()
            }
            ActionValue::Axis1D(value) => {
                (value * time.delta_kind(self.time_kind).as_secs_f32()).into()
            }
            ActionValue::Axis2D(value) => {
                (value * time.delta_kind(self.time_kind).as_secs_f32()).into()
            }
            ActionValue::Axis3D(value) => {
                (value * time.delta_kind(self.time_kind).as_secs_f32()).into()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use core::time::Duration;

    use bevy::prelude::*;

    use super::*;
    use crate::context;

    #[test]
    fn scaling() {
        let (mut world, mut state) = context::init_world();
        world
            .resource_mut::<Time<Real>>()
            .advance_by(Duration::from_millis(500));
        let (time, actions) = state.get(&world);

        assert_eq!(
            DeltaScale::default().transform(&actions, &time, true.into()),
            0.5.into()
        );
        assert_eq!(
            DeltaScale::default().transform(&actions, &time, false.into()),
            0.0.into()
        );
        assert_eq!(
            DeltaScale::default().transform(&actions, &time, 0.5.into()),
            0.25.into()
        );
        assert_eq!(
            DeltaScale::default().transform(&actions, &time, Vec2::ONE.into()),
            (0.5, 0.5).into()
        );
        assert_eq!(
            DeltaScale::default().transform(&actions, &time, Vec3::ONE.into()),
            (0.5, 0.5, 0.5).into()
        );
    }
}
