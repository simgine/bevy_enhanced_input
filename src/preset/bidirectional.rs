use bevy::{ecs::spawn::SpawnableList, prelude::*};

use crate::prelude::*;

/// A preset to map 2 buttons as 1-dimensional input.
#[derive(Debug, Clone, Copy)]
pub struct Bidirectional<P, N> {
    pub positive: P,
    pub negative: N,
}

impl<P, N, T: Clone> WithBundle<T> for Bidirectional<P, N> {
    type Output = Bidirectional<(P, T), (N, T)>;

    fn with(self, bundle: T) -> Self::Output {
        Bidirectional {
            positive: (self.positive, bundle.clone()),
            negative: (self.negative, bundle),
        }
    }
}

impl<P: Bundle, N: Bundle> SpawnableList<BindingOf> for Bidirectional<P, N> {
    fn spawn(self, world: &mut World, entity: Entity) {
        world.spawn((BindingOf(entity), self.positive));
        world.spawn((BindingOf(entity), self.negative, Negate::all()));
    }

    fn size_hint(&self) -> usize {
        2
    }
}

impl Bidirectional<Binding, Binding> {
    /// Maps WASD keys as 1-dimensional input.
    #[must_use]
    pub fn ad_keys() -> Self {
        Self {
            positive: KeyCode::KeyD.into(),
            negative: KeyCode::KeyA.into(),
        }
    }

    /// Maps keyboard arrow keys as 1-dimensional input.
    #[must_use]
    pub fn arrow_keys() -> Self {
        Self {
            positive: KeyCode::ArrowRight.into(),
            negative: KeyCode::ArrowLeft.into(),
        }
    }
}
