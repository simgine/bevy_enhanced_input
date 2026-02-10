use alloc::slice;
use core::{
    fmt::{self, Debug, Formatter},
    iter::Copied,
    marker::PhantomData,
};

use bevy::{
    ecs::relationship::{RelatedSpawner, RelatedSpawnerCommands},
    prelude::*,
};
#[cfg(feature = "serialize")]
use serde::{Deserialize, Serialize};

/// Context entity associated with this action entity.
///
/// See also the [`actions!`](crate::prelude::actions) macro for conveniently spawning associated actions.
#[derive(Component, Deref)]
#[cfg_attr(
    feature = "reflect",
    derive(Reflect),
    reflect(Clone, Component, Debug, PartialEq, type_path = false)
)]
#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
#[cfg_attr(
    all(feature = "reflect", feature = "serialize"),
    reflect(Serialize, Deserialize)
)]
#[relationship(relationship_target = Actions<C>)]
pub struct ActionOf<C: Component> {
    #[deref]
    #[relationship]
    entity: Entity,
    #[cfg_attr(feature = "reflect", reflect(ignore))]
    marker: PhantomData<C>,
}

impl<C: Component> ActionOf<C> {
    #[must_use]
    pub const fn new(entity: Entity) -> Self {
        Self {
            entity,
            marker: PhantomData,
        }
    }
}

impl<C: Component> Debug for ActionOf<C> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.debug_struct("ActionOf")
            .field("entity", &self.entity)
            .finish()
    }
}

impl<C: Component> Clone for ActionOf<C> {
    fn clone(&self) -> Self {
        Self {
            entity: self.entity,
            marker: PhantomData,
        }
    }
}

impl<C: Component> PartialEq for ActionOf<C> {
    fn eq(&self, other: &Self) -> bool {
        self.entity == other.entity
    }
}

impl<C: Component> Eq for ActionOf<C> {}

#[cfg(feature = "reflect")]
impl<C: Component> bevy::reflect::TypePath for ActionOf<C> {
    fn type_path() -> &'static str {
        use bevy::reflect::utility::GenericTypePathCell;
        static CELL: GenericTypePathCell = GenericTypePathCell::new();
        CELL.get_or_insert::<Self, _>(|| {
            format!(
                concat!(::core::module_path!(), "::ActionOf<{}>"),
                ::core::any::type_name::<C>()
            )
        })
    }

    fn short_type_path() -> &'static str {
        use bevy::reflect::utility::GenericTypePathCell;
        static CELL: GenericTypePathCell = GenericTypePathCell::new();
        CELL.get_or_insert::<Self, _>(|| format!("ActionOf<{}>", ::core::any::type_name::<C>()))
    }

    fn type_ident() -> Option<&'static str> {
        ::core::option::Option::Some("ActionOf")
    }

    fn module_path() -> Option<&'static str> {
        ::core::option::Option::Some(::core::module_path!())
    }

    fn crate_name() -> Option<&'static str> {
        ::core::option::Option::Some(::core::module_path!().split(':').next().unwrap())
    }
}

/// Action entities associated with context `C`.
///
/// See also the [`actions!`](crate::prelude::actions) macro for conveniently spawning associated actions.
#[derive(Component, Deref)]
#[cfg_attr(
    feature = "reflect",
    derive(Reflect),
    reflect(Component, type_path = false)
)]
#[relationship_target(relationship = ActionOf<C>, linked_spawn)]
pub struct Actions<C: Component> {
    #[deref]
    #[relationship]
    entities: Vec<Entity>,
    #[cfg_attr(feature = "reflect", reflect(ignore))]
    marker: PhantomData<C>,
}

impl<'a, C: Component> IntoIterator for &'a Actions<C> {
    type Item = Entity;
    type IntoIter = Copied<slice::Iter<'a, Entity>>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<C: Component> Default for Actions<C> {
    fn default() -> Self {
        Self {
            entities: Default::default(),
            marker: PhantomData,
        }
    }
}

impl<C: Component> Debug for Actions<C> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.debug_struct("Actions")
            .field("entities", &self.entities)
            .finish()
    }
}

impl<C: Component> Clone for Actions<C> {
    fn clone(&self) -> Self {
        Self {
            entities: self.entities.clone(),
            marker: PhantomData,
        }
    }
}

impl<C: Component> PartialEq for Actions<C> {
    fn eq(&self, other: &Self) -> bool {
        self.entities == other.entities
    }
}

impl<C: Component> Eq for Actions<C> {}

#[cfg(feature = "reflect")]
impl<C: Component> bevy::reflect::TypePath for Actions<C> {
    fn type_path() -> &'static str {
        use bevy::reflect::utility::GenericTypePathCell;
        static CELL: GenericTypePathCell = GenericTypePathCell::new();
        CELL.get_or_insert::<Self, _>(|| {
            format!(
                concat!(::core::module_path!(), "::Actions<{}>"),
                ::core::any::type_name::<C>()
            )
        })
    }

    fn short_type_path() -> &'static str {
        use bevy::reflect::utility::GenericTypePathCell;
        static CELL: GenericTypePathCell = GenericTypePathCell::new();
        CELL.get_or_insert::<Self, _>(|| format!("Actions<{}>", ::core::any::type_name::<C>()))
    }

    fn type_ident() -> Option<&'static str> {
        ::core::option::Option::Some("Actions")
    }

    fn module_path() -> Option<&'static str> {
        ::core::option::Option::Some(::core::module_path!())
    }

    fn crate_name() -> Option<&'static str> {
        ::core::option::Option::Some(::core::module_path!().split(':').next().unwrap())
    }
}

/// A type alias over [`RelatedSpawner`] used to spawn action entities containing an [`ActionOf`] relationship.
pub type ActionSpawner<'w, C> = RelatedSpawner<'w, ActionOf<C>>;

/// A type alias over [`RelatedSpawnerCommands`] used to spawn action entities containing an [`ActionOf`] relationship.
pub type ActionSpawnerCommands<'w, C> = RelatedSpawnerCommands<'w, ActionOf<C>>;

/// Returns a [`SpawnRelatedBundle`](bevy::ecs::spawn::SpawnRelatedBundle) that will insert the [`Actions<C>`] component and
/// spawn a [`SpawnableList`](bevy::ecs::spawn::SpawnableList) of entities with given bundles that relate to the context entity
/// via the [`ActionOf<C>`] component.
///
/// Similar to [`related!`], but instead of specifying [`Actions<C>`], you only write `C` itself.
///
/// See also [`bindings!`](crate::prelude::bindings).
///
/// # Examples
///
/// A List of context actions with a component. You usually spawn actions with at least [`Bindings`](crate::prelude::Bindings),
/// but actions alone could be used for networking or for later mocking.
///
/// ```
/// # use bevy::prelude::*;
/// # use bevy_enhanced_input::prelude::*;
/// # let mut world = World::new();
/// world.spawn(actions!(Player[
///     Action::<Fire>::new(),
///     Action::<Jump>::new()
/// ]));
/// # assert_eq!(world.entities().count_spawned(), 3);
/// # #[derive(Component)]
/// # struct Player;
/// # #[derive(InputAction)]
/// # #[action_output(bool)]
/// # struct Fire;
/// # #[derive(InputAction)]
/// # #[action_output(bool)]
/// # struct Jump;
/// ```
///
/// A single context action with components.
///
/// ```
/// # use bevy::prelude::*;
/// # use bevy_enhanced_input::prelude::*;
/// # let mut world = World::new();
/// world.spawn(actions!(Player[(
///     Action::<Fire>::new(),
///     bindings![MouseButton::Left],
/// )]));
/// # assert_eq!(world.entities().count_spawned(), 3);
/// # #[derive(Component)]
/// # struct Player;
/// # #[derive(InputAction)]
/// # #[action_output(bool)]
/// # struct Fire;
/// ```
///
/// A List of context actions with multiple components.
///
/// ```
/// # use bevy::prelude::*;
/// # use bevy_enhanced_input::prelude::*;
/// # let mut world = World::new();
/// world.spawn(actions!(Player[
///     (
///         Action::<Movement>::new(),
///         Bindings::spawn(Cardinal::wasd_keys()),
///     ),
///     (
///         Action::<Jump>::new(),
///         bindings![KeyCode::Space],
///     ),
/// ]));
/// # assert_eq!(world.entities().count_spawned(), 8);
/// # #[derive(Component)]
/// # struct Player;
/// # #[derive(InputAction)]
/// # #[action_output(Vec2)]
/// # struct Movement;
/// # #[derive(InputAction)]
/// # #[action_output(bool)]
/// # struct Jump;
/// ```
#[macro_export]
macro_rules! actions {
    ($context:ty [$($action:expr),*$(,)?]) => {
        ::bevy::prelude::related!($crate::prelude::Actions<$context>[$($action),*])
    };
}
