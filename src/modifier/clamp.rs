use bevy::prelude::*;

use crate::prelude::*;

/// Restricts input to a certain interval independently along each axis.
///
/// [`ActionValue::Bool`] will be converted into [`ActionValue::Axis1D`] before clamping.
///
/// # Examples
///
/// Bind only positive or negative direction of [`GamepadAxis::LeftStickY`].
///
/// ```
/// use bevy::prelude::*;
/// use bevy_enhanced_input::prelude::*;
///
/// actions!(Ui[
///     (
///         Action::<Up>::new(),
///         Clamp::pos(),
///         bindings![GamepadAxis::LeftStickY],
///     ),
///     (
///         Action::<Down>::new(),
///         Clamp::neg(),
///         bindings![GamepadAxis::LeftStickY],
///     ),
/// ]);
///
/// #[derive(Component, TypePath)]
/// struct Ui;
///
/// #[derive(InputAction)]
/// #[action_output(bool)]
/// struct Up;
///
/// #[derive(InputAction)]
/// #[action_output(bool)]
/// struct Down;
/// ```
#[derive(Component, Debug, Clone, Copy)]
#[cfg_attr(feature = "reflect", derive(Reflect), reflect(Clone, Component, Debug))]
pub struct Clamp {
    /// Minimum value per axis.
    pub min: Vec3,
    /// Maximum value per axis.
    pub max: Vec3,
}

impl Clamp {
    /// Creates a new instance that restricts all axes only to positive numbers.
    ///
    /// Any negative values will become 0.0.
    #[must_use]
    pub const fn pos() -> Self {
        Self::splat(0.0, f32::MAX)
    }

    /// Creates a new instance that restricts all axes only to negative numbers.
    ///
    /// Any positive values will become 0.0.
    #[must_use]
    pub const fn neg() -> Self {
        Self::splat(f32::MIN, 0.0)
    }

    /// Creates a new instance with all axes set to `min` and `max`.
    #[must_use]
    pub const fn splat(min: f32, max: f32) -> Self {
        Self::new(Vec3::splat(min), Vec3::splat(max))
    }

    #[must_use]
    pub const fn new(min: Vec3, max: Vec3) -> Self {
        Self { min, max }
    }
}

impl InputModifier for Clamp {
    fn transform(
        &mut self,
        _actions: &ActionsQuery,
        _time: &ContextTime,
        value: ActionValue,
    ) -> ActionValue {
        match value {
            ActionValue::Bool(value) => {
                let value: f32 = if value { 1.0 } else { 0.0 };
                value.clamp(self.min.x, self.max.x).into()
            }
            ActionValue::Axis1D(value) => value.clamp(self.min.x, self.max.x).into(),
            ActionValue::Axis2D(value) => value.clamp(self.min.xy(), self.max.xy()).into(),
            ActionValue::Axis3D(value) => value.clamp(self.min, self.max).into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context;

    #[test]
    fn clamping() {
        let (world, mut state) = context::init_world();
        let (time, actions) = state.get(&world);

        let mut modifier = Clamp::splat(0.0, 1.0);
        assert_eq!(modifier.transform(&actions, &time, true.into()), 1.0.into());
        assert_eq!(
            modifier.transform(&actions, &time, false.into()),
            0.0.into()
        );
        assert_eq!(modifier.transform(&actions, &time, 2.0.into()), 1.0.into());
        assert_eq!(
            modifier.transform(&actions, &time, (-1.0).into()),
            0.0.into()
        );
        assert_eq!(
            modifier.transform(&actions, &time, Vec2::new(-1.0, 2.0).into()),
            Vec2::new(0.0, 1.0).into()
        );
        assert_eq!(
            modifier.transform(&actions, &time, Vec3::new(-2.0, 0.5, 3.0).into()),
            Vec3::new(0.0, 0.5, 1.0).into()
        );
    }
}
