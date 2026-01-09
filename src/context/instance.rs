use core::{any::TypeId, cmp::Reverse, marker::PhantomData};

use bevy::{
    ecs::{
        schedule::ScheduleLabel,
        world::{FilteredEntityMut, FilteredEntityRef},
    },
    prelude::*,
};

use crate::{context::ContextActivity, prelude::*};

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
        let index = self
            .binary_search_by_key(&Reverse(priority), |inst| Reverse(inst.priority))
            .unwrap_or_else(|i| i);
        self.instances.insert(index, instance);
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
    pub(super) name: ShortName<'static>,
    type_id: TypeId,
    priority: usize,
    is_active: fn(&Self, &FilteredEntityRef) -> bool,
    actions: for<'a> fn(&Self, &'a FilteredEntityRef) -> Option<&'a [Entity]>,
    actions_mut: for<'a> fn(&Self, &'a mut FilteredEntityMut) -> Option<Mut<'a, [Entity]>>,
}

impl ContextInstance {
    /// Creates a new instance for context `C`.
    #[must_use]
    fn new<C: Component>(entity: Entity, priority: usize) -> Self {
        Self {
            entity,
            name: ShortName::of::<C>(),
            type_id: TypeId::of::<C>(),
            priority,
            is_active: Self::is_active_typed::<C>,
            actions: Self::actions_typed::<C>,
            actions_mut: Self::actions_mut_typed::<C>,
        }
    }

    /// Returns the value from [`ContextActivity<C>`].
    pub(super) fn is_active(&self, context: &FilteredEntityRef) -> bool {
        (self.is_active)(self, context)
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

    pub(super) fn is_active_typed<C: Component>(&self, context: &FilteredEntityRef) -> bool {
        context
            .get::<ContextActivity<C>>()
            .is_some_and(|&active| *active)
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
