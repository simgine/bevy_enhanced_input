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
    /// Maps A and D keys as 1-dimensional input.
    #[must_use]
    pub fn ad_keys() -> Self {
        Self {
            positive: KeyCode::KeyD.into(),
            negative: KeyCode::KeyA.into(),
        }
    }

    /// Maps W and S keys as 1-dimensional input.
    #[must_use]
    pub fn ws_keys() -> Self {
        Self {
            positive: KeyCode::KeyW.into(),
            negative: KeyCode::KeyS.into(),
        }
    }

    /// Maps left and right keyboard arrow keys as 1-dimensional input.
    #[must_use]
    pub fn left_right_arrow() -> Self {
        Self {
            positive: KeyCode::ArrowRight.into(),
            negative: KeyCode::ArrowLeft.into(),
        }
    }

    /// Maps up and down keyboard arrow keys as 1-dimensional input.
    #[must_use]
    pub fn up_down_arrow() -> Self {
        Self {
            positive: KeyCode::ArrowUp.into(),
            negative: KeyCode::ArrowDown.into(),
        }
    }

    /// Maps up and down D-pad buttons as 1-dimensional horizontal input.
    #[must_use]
    pub fn left_right_dpad() -> Self {
        Self {
            positive: GamepadButton::DPadRight.into(),
            negative: GamepadButton::DPadLeft.into(),
        }
    }

    /// Maps left and right D-pad buttons as 1-dimensional horizontal input.
    #[must_use]
    pub fn up_down_dpad() -> Self {
        Self {
            positive: GamepadButton::DPadUp.into(),
            negative: GamepadButton::DPadDown.into(),
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
