use bevy::{ptr::{move_as_ptr, MovingPtr}, ecs::{spawn::SpawnableList, }, prelude::*};
use crate::prelude::*;

/// A preset to map 6 buttons as 3-dimensional input.
#[derive(Debug, Clone, Copy)]
pub struct Spatial<F, B, L, R, U, D> {
    pub forward: F,
    pub backward: B,
    pub left: L,
    pub right: R,
    pub up: U,
    pub down: D,
}

impl<F, B, L, R, U, D, T: Clone> WithBundle<T> for Spatial<F, B, L, R, U, D> {
    type Output = Spatial<(F, T), (B, T), (L, T), (R, T), (U, T), (D, T)>;

    fn with(self, bundle: T) -> Self::Output {
        Spatial {
            forward: (self.forward, bundle.clone()),
            backward: (self.backward, bundle.clone()),
            left: (self.left, bundle.clone()),
            right: (self.right, bundle.clone()),
            up: (self.up, bundle.clone()),
            down: (self.down, bundle),
        }
    }
}

impl Spatial<Binding, Binding, Binding, Binding, Binding, Binding> {
    /// Maps WASD keys for horizontal (XZ) inputs and takes in up/down mappings.
    #[must_use]
    pub fn wasd_and(up: KeyCode, down: KeyCode) -> Self {
        Spatial {
            forward: KeyCode::KeyW.into(),
            backward: KeyCode::KeyS.into(),
            left: KeyCode::KeyA.into(),
            right: KeyCode::KeyD.into(),
            up: up.into(),
            down: down.into(),
        }
    }

    /// Maps arrow keys for horizontal (XZ) inputs and takes in up/down mappings.
    #[must_use]
    pub fn arrows_and(up: KeyCode, down: KeyCode) -> Self {
        Spatial {
            forward: KeyCode::ArrowUp.into(),
            backward: KeyCode::ArrowDown.into(),
            left: KeyCode::ArrowLeft.into(),
            right: KeyCode::ArrowRight.into(),
            up: up.into(),
            down: down.into(),
        }
    }
}

impl<F, B, L, R, U, D> SpawnableList<BindingOf> for Spatial<F, B, L, R, U, D>
where
    F: Bundle,
    B: Bundle,
    L: Bundle,
    R: Bundle,
    U: Bundle,
    D: Bundle,
{
    fn spawn(self, world: &mut World, entity: Entity) {
        let xy = Cardinal {
            north: self.up,
            east: self.right,
            south: self.down,
            west: self.left,
        };
        move_as_ptr!(xy);
        SpawnableList::spawn(xy, world, entity);

        let z = Bidirectional {
            positive: self.backward,
            negative: self.forward,
        };
        move_as_ptr!(z);
        SpawnableList::spawn(z.with(SwizzleAxis::ZYX), world, entity);
    }

    fn size_hint(&self) -> usize {
        6
    }
}
