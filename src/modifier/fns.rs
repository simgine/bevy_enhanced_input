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

pub trait InputModifierAppExt {
    /// Registers an input modifier, making it accessible during context evaluation.
    ///
    /// All built-in modifiers are already registered.
    fn add_input_modifier<M: InputModifier + Component<Mutability = Mutable>>(
        &mut self,
    ) -> &mut Self;
}

impl InputModifierAppExt for App {
    fn add_input_modifier<M: InputModifier + Component<Mutability = Mutable>>(
        &mut self,
    ) -> &mut Self {
        let id = self.world_mut().register_component::<M>();
        let mut registry = self.world_mut().resource_mut::<ModifierRegistry>();
        registry.0.push(id);

        self.add_observer(register_modifier::<M>)
            .add_observer(unregister_modifier::<M>)
            .register_required_components::<M, ModifierFns>()
    }
}

fn register_modifier<M: InputModifier + Component<Mutability = Mutable>>(
    add: On<Add, M>,
    mut modifiers: Query<&mut ModifierFns, Allow<Disabled>>,
) {
    let mut fns = modifiers.get_mut(add.entity).unwrap();
    fns.0.push(get_modifier::<M>);
}

fn unregister_modifier<M: InputModifier + Component<Mutability = Mutable>>(
    remove: On<Remove, M>,
    mut modifiers: Query<&mut ModifierFns, Allow<Disabled>>,
) {
    let mut fns = modifiers.get_mut(remove.entity).unwrap();
    let index = fns
        .iter()
        .position(|&f| ptr::fn_addr_eq(f, get_modifier::<M> as GetModifierFn))
        .unwrap();
    fns.0.remove(index);
}

/// IDs of all registered input modifiers.
///
/// Used to dynamically register access for [`FilteredEntityMut`].
///
/// Exists only during the plugin initialization.
#[derive(Resource, Deref, Default)]
pub(crate) struct ModifierRegistry(Vec<ComponentId>);

/// Functions to retrieve modifier components currently present on the entity as the trait.
///
/// Since we don't know the exact modifiers on an entity ahead of time,
/// we dynamically get them as the trait from [`FilteredEntityMut`].
///
/// Updated automatically using triggers.
#[derive(Component, Deref, Default)]
pub(crate) struct ModifierFns(Vec<GetModifierFn>);

type GetModifierFn = for<'a> fn(&'a mut FilteredEntityMut) -> &'a mut dyn InputModifier;

fn get_modifier<'a, C: InputModifier + Component<Mutability = Mutable>>(
    entity: &'a mut FilteredEntityMut,
) -> &'a mut dyn InputModifier {
    entity.get_mut::<C>().unwrap().into_inner()
}
