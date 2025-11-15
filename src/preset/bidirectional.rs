use bevy::{ecs::spawn::SpawnableList, prelude::*, ptr::MovingPtr};

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
    fn spawn(this: MovingPtr<'_, Self>, world: &mut World, entity: Entity) {
        let bidirectional = this.read();
        world.spawn((BindingOf(entity), bidirectional.positive));
        world.spawn((BindingOf(entity), bidirectional.negative, Negate::all()));
    }

    fn size_hint(&self) -> usize {
        2
    }
}

impl Bidirectional<Binding, Binding> {
    /// Maps 2 bindings as 1-dimensional input.
    #[must_use]
    pub fn new(positive: impl Into<Binding>, negative: impl Into<Binding>) -> Self {
        Self {
            positive: positive.into(),
            negative: negative.into(),
        }
    }

    /// Applies keyboard modifiers to all bindings.
    #[must_use]
    pub fn with_mod_keys(self, mod_keys: ModKeys) -> Self {
        Self {
            positive: self.positive.with_mod_keys(mod_keys),
            negative: self.negative.with_mod_keys(mod_keys),
        }
    }
}
