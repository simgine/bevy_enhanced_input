use core::ptr;

use bevy::{
    ecs::{
        component::{ComponentId, Mutable},
        entity_disabling::Disabled,
        world::FilteredEntityMut,
    },
    prelude::*,
};

use crate::prelude::*;

pub trait InputConditionAppExt {
    /// Registers an input condition, making it accessible during context evaluation.
    ///
    /// All built-in conditions are already registered.
    fn add_input_condition<C: InputCondition + Component<Mutability = Mutable>>(
        &mut self,
    ) -> &mut Self;
}

impl InputConditionAppExt for App {
    fn add_input_condition<C: InputCondition + Component<Mutability = Mutable>>(
        &mut self,
    ) -> &mut Self {
        let id = self.world_mut().register_component::<C>();
        let mut registry = self.world_mut().resource_mut::<ConditionRegistry>();
        registry.0.push(id);

        self.add_observer(register_condition::<C>)
            .add_observer(unregister_condition::<C>)
            .register_required_components::<C, ConditionFns>()
    }
}

fn register_condition<C: InputCondition + Component<Mutability = Mutable>>(
    add: On<Add, C>,
    mut conditions: Query<&mut ConditionFns, Allow<Disabled>>,
) {
    let mut fns = conditions.get_mut(add.entity).unwrap();
    fns.0.push(get_condition::<C>);
}

fn unregister_condition<C: InputCondition + Component<Mutability = Mutable>>(
    remove: On<Remove, C>,
    mut conditions: Query<&mut ConditionFns, Allow<Disabled>>,
) {
    let mut fns = conditions.get_mut(remove.entity).unwrap();
    let index = fns
        .iter()
        .position(|&f| ptr::fn_addr_eq(f, get_condition::<C> as GetConditionFn))
        .unwrap();
    fns.0.remove(index);
}

/// IDs of all registered input conditions.
///
/// Used to dynamically register access for [`FilteredEntityMut`].
///
/// Exists only during the plugin initialization.
#[derive(Resource, Deref, Default)]
pub(crate) struct ConditionRegistry(Vec<ComponentId>);

/// Functions to retrieve condition components currently present on the entity.
///
/// Since we don't know the exact conditions on an entity ahead of time,
/// we dynamically get them as the trait from [`FilteredEntityMut`].
///
/// Updated automatically using triggers.
#[derive(Component, Deref, Default)]
pub(crate) struct ConditionFns(Vec<GetConditionFn>);

type GetConditionFn = for<'a> fn(&'a mut FilteredEntityMut) -> &'a mut dyn InputCondition;

fn get_condition<'a, C: InputCondition + Component<Mutability = Mutable>>(
    entity: &'a mut FilteredEntityMut,
) -> &'a mut dyn InputCondition {
    entity.get_mut::<C>().unwrap().into_inner()
}
