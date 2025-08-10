use core::{
    any::{self, TypeId},
    cmp::Reverse,
    marker::PhantomData,
};

use bevy::{
    ecs::{
        schedule::ScheduleLabel,
        world::{FilteredEntityMut, FilteredEntityRef},
    },
    prelude::*,
};

use crate::prelude::*;

/// Stores information about instantiated contexts for a schedule `S`.
///
/// Used to iterate over them in a defined order and operate in a type-erased manner.
#[derive(Resource, Default, Deref)]
pub(crate) struct ContextInstances<S: ScheduleLabel> {
    #[deref]
    instances: Vec<ContextInstance>,
    marker: PhantomData<S>,
}

impl<S: ScheduleLabel> ContextInstances<S> {
    pub(super) fn add<C: Component>(&mut self, entity: Entity, priority: usize) {
        let instance = ContextInstance::new::<C>(entity, priority);
        match self.binary_search_by_key(&Reverse(priority), |inst| Reverse(inst.priority)) {
            Ok(index) => {
                // Insert last to preserve entry creation order.
                let last_priority_index = self
                    .iter()
                    .skip(index + 1)
                    .position(|inst| inst.priority != priority)
                    .unwrap_or_default();
                self.instances
                    .insert(index + last_priority_index + 1, instance);
            }
            Err(index) => self.instances.insert(index, instance),
        };
    }

    pub(super) fn remove<C: Component>(&mut self, entity: Entity) {
        let index = self
            .iter()
            .position(|inst| inst.entity == entity && inst.type_id == TypeId::of::<C>())
            .expect("context instance should be created before removal");
        self.instances.remove(index);
    }
}

/// Meta information for context on an entity.
pub(crate) struct ContextInstance {
    pub(super) entity: Entity,
    pub(super) name: &'static str,
    type_id: TypeId,
    priority: usize,
    actions: for<'a> fn(&Self, &'a FilteredEntityRef) -> Option<&'a [Entity]>,
    actions_mut: for<'a> fn(&Self, &'a mut FilteredEntityMut) -> Option<Mut<'a, [Entity]>>,
}

impl ContextInstance {
    /// Creates a new instance for context `C`.
    #[must_use]
    fn new<C: Component>(entity: Entity, priority: usize) -> Self {
        Self {
            entity,
            name: any::type_name::<C>(),
            type_id: TypeId::of::<C>(),
            priority,
            actions: Self::actions_typed::<C>,
            actions_mut: Self::actions_mut_typed::<C>,
        }
    }

    /// Returns a reference to entities from [`Actions<C>`], for which this instance was created.
    pub(super) fn actions<'a>(&self, context: &'a FilteredEntityRef) -> Option<&'a [Entity]> {
        (self.actions)(self, context)
    }

    /// Returns a mutable reference to entities from [`Actions<C>`], for which this instance was created.
    ///
    /// Used only to sort entities.
    pub(super) fn actions_mut<'a>(
        &self,
        context: &'a mut FilteredEntityMut,
    ) -> Option<Mut<'a, [Entity]>> {
        (self.actions_mut)(self, context)
    }

    fn actions_typed<'a, C: Component>(
        &self,
        context: &'a FilteredEntityRef,
    ) -> Option<&'a [Entity]> {
        context.get::<Actions<C>>().map(|actions| &***actions)
    }

    fn actions_mut_typed<'a, C: Component>(
        &self,
        context: &'a mut FilteredEntityMut,
    ) -> Option<Mut<'a, [Entity]>> {
        context
            .get_mut::<Actions<C>>()
            .map(|a| a.map_unchanged(|a| &mut **a.collection_mut_risky()))
    }
}
