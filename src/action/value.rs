use core::fmt::Debug;

use bevy::prelude::*;
#[cfg(feature = "serialize")]
use serde::{Deserialize, Serialize};

/// Value for [`Action<C>`](super::Action) in dynamically typed form.
///
/// Updated from [`Bindings`](crate::prelude::Bindings) and optionally modified by
/// [`modifiers`](crate::modifier), or overridden by [`ActionMock`](super::ActionMock) if present.
#[derive(Component, Reflect, Debug, PartialEq, Clone, Copy)]
#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serialize", reflect(Serialize, Deserialize))]
pub enum ActionValue {
    Bool(bool),
    Axis1D(f32),
    Axis2D(Vec2),
    Axis3D(Vec3),
}

impl ActionValue {
    /// Creates a zero-initialized value for the specified dimension.
    #[must_use]
    pub const fn zero(dim: ActionValueDim) -> Self {
        match dim {
            ActionValueDim::Bool => ActionValue::Bool(false),
            ActionValueDim::Axis1D => ActionValue::Axis1D(0.0),
            ActionValueDim::Axis2D => ActionValue::Axis2D(Vec2::ZERO),
            ActionValueDim::Axis3D => ActionValue::Axis3D(Vec3::ZERO),
        }
    }

    /// Returns dimension.
    #[must_use]
    pub const fn dim(self) -> ActionValueDim {
        match self {
            Self::Bool(_) => ActionValueDim::Bool,
            Self::Axis1D(_) => ActionValueDim::Axis1D,
            Self::Axis2D(_) => ActionValueDim::Axis2D,
            Self::Axis3D(_) => ActionValueDim::Axis3D,
        }
    }

    /// Converts the value into the specified variant based on the dimension.
    ///
    /// If the new dimension is larger, the additional axes will be set to zero.
    /// If the new dimension is smaller, the extra axes will be discarded.
    #[must_use]
    pub fn convert(self, dim: ActionValueDim) -> Self {
        match dim {
            ActionValueDim::Bool => self.as_bool().into(),
            ActionValueDim::Axis1D => self.as_axis1d().into(),
            ActionValueDim::Axis2D => self.as_axis2d().into(),
            ActionValueDim::Axis3D => self.as_axis3d().into(),
        }
    }

    /// Returns `true` if the value in sufficiently large.
    #[must_use]
    pub fn is_actuated(self, actuation: f32) -> bool {
        self.as_axis3d().length_squared() >= actuation * actuation
    }

    /// Returns the value as a boolean.
    ///
    /// If the value is not [`ActionValue::Bool`],
    /// it returns `false` if the value is zero, and `true` otherwise.
    #[must_use]
    pub fn as_bool(self) -> bool {
        match self {
            Self::Bool(value) => value,
            Self::Axis1D(value) => value != 0.0,
            Self::Axis2D(value) => value != Vec2::ZERO,
            Self::Axis3D(value) => value != Vec3::ZERO,
        }
    }

    /// Returns the value as a 1-dimensional axis.
    ///
    /// For [`ActionValue::Bool`], it returns `1.0` if `true`, otherwise `0.0`.
    /// For multi-dimensional values, it returns the X axis.
    #[must_use]
    pub const fn as_axis1d(self) -> f32 {
        match self {
            Self::Bool(value) => {
                if value {
                    1.0
                } else {
                    0.0
                }
            }
            Self::Axis1D(value) => value,
            Self::Axis2D(value) => value.x,
            Self::Axis3D(value) => value.x,
        }
    }

    /// Returns the value as a 2-dimensional axis.
    ///
    /// For [`ActionValue::Bool`], it returns [`Vec2::X`] if `true`, otherwise [`Vec2::ZERO`].
    /// For [`ActionValue::Axis1D`], it maps the value to the X axis.
    /// For [`ActionValue::Axis3D`], it returns the X and Y axes.
    #[must_use]
    pub fn as_axis2d(self) -> Vec2 {
        match self {
            Self::Bool(value) => {
                if value {
                    Vec2::X
                } else {
                    Vec2::ZERO
                }
            }
            Self::Axis1D(value) => Vec2::X * value,
            Self::Axis2D(value) => value,
            Self::Axis3D(value) => value.xy(),
        }
    }

    /// Returns the value as a 3-dimensional axis .
    ///
    /// For [`ActionValue::Bool`], it returns [`Vec3::X`] if `true`, otherwise [`Vec3::ZERO`].
    /// For [`ActionValue::Axis1D`], it maps the value to the X axis.
    /// For [`ActionValue::Axis2D`], it maps the value to the X and Y axes.
    #[must_use]
    pub fn as_axis3d(self) -> Vec3 {
        match self {
            Self::Bool(value) => {
                if value {
                    Vec3::X
                } else {
                    Vec3::ZERO
                }
            }
            Self::Axis1D(value) => Vec3::X * value,
            Self::Axis2D(value) => value.extend(0.0),
            Self::Axis3D(value) => value,
        }
    }
}

/// A dimension discriminant for [`ActionValue`].
#[derive(Debug, PartialEq, Eq, PartialOrd, Reflect, Ord, Clone, Copy)]
#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serialize", reflect(Serialize, Deserialize))]
pub enum ActionValueDim {
    Bool,
    Axis1D,
    Axis2D,
    Axis3D,
}

impl From<bool> for ActionValue {
    fn from(value: bool) -> Self {
        ActionValue::Bool(value)
    }
}

impl From<f32> for ActionValue {
    fn from(value: f32) -> Self {
        ActionValue::Axis1D(value)
    }
}

impl From<Vec2> for ActionValue {
    fn from(value: Vec2) -> Self {
        ActionValue::Axis2D(value)
    }
}

impl From<Vec3> for ActionValue {
    fn from(value: Vec3) -> Self {
        ActionValue::Axis3D(value)
    }
}

impl From<(f32, f32)> for ActionValue {
    fn from(value: (f32, f32)) -> Self {
        ActionValue::Axis2D(value.into())
    }
}

impl From<(f32, f32, f32)> for ActionValue {
    fn from(value: (f32, f32, f32)) -> Self {
        ActionValue::Axis3D(value.into())
    }
}

impl From<ActionValue> for bool {
    fn from(value: ActionValue) -> Self {
        value.as_bool()
    }
}

impl From<ActionValue> for f32 {
    fn from(value: ActionValue) -> Self {
        value.as_axis1d()
    }
}

impl From<ActionValue> for Vec2 {
    fn from(value: ActionValue) -> Self {
        value.as_axis2d()
    }
}

impl From<ActionValue> for Vec3 {
    fn from(value: ActionValue) -> Self {
        value.as_axis3d()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bool_conversion() {
        let value = ActionValue::Bool(true);
        assert_eq!(value.convert(ActionValueDim::Bool), true.into());
        assert_eq!(value.convert(ActionValueDim::Axis1D), 1.0.into());
        assert_eq!(value.convert(ActionValueDim::Axis2D), (1.0, 0.0).into());
        assert_eq!(
            value.convert(ActionValueDim::Axis3D),
            (1.0, 0.0, 0.0).into()
        );
    }

    #[test]
    fn axis1d_conversion() {
        let value = ActionValue::Axis1D(1.0);
        assert_eq!(value.convert(ActionValueDim::Bool), true.into());
        assert_eq!(value.convert(ActionValueDim::Axis1D), 1.0.into());
        assert_eq!(value.convert(ActionValueDim::Axis2D), (1.0, 0.0).into());
        assert_eq!(
            value.convert(ActionValueDim::Axis3D),
            (1.0, 0.0, 0.0).into()
        );
    }

    #[test]
    fn axis2d_conversion() {
        let value = ActionValue::Axis2D(Vec2::ONE);
        assert_eq!(value.convert(ActionValueDim::Bool), true.into());
        assert_eq!(value.convert(ActionValueDim::Axis1D), 1.0.into());
        assert_eq!(value.convert(ActionValueDim::Axis2D), Vec2::ONE.into());
        assert_eq!(
            value.convert(ActionValueDim::Axis3D),
            (1.0, 1.0, 0.0).into()
        );
    }

    #[test]
    fn axis3d_conversion() {
        let value = ActionValue::Axis3D(Vec3::ONE);
        assert_eq!(value.convert(ActionValueDim::Bool), true.into());
        assert_eq!(value.convert(ActionValueDim::Axis1D), 1.0.into());
        assert_eq!(value.convert(ActionValueDim::Axis2D), Vec2::ONE.into());
        assert_eq!(value.convert(ActionValueDim::Axis3D), Vec3::ONE.into());
    }
}
