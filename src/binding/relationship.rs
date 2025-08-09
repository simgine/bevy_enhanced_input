use alloc::slice;
use core::iter::Copied;

use bevy::{
    ecs::relationship::{RelatedSpawner, RelatedSpawnerCommands},
    prelude::*,
};
use serde::{Deserialize, Serialize};

/// Action entity associated with this binding entity.
///
/// See also the [`bindings!`](crate::prelude::bindings) macro for conveniently spawning associated actions.
#[derive(Component, Deref, Reflect, Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
#[relationship(relationship_target = Bindings)]
pub struct BindingOf(pub Entity);

/// Binding entities associated with this action entity.
///
/// See also the [`bindings!`](crate::prelude::bindings) macro for conveniently spawning associated actions.
#[derive(Component, Deref, Reflect, Debug, Default, PartialEq, Eq)]
#[relationship_target(relationship = BindingOf, linked_spawn)]
pub struct Bindings(Vec<Entity>);

impl<'a> IntoIterator for &'a Bindings {
    type Item = Entity;
    type IntoIter = Copied<slice::Iter<'a, Entity>>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

/// A type alias over [`RelatedSpawner`] used to spawn binding entities containing a [`BindingOf`] relationship.
pub type BindingSpawner<'w> = RelatedSpawner<'w, BindingOf>;

/// A type alias over [`RelatedSpawnerCommands`] used to spawn binding entities containing a [`BindingOf`] relationship.
pub type BindingSpawnerCommands<'w> = RelatedSpawnerCommands<'w, BindingOf>;

/// Returns a [`SpawnRelatedBundle`](bevy::ecs::spawn::SpawnRelatedBundle) that will insert the [`Bindings`] component and
/// spawn a [`SpawnableList`] of entities with given bundles that relate to the context entity via the
/// [`BindingOf`] component.
///
/// Similar to [`related!`], but allows you to omit the explicit [`Bindings`] type and automatically wraps elements using
/// [`Binding::from`](crate::prelude::Binding::from).
///
/// The macro accepts either individual elements that implement [`Into<Binding>`], or tuples where the first element implements
/// [`Into<Binding>`] and the remaining elements are bundles.
///
/// The macro can't be used to spawn [presets](crate::preset). See the module documentation for more details.
///
/// See also [`actions!`](crate::prelude::actions).
///
/// # Examples
///
/// A list of action bindings with components constructed from values that implement [`Into<Binding>`].
///
/// ```
/// # use bevy::prelude::*;
/// # use bevy_enhanced_input::prelude::*;
/// # let mut world = World::new();
/// world.spawn(bindings![KeyCode::Space, GamepadButton::South]);
/// # assert_eq!(world.entities().len(), 3);
/// ```
///
/// A single action binding with the first component constructed from a value implementing [`Into<Binding>`],
/// and the rest as regular components.
///
/// ```
/// # use bevy::prelude::*;
/// # use bevy_enhanced_input::prelude::*;
/// # let mut world = World::new();
/// world.spawn(bindings![(
///     GamepadButton::RightTrigger2,
///     Down::new(0.3),
/// )]);
/// # assert_eq!(world.entities().len(), 2);
/// ```
///
/// A list of action bindings with the first component constructed from a value implementing [`Into<Binding>`],
/// and the rest as regular components.
///
/// ```
/// # use bevy::prelude::*;
/// # use bevy_enhanced_input::prelude::*;
/// # let mut world = World::new();
/// world.spawn(bindings![
///     (GamepadButton::RightTrigger2, Down::new(0.3)),
///     MouseButton::Left,
/// ]);
/// # assert_eq!(world.entities().len(), 3);
/// ```
///
/// [`SpawnableList`]: bevy::ecs::spawn::SpawnableList
#[macro_export]
macro_rules! bindings {
    [$($binding:expr),*$(,)?] => {
        ::bevy::prelude::related!($crate::prelude::Bindings[$($crate::prelude::IntoBindingBundle::into_binding_bundle($binding)),*])
    };
}

use crate::prelude::*;

/// Types that can be converted into a bundle with a [`Binding`].
///
/// Used to avoid writing [`Binding::from`] inside [`bindings!`].
pub trait IntoBindingBundle {
    /// Returns a bundle.
    fn into_binding_bundle(self) -> impl Bundle;
}

impl<B: Into<Binding>> IntoBindingBundle for B {
    fn into_binding_bundle(self) -> impl Bundle {
        self.into()
    }
}

macro_rules! impl_into_binding_bundle {
    ($($C:ident),*) => {
        impl<B: Into<Binding>, $($C: Bundle,)*> IntoBindingBundle for (B, $($C),*) {
            #[allow(non_snake_case, reason = "tuple unpack")]
            fn into_binding_bundle(self) -> impl Bundle {
                let (b, $($C),* ) = self;
                (b.into(), $($C),*)
            }
        }
    }
}

variadics_please::all_tuples!(impl_into_binding_bundle, 0, 14, C);
