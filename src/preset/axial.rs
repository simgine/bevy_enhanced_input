use bevy::{ecs::spawn::SpawnableList, prelude::*, ptr::MovingPtr};

use crate::prelude::*;

/// A preset to map 2 axes as 2-dimensional input.
#[derive(Debug, Clone, Copy)]
pub struct Axial<X, Y> {
    pub x: X,
    pub y: Y,
}

impl<X, Y, T: Clone> WithBundle<T> for Axial<X, Y> {
    type Output = Axial<(X, T), (Y, T)>;

    fn with(self, bundle: T) -> Self::Output {
        Axial {
            x: (self.x, bundle.clone()),
            y: (self.y, bundle),
        }
    }
}

impl Axial<Binding, Binding> {
    /// Maps left stick as 2-dimensional input.
    #[must_use]
    pub fn left_stick() -> Self {
        Self {
            x: GamepadAxis::LeftStickX.into(),
            y: GamepadAxis::LeftStickY.into(),
        }
    }

    /// Maps right stick as 2-dimensional input.
    #[must_use]
    pub fn right_stick() -> Self {
        Self {
            x: GamepadAxis::RightStickX.into(),
            y: GamepadAxis::RightStickY.into(),
        }
    }
}

impl<X: Bundle, Y: Bundle> SpawnableList<BindingOf> for Axial<X, Y> {
    fn spawn(this: MovingPtr<'_, Self>, world: &mut World, entity: Entity) {
        let axial = this.read();
        world.spawn((BindingOf(entity), axial.x));
        world.spawn((BindingOf(entity), axial.y, SwizzleAxis::YXZ));
    }

    fn size_hint(&self) -> usize {
        2
    }
}
