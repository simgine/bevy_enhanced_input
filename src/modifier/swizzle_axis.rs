use bevy::prelude::*;

use crate::prelude::*;

/// Swizzle axis components of an input value.
///
/// Useful for things like mapping a 1D input onto the Y axis of a 2D action.
///
/// It tries to preserve the original dimension. However, if an axis from the original
/// is promoted to a higher dimension, the value's type changes. Missing axes will be replaced with zero.
///
/// For example, [`ActionValue::Bool`] will remain unchanged for [`Self::XZY`] (X in the first place).
/// But for variants like [`Self::YXZ`] (where X becomes the second component), it will be
/// converted into [`ActionValue::Axis2D`] with Y set to the value.
#[derive(Component, Debug, Clone, Copy)]
#[cfg_attr(feature = "reflect", derive(Reflect), reflect(Clone, Component, Debug))]
pub enum SwizzleAxis {
    /// Swap X and Y axis. Useful for binding 1D inputs to the Y axis for 2D actions.
    YXZ,
    /// Swap X and Z axis.
    ZYX,
    /// Swap Y and Z axis.
    XZY,
    /// Reorder all axes, Y first.
    YZX,
    /// Reorder all axes, Z first.
    ZXY,

    /// Replace Z with Y.
    XXY,
    /// Replace Y and Z with X.
    XXZ,
    /// Replace X and Z with Y.
    YYX,
    /// Replace X and Z with Y and Z respectively.
    YYZ,
    /// Replace X and Y with Z.
    ZZX,
    /// Replace X and Y with Z and Y respectively.
    ZZY,

    /// Replace all axes with X.
    XXX,
    /// Replace all axes with Y.
    YYY,
    /// Replace all axes with Z.
    ZZZ,
}

impl InputModifier for SwizzleAxis {
    fn transform(
        &mut self,
        _actions: &ActionsQuery,
        _time: &ContextTime,
        value: ActionValue,
    ) -> ActionValue {
        match value {
            ActionValue::Bool(value) => {
                let value = if value { 1.0 } else { 0.0 };
                self.transform(_actions, _time, value.into())
            }
            ActionValue::Axis1D(value) => match self {
                SwizzleAxis::YXZ | SwizzleAxis::ZXY => (Vec2::Y * value).into(),
                SwizzleAxis::ZYX | SwizzleAxis::YZX | SwizzleAxis::YYX | SwizzleAxis::ZZX => {
                    (Vec3::Z * value).into()
                }
                SwizzleAxis::XZY => value.into(),
                SwizzleAxis::XXY | SwizzleAxis::XXZ => Vec2::splat(value).into(),
                SwizzleAxis::YYZ | SwizzleAxis::YYY | SwizzleAxis::ZZZ | SwizzleAxis::ZZY => {
                    0.0.into()
                }
                SwizzleAxis::XXX => Vec3::splat(value).into(),
            },
            ActionValue::Axis2D(value) => match self {
                SwizzleAxis::YXZ => value.yx().into(),
                SwizzleAxis::ZYX => (0.0, value.y, value.x).into(),
                SwizzleAxis::XZY => (value.x, 0.0, value.y).into(),
                SwizzleAxis::YZX => (value.y, 0.0, value.x).into(),
                SwizzleAxis::ZXY => (0.0, value.x, value.y).into(),
                SwizzleAxis::XXY => value.xxy().into(),
                SwizzleAxis::XXZ => value.xx().into(),
                SwizzleAxis::YYX => value.yyx().into(),
                SwizzleAxis::YYZ => value.yy().into(),
                SwizzleAxis::ZZX => (value.x * Vec3::Z).into(),
                SwizzleAxis::ZZY => (value.y * Vec3::Z).into(),
                SwizzleAxis::XXX => value.xxx().into(),
                SwizzleAxis::YYY => value.yyy().into(),
                SwizzleAxis::ZZZ => Vec2::ZERO.into(),
            },
            ActionValue::Axis3D(value) => match self {
                SwizzleAxis::YXZ => value.yxz().into(),
                SwizzleAxis::ZYX => value.zyx().into(),
                SwizzleAxis::XZY => value.xzy().into(),
                SwizzleAxis::YZX => value.yzx().into(),
                SwizzleAxis::ZXY => value.zxy().into(),
                SwizzleAxis::XXY => value.xxy().into(),
                SwizzleAxis::XXZ => value.xxz().into(),
                SwizzleAxis::YYX => value.yyx().into(),
                SwizzleAxis::YYZ => value.yyz().into(),
                SwizzleAxis::ZZX => value.zzx().into(),
                SwizzleAxis::ZZY => value.zzy().into(),
                SwizzleAxis::XXX => value.xxx().into(),
                SwizzleAxis::YYY => value.yyy().into(),
                SwizzleAxis::ZZZ => value.zzz().into(),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context;

    #[test]
    fn yxz() {
        let (world, mut state) = context::init_world();
        let (time, actions) = state.get(&world);

        let mut modifier = SwizzleAxis::YXZ;
        assert_eq!(
            modifier.transform(&actions, &time, true.into()),
            Vec2::Y.into()
        );
        assert_eq!(
            modifier.transform(&actions, &time, false.into()),
            Vec2::ZERO.into()
        );
        assert_eq!(
            modifier.transform(&actions, &time, 1.0.into()),
            Vec2::Y.into()
        );
        assert_eq!(
            modifier.transform(&actions, &time, (0.0, 1.0).into()),
            (1.0, 0.0).into()
        );
        assert_eq!(
            modifier.transform(&actions, &time, (0.0, 1.0, 2.0).into()),
            (1.0, 0.0, 2.0).into(),
        );
    }

    #[test]
    fn zyx() {
        let (world, mut state) = context::init_world();
        let (time, actions) = state.get(&world);

        let mut modifier = SwizzleAxis::ZYX;
        assert_eq!(
            modifier.transform(&actions, &time, true.into()),
            Vec3::Z.into()
        );
        assert_eq!(
            modifier.transform(&actions, &time, false.into()),
            Vec3::ZERO.into()
        );
        assert_eq!(
            modifier.transform(&actions, &time, 1.0.into()),
            Vec3::Z.into()
        );
        assert_eq!(
            modifier.transform(&actions, &time, (0.0, 1.0).into()),
            (0.0, 1.0, 0.0).into()
        );
        assert_eq!(
            modifier.transform(&actions, &time, (0.0, 1.0, 2.0).into()),
            (2.0, 1.0, 0.0).into(),
        );
    }

    #[test]
    fn xzy() {
        let (world, mut state) = context::init_world();
        let (time, actions) = state.get(&world);

        let mut modifier = SwizzleAxis::XZY;
        assert_eq!(modifier.transform(&actions, &time, true.into()), 1.0.into());
        assert_eq!(
            modifier.transform(&actions, &time, false.into()),
            0.0.into()
        );
        assert_eq!(modifier.transform(&actions, &time, 1.0.into()), 1.0.into());
        assert_eq!(
            modifier.transform(&actions, &time, (0.0, 1.0).into()),
            (0.0, 0.0, 1.0).into()
        );
        assert_eq!(
            modifier.transform(&actions, &time, (0.0, 1.0, 2.0).into()),
            (0.0, 2.0, 1.0).into(),
        );
    }

    #[test]
    fn yzx() {
        let (world, mut state) = context::init_world();
        let (time, actions) = state.get(&world);

        let mut modifier = SwizzleAxis::YZX;
        assert_eq!(
            modifier.transform(&actions, &time, true.into()),
            Vec3::Z.into()
        );
        assert_eq!(
            modifier.transform(&actions, &time, false.into()),
            Vec3::ZERO.into()
        );
        assert_eq!(
            modifier.transform(&actions, &time, 1.0.into()),
            Vec3::Z.into()
        );
        assert_eq!(
            modifier.transform(&actions, &time, (0.0, 1.0).into()),
            (1.0, 0.0, 0.0).into()
        );
        assert_eq!(
            modifier.transform(&actions, &time, (0.0, 1.0, 2.0).into()),
            (1.0, 2.0, 0.0).into(),
        );
    }

    #[test]
    fn zxy() {
        let (world, mut state) = context::init_world();
        let (time, actions) = state.get(&world);

        let mut modifier = SwizzleAxis::ZXY;
        assert_eq!(
            modifier.transform(&actions, &time, true.into()),
            Vec2::Y.into()
        );
        assert_eq!(
            modifier.transform(&actions, &time, false.into()),
            Vec2::ZERO.into()
        );
        assert_eq!(
            modifier.transform(&actions, &time, 1.0.into()),
            Vec2::Y.into()
        );
        assert_eq!(
            modifier.transform(&actions, &time, (0.0, 1.0).into()),
            (0.0, 0.0, 1.0).into()
        );
        assert_eq!(
            modifier.transform(&actions, &time, (0.0, 1.0, 2.0).into()),
            (2.0, 0.0, 1.0).into(),
        );
    }

    #[test]
    fn xxy() {
        let (world, mut state) = context::init_world();
        let (time, actions) = state.get(&world);

        let mut modifier = SwizzleAxis::XXY;
        assert_eq!(
            modifier.transform(&actions, &time, true.into()),
            Vec2::splat(1.0).into()
        );
        assert_eq!(
            modifier.transform(&actions, &time, false.into()),
            Vec2::splat(0.0).into()
        );
        assert_eq!(
            modifier.transform(&actions, &time, 3.0.into()),
            Vec2::splat(3.0).into()
        );
        assert_eq!(
            modifier.transform(&actions, &time, (2.0, 5.0).into()),
            (2.0, 2.0, 5.0).into()
        );
        assert_eq!(
            modifier.transform(&actions, &time, (2.0, 5.0, 7.0).into()),
            (2.0, 2.0, 5.0).into()
        );
    }

    #[test]
    fn yyx() {
        let (world, mut state) = context::init_world();
        let (time, actions) = state.get(&world);

        let mut modifier = SwizzleAxis::YYX;
        assert_eq!(
            modifier.transform(&actions, &time, true.into()),
            (Vec3::Z * 1.0).into()
        );
        assert_eq!(
            modifier.transform(&actions, &time, false.into()),
            Vec3::ZERO.into()
        );
        assert_eq!(
            modifier.transform(&actions, &time, 4.0.into()),
            (Vec3::Z * 4.0).into()
        );
        assert_eq!(
            modifier.transform(&actions, &time, (3.0, 6.0).into()),
            (6.0, 6.0, 3.0).into()
        );
        assert_eq!(
            modifier.transform(&actions, &time, (3.0, 6.0, 9.0).into()),
            (6.0, 6.0, 3.0).into()
        );
    }

    #[test]
    fn xxz() {
        let (world, mut state) = context::init_world();
        let (time, actions) = state.get(&world);

        let mut modifier = SwizzleAxis::XXZ;
        assert_eq!(
            modifier.transform(&actions, &time, true.into()),
            Vec2::splat(1.0).into()
        );
        assert_eq!(
            modifier.transform(&actions, &time, false.into()),
            Vec2::splat(0.0).into()
        );
        assert_eq!(
            modifier.transform(&actions, &time, 3.5.into()),
            Vec2::splat(3.5).into()
        );
        assert_eq!(
            modifier.transform(&actions, &time, (2.0, 4.0).into()),
            (2.0, 2.0).into()
        );
        assert_eq!(
            modifier.transform(&actions, &time, (2.0, 4.0, 6.0).into()),
            (2.0, 2.0, 6.0).into()
        );
    }

    #[test]
    fn yyz() {
        let (world, mut state) = context::init_world();
        let (time, actions) = state.get(&world);

        let mut modifier = SwizzleAxis::YYZ;
        assert_eq!(modifier.transform(&actions, &time, true.into()), 0.0.into());
        assert_eq!(
            modifier.transform(&actions, &time, false.into()),
            0.0.into()
        );
        assert_eq!(modifier.transform(&actions, &time, 2.0.into()), 0.0.into());
        assert_eq!(
            modifier.transform(&actions, &time, (1.0, 3.0).into()),
            (3.0, 3.0).into()
        );
        assert_eq!(
            modifier.transform(&actions, &time, (1.0, 3.0, 5.0).into()),
            (3.0, 3.0, 5.0).into()
        );
    }

    #[test]
    fn zzx() {
        let (world, mut state) = context::init_world();
        let (time, actions) = state.get(&world);

        let mut modifier = SwizzleAxis::ZZX;
        assert_eq!(
            modifier.transform(&actions, &time, true.into()),
            (Vec3::Z * 1.0).into()
        );
        assert_eq!(
            modifier.transform(&actions, &time, false.into()),
            (Vec3::Z * 0.0).into()
        );
        assert_eq!(
            modifier.transform(&actions, &time, 7.0.into()),
            (Vec3::Z * 7.0).into()
        );
        assert_eq!(
            modifier.transform(&actions, &time, (3.0, 5.0).into()),
            (Vec3::Z * 3.0).into()
        );
        assert_eq!(
            modifier.transform(&actions, &time, (3.0, 5.0, 8.0).into()),
            (8.0, 8.0, 3.0).into()
        );
    }

    #[test]
    fn zzy() {
        let (world, mut state) = context::init_world();
        let (time, actions) = state.get(&world);

        let mut modifier = SwizzleAxis::ZZY;
        assert_eq!(modifier.transform(&actions, &time, true.into()), 0.0.into());
        assert_eq!(
            modifier.transform(&actions, &time, false.into()),
            0.0.into()
        );
        assert_eq!(modifier.transform(&actions, &time, 4.0.into()), 0.0.into());
        assert_eq!(
            modifier.transform(&actions, &time, (1.0, 2.0).into()),
            (2.0 * Vec3::Z).into()
        );
        assert_eq!(
            modifier.transform(&actions, &time, (1.0, 2.0, 3.0).into()),
            (3.0, 3.0, 2.0).into()
        );
    }

    #[test]
    fn xxx() {
        let (world, mut state) = context::init_world();
        let (time, actions) = state.get(&world);

        let mut modifier = SwizzleAxis::XXX;
        assert_eq!(
            modifier.transform(&actions, &time, true.into()),
            Vec3::ONE.into()
        );
        assert_eq!(
            modifier.transform(&actions, &time, false.into()),
            Vec3::ZERO.into()
        );
        assert_eq!(
            modifier.transform(&actions, &time, 1.0.into()),
            Vec3::ONE.into()
        );
        assert_eq!(
            modifier.transform(&actions, &time, (2.0, 3.0).into()),
            Vec3::splat(2.0).into()
        );
        assert_eq!(
            modifier.transform(&actions, &time, (4.0, 5.0, 6.0).into()),
            Vec3::splat(4.0).into()
        );
    }

    #[test]
    fn yyy() {
        let (world, mut state) = context::init_world();
        let (time, actions) = state.get(&world);

        let mut modifier = SwizzleAxis::YYY;
        assert_eq!(modifier.transform(&actions, &time, true.into()), 0.0.into());
        assert_eq!(
            modifier.transform(&actions, &time, false.into()),
            0.0.into()
        );
        assert_eq!(modifier.transform(&actions, &time, 1.0.into()), 0.0.into());
        assert_eq!(
            modifier.transform(&actions, &time, (2.0, 3.0).into()),
            Vec3::splat(3.0).into()
        );
        assert_eq!(
            modifier.transform(&actions, &time, (4.0, 5.0, 6.0).into()),
            Vec3::splat(5.0).into()
        );
    }

    #[test]
    fn zzz() {
        let (world, mut state) = context::init_world();
        let (time, actions) = state.get(&world);

        let mut modifier = SwizzleAxis::ZZZ;
        assert_eq!(modifier.transform(&actions, &time, true.into()), 0.0.into());
        assert_eq!(
            modifier.transform(&actions, &time, false.into()),
            0.0.into()
        );
        assert_eq!(modifier.transform(&actions, &time, 1.0.into()), 0.0.into());
        assert_eq!(
            modifier.transform(&actions, &time, (2.0, 3.0).into()),
            Vec2::ZERO.into()
        );
        assert_eq!(
            modifier.transform(&actions, &time, (4.0, 5.0, 6.0).into()),
            Vec3::splat(6.0).into()
        );
    }
}
