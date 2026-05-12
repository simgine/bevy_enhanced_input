//! Custom binding sources.
//!
//! A [`BindingSource`] component attached to a binding entity with [`Binding::Custom`]
//! produces an [`ActionValue`] that flows through the normal modifier/condition pipeline.
//! Register with [`BindingSourceAppExt::add_binding_source`].

use core::{fmt::Debug, ptr};

use bevy::{
    ecs::{
        component::{ComponentId, Mutable},
        entity_disabling::Disabled,
        world::FilteredEntityMut,
    },
    prelude::*,
};

use crate::prelude::*;

/// Produces an [`ActionValue`] for a [`Binding::Custom`] entry on the same binding entity.
///
/// Register with [`BindingSourceAppExt::add_binding_source`].
pub trait BindingSource: Debug {
    /// Returns the current value of this source. Called once per binding evaluation.
    fn read(
        &mut self,
        actions: &ActionsQuery,
        time: &ContextTime,
    ) -> ActionValue;
}

pub trait BindingSourceAppExt {
    /// Registers a binding source, making it accessible during context evaluation.
    fn add_binding_source<S: BindingSource + Component<Mutability = Mutable>>(
        &mut self,
    ) -> &mut Self;
}

impl BindingSourceAppExt for App {
    fn add_binding_source<S: BindingSource + Component<Mutability = Mutable>>(
        &mut self,
    ) -> &mut Self {
        let id = self.world_mut().register_component::<S>();
        let mut registry = self.world_mut().resource_mut::<BindingSourceRegistry>();
        registry.0.push(id);

        self.add_observer(register_source::<S>)
            .add_observer(unregister_source::<S>)
            .register_required_components::<S, BindingSourceFns>()
    }
}

fn register_source<S: BindingSource + Component<Mutability = Mutable>>(
    add: On<Add, S>,
    mut sources: Query<&mut BindingSourceFns, Allow<Disabled>>,
) {
    let mut fns = sources.get_mut(add.entity).unwrap();
    fns.0.push(get_source::<S>);
}

fn unregister_source<S: BindingSource + Component<Mutability = Mutable>>(
    remove: On<Remove, S>,
    mut sources: Query<&mut BindingSourceFns, Allow<Disabled>>,
) {
    let mut fns = sources.get_mut(remove.entity).unwrap();
    let index = fns
        .iter()
        .position(|&f| ptr::fn_addr_eq(f, get_source::<S> as GetSourceFn))
        .unwrap();
    fns.0.remove(index);
}

/// IDs of all registered binding sources.
///
/// Used to dynamically register access for [`FilteredEntityMut`].
///
/// Exists only during the plugin initialization.
#[derive(Resource, Deref, Default)]
pub(crate) struct BindingSourceRegistry(pub(crate) Vec<ComponentId>);

/// Functions to retrieve binding source components currently present on the entity.
///
/// Updated automatically using triggers.
#[derive(Component, Deref, Default)]
pub(crate) struct BindingSourceFns(Vec<GetSourceFn>);

type GetSourceFn = for<'a> fn(&'a mut FilteredEntityMut) -> &'a mut dyn BindingSource;

fn get_source<'a, S: BindingSource + Component<Mutability = Mutable>>(
    entity: &'a mut FilteredEntityMut,
) -> &'a mut dyn BindingSource {
    entity.get_mut::<S>().unwrap().into_inner()
}
