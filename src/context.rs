pub mod input_reader;
mod instance;
pub mod time;
mod trigger_tracker;

use core::{
    any::{self, TypeId},
    cmp::{Ordering, Reverse},
    marker::PhantomData,
};

#[cfg(test)]
use bevy::ecs::system::SystemState;
use bevy::{
    ecs::{
        component::ComponentId,
        entity_disabling::Disabled,
        schedule::ScheduleLabel,
        system::{ParamBuilder, QueryParamBuilder},
        world::{FilteredEntityMut, FilteredEntityRef},
    },
    prelude::*,
};
use log::{debug, trace};
#[cfg(feature = "serialize")]
use serde::{Deserialize, Serialize};

use crate::{
    action::fns::ActionFns,
    binding::FirstActivation,
    condition::fns::{ConditionFns, ConditionRegistry},
    context::{input_reader::PendingBindings, trigger_tracker::TriggerTracker},
    modifier::fns::{ModifierFns, ModifierRegistry},
    prelude::*,
};
use input_reader::InputReader;
use instance::ContextInstances;

/// An extension trait for [`App`] to assign input to components.
pub trait InputContextAppExt {
    /// Registers type `C` as an input context, whose actions will be evaluated during [`PreUpdate`].
    ///
    /// Action evaluation follows these steps:
    ///
    /// - If the action has an [`ActionMock`] component, use the mocked [`ActionValue`] and [`ActionState`] directly.
    /// - Otherwise, evaluate the action from its bindings:
    ///     1. Iterate over each binding from the [`Bindings`] component.
    ///         1. Read the binding input as an [`ActionValue`], or [`ActionValue::zero`] if the input was already consumed by another action.
    ///            The enum variant depends on the input source.
    ///         2. Apply all binding-level [`InputModifier`]s.
    ///         3. Evaluate all input-level [`InputCondition`]s, combining their results based on their [`InputCondition::kind`].
    ///     2. Select all [`ActionValue`]s with the most significant [`ActionState`] and combine them using the
    ///        [`ActionSettings::accumulation`] strategy.
    ///     3. Convert the combined value to [`ActionOutput::DIM`] using [`ActionValue::convert`].
    ///     4. Apply all action-level [`InputModifier`]s.
    ///     5. Evaluate all action-level [`InputCondition`]s, combining their results based on their [`InputCondition::kind`].
    ///     6. Convert the final value to [`ActionOutput::DIM`] again using [`ActionValue::convert`].
    ///     7. Apply the resulting [`ActionState`] and [`ActionValue`] to the action entity.
    ///     8. If the final state is not [`ActionState::None`], consume the binding input value.
    ///
    /// This logic may look complicated, but you don't have to memorize it. It behaves surprisingly intuitively.
    fn add_input_context<C: Component>(&mut self) -> &mut Self {
        self.add_input_context_to::<PreUpdate, C>()
    }

    /// Like [`Self::add_input_context`], but allows specifying the schedule
    /// in which the context's actions will be evaluated.
    ///
    /// For example, if your game logic runs inside [`FixedMain`](bevy::app::FixedMain), you can set the schedule
    /// to [`FixedPreUpdate`]. This way, if the schedule runs multiple times per frame, events like [`Started`] or
    /// [`Completed`] will be triggered only once per schedule run.
    fn add_input_context_to<S: ScheduleLabel + Default, C: Component>(&mut self) -> &mut Self;
}

impl InputContextAppExt for App {
    fn add_input_context_to<S: ScheduleLabel + Default, C: Component>(&mut self) -> &mut Self {
        debug!(
            "registering `{}` for `{}`",
            any::type_name::<C>(),
            any::type_name::<S>(),
        );

        let actions_id = self.world_mut().register_component::<Actions<C>>();
        let activity_id = self.world_mut().register_component::<ContextActivity<C>>();
        let mut registry = self.world_mut().resource_mut::<ContextRegistry>();
        if let Some(contexts) = registry
            .iter_mut()
            .find(|c| c.schedule_id == TypeId::of::<S>())
        {
            debug_assert!(
                !contexts.actions_ids.contains(&actions_id),
                "context `{}` shouldn't be added more then once",
                any::type_name::<C>()
            );
            contexts.actions_ids.push(actions_id);
            contexts.activity_ids.push(activity_id);
        } else {
            let mut contexts = ScheduleContexts::new::<S>();
            contexts.actions_ids.push(actions_id);
            contexts.activity_ids.push(activity_id);
            registry.push(contexts);
        }

        let _ = self.try_register_required_components::<C, ContextPriority<C>>();
        let _ = self.try_register_required_components::<C, ContextActivity<C>>();

        self.add_observer(register::<C, S>)
            .add_observer(unregister::<C, S>)
            .add_observer(reset_action::<C>)
            .add_observer(deactivate::<C>);

        self
    }
}

/// Tracks registered input contexts for each schedule.
///
/// In Bevy, it's impossible to know which schedule is used inside a system,
/// so we genericize update systems over schedules.
///
/// This resource stores registered contexts per-schedule in a type-erased way
/// to perform the setup after all registrations in [`App::finish`].
///
/// Exists only during the plugin initialization.
#[derive(Resource, Default, Deref, DerefMut)]
pub(crate) struct ContextRegistry(Vec<ScheduleContexts>);

pub(crate) struct ScheduleContexts {
    /// Schedule ID for which all actions were registered.
    schedule_id: TypeId,

    /// IDs of [`Actions<C>`].
    actions_ids: Vec<ComponentId>,

    /// IDs of [`ContextActivity<C>`].
    activity_ids: Vec<ComponentId>,

    /// Configures the app for this schedule.
    setup: fn(&Self, &mut App, &ConditionRegistry, &ModifierRegistry),
}

impl ScheduleContexts {
    /// Creates a new instance for schedule `S`.
    ///
    /// [`Self::setup`] will configure the app for `S`.
    #[must_use]
    fn new<S: ScheduleLabel + Default>() -> Self {
        Self {
            schedule_id: TypeId::of::<S>(),
            actions_ids: Default::default(),
            activity_ids: Default::default(),
            // Since the type is not present in the function signature, we can store
            // functions for specific type without making the struct generic.
            setup: Self::setup_typed::<S>,
        }
    }

    /// Calls [`Self::setup_typed`] for `S` that was associated in [`Self::new`].
    pub(crate) fn setup(
        &self,
        app: &mut App,
        conditions: &ConditionRegistry,
        modifiers: &ModifierRegistry,
    ) {
        (self.setup)(self, app, conditions, modifiers);
    }

    /// Configures the app for all contexts registered for schedule `C`.
    pub(crate) fn setup_typed<S: ScheduleLabel + Default>(
        &self,
        app: &mut App,
        conditions: &ConditionRegistry,
        modifiers: &ModifierRegistry,
    ) {
        debug!("setting up systems for `{}`", any::type_name::<S>());

        let update_fn = (
            ParamBuilder,
            ParamBuilder,
            ParamBuilder,
            ParamBuilder,
            QueryParamBuilder::new(|builder| {
                builder
                    .data::<Option<&GamepadDevice>>()
                    .optional(|builder| {
                        for &id in &self.activity_ids {
                            builder.mut_id(id);
                        }
                        for &id in &self.actions_ids {
                            builder.mut_id(id);
                        }
                    });
            }),
            ParamBuilder,
            ParamBuilder,
            ParamBuilder,
            QueryParamBuilder::new(|builder| {
                builder.optional(|builder| {
                    for &id in &**conditions {
                        builder.mut_id(id);
                    }
                    for &id in &**modifiers {
                        builder.mut_id(id);
                    }
                });
            }),
        )
            .build_state(app.world_mut())
            .build_system(update::<S>);

        let trigger_fn = (
            ParamBuilder,
            ParamBuilder,
            QueryParamBuilder::new(|builder| {
                builder.optional(|builder| {
                    for &id in &self.activity_ids {
                        builder.mut_id(id);
                    }
                    for &id in &self.actions_ids {
                        builder.ref_id(id);
                    }
                });
            }),
            ParamBuilder,
        )
            .build_state(app.world_mut())
            .build_system(apply::<S>);

        app.init_resource::<ContextInstances<S>>()
            .configure_sets(
                S::default(),
                (EnhancedInputSystems::Update, EnhancedInputSystems::Apply).chain(),
            )
            .add_systems(
                S::default(),
                (
                    update_fn.in_set(EnhancedInputSystems::Update),
                    trigger_fn.in_set(EnhancedInputSystems::Apply),
                ),
            );
    }
}

fn register<C: Component, S: ScheduleLabel>(
    add: On<Insert, ContextPriority<C>>,
    mut instances: ResMut<ContextInstances<S>>,
    // TODO Bevy 0.17: Use `Allows` filter instead of `Has`.
    contexts: Query<(&ContextPriority<C>, Has<Disabled>)>,
) {
    debug!(
        "registering `{}` to `{}`",
        any::type_name::<C>(),
        add.entity,
    );

    let (&priority, _) = contexts.get(add.entity).unwrap();
    instances.add::<C>(add.entity, *priority);
}

fn unregister<C: Component, S: ScheduleLabel>(
    add: On<Replace, ContextPriority<C>>,
    mut instances: ResMut<ContextInstances<S>>,
) {
    debug!(
        "unregistering `{}` from `{}`",
        any::type_name::<C>(),
        add.entity,
    );

    instances.remove::<C>(add.entity);
}

fn deactivate<C: Component>(
    add: On<Insert, ContextActivity<C>>,
    mut pending: ResMut<PendingBindings>,
    contexts: Query<(&ContextActivity<C>, &Actions<C>)>,
    actions: Query<(&ActionSettings, &Bindings)>,
    bindings: Query<&Binding>,
) {
    let Ok((&active, context_actions)) = contexts.get(add.entity) else {
        return;
    };

    debug!(
        "setting activity of `{}` to `{}`",
        any::type_name::<C>(),
        *active,
    );

    if !*active {
        for (settings, action_bindings) in actions.iter_many(context_actions) {
            if settings.require_reset {
                pending.extend(bindings.iter_many(action_bindings).copied());
            }
        }
    }
}

/// Resets action data and triggers corresponding events on removal.
pub(crate) fn reset_action<C: Component>(
    add: On<Remove, ActionOf<C>>,
    mut commands: Commands,
    mut pending: ResMut<PendingBindings>,
    mut actions: Query<(
        &ActionOf<C>,
        &ActionSettings,
        &ActionFns,
        Option<&Bindings>,
        &mut ActionValue,
        &mut ActionState,
        &mut ActionEvents,
        &mut ActionTime,
    )>,
    bindings: Query<&Binding>,
) {
    let Ok((action_of, settings, fns, action_bindings, mut value, mut state, mut events, mut time)) =
        actions.get_mut(add.entity)
    else {
        trace!("ignoring reset for `{}`", add.entity);
        return;
    };

    *time = Default::default();
    events.set_if_neq(ActionEvents::new(*state, ActionState::None));
    state.set_if_neq(Default::default());
    value.set_if_neq(ActionValue::zero(value.dim()));

    fns.trigger(
        &mut commands,
        **action_of,
        add.entity,
        *state,
        *events,
        *value,
        *time,
    );

    if let Some(action_bindings) = action_bindings
        && settings.require_reset
    {
        pending.extend(bindings.iter_many(action_bindings).copied());
    }
}

/// Marks an [`Action<C>`] as manually mocked, skipping the [`EnhancedInputSet::Update`] logic for it.
///
/// This allows modifying any action data without its values being overridden during evaluation.
///
/// Takes precedence over [`ActionMock`], which drives specific [`ActionValue`] and [`ActionState`] during evaluation.
#[derive(Component)]
pub struct ExternallyMocked;

#[allow(clippy::too_many_arguments)]
fn update<S: ScheduleLabel>(
    mut consume_buffer: Local<Vec<Binding>>, // Consumed inputs during state evaluation.
    time: ContextTime,
    mut reader: InputReader,
    instances: Res<ContextInstances<S>>,
    mut contexts: Query<FilteredEntityMut>,
    mut actions: Query<
        (
            Entity,
            &Name,
            &ActionSettings,
            Option<&Bindings>,
            Option<&ModifierFns>,
            Option<&ConditionFns>,
            Option<&mut ActionMock>,
        ),
        Without<ExternallyMocked>,
    >,
    mut actions_data: Query<(
        &'static mut ActionValue,
        &'static mut ActionState,
        &'static mut ActionEvents,
        &'static mut ActionTime,
    )>,
    mut bindings: Query<
        (
            Entity,
            &Binding,
            &mut FirstActivation,
            Option<&ModifierFns>,
            Option<&ConditionFns>,
        ),
        Without<ActionSettings>,
    >,
    mut conds_and_mods: Query<FilteredEntityMut>,
) {
    reader.clear_consumed::<S>();

    for instance in &**instances {
        let Ok(mut context) = contexts.get_mut(instance.entity) else {
            trace!(
                "skipping updating `{}` on disabled `{}`",
                instance.name, instance.entity
            );
            continue;
        };

        let gamepad = context.get::<GamepadDevice>().copied().unwrap_or_default();
        let context_active = instance.is_active(&context.as_readonly());
        let Some(mut context_actions) = instance.actions_mut(&mut context) else {
            continue;
        };

        let mods_count = |action: &Entity| {
            let Ok((.., action_bindings, _, _, _)) = actions.get(*action) else {
                return Reverse(0);
            };

            let value = bindings
                .iter_many(action_bindings.into_iter().flatten())
                .map(|(_, b, ..)| b.mod_keys_count())
                .max()
                .unwrap_or(0);
            Reverse(value)
        };

        if !context_actions.is_sorted_by_key(mods_count) {
            context_actions.sort_by_cached_key(mods_count);
        }

        trace!("updating `{}` on `{}`", instance.name, instance.entity);

        reader.set_gamepad(gamepad);

        let mut actions_iter = actions.iter_many_mut(&*context_actions);
        while let Some((
            action,
            action_name,
            action_settings,
            action_bindings,
            modifiers,
            conditions,
            mock,
        )) = actions_iter.fetch_next()
        {
            let (new_state, new_value) = if !context_active {
                trace!("skipping updating `{action_name}` due to inactive context");
                let dim = actions_data.get(action).map(|(v, ..)| v.dim()).unwrap();
                (ActionState::None, ActionValue::zero(dim))
            } else if let Some(mut mock) = mock
                && mock.enabled
            {
                trace!("updating `{action_name}` from `{mock:?}`");
                let expired = match &mut mock.span {
                    MockSpan::Updates(ticks) => {
                        *ticks = ticks.saturating_sub(1);
                        *ticks == 0
                    }
                    MockSpan::Duration(duration) => {
                        *duration = duration.saturating_sub(time.delta());
                        trace!("reducing mock duration by {:?}", time.delta());
                        duration.is_zero()
                    }
                    MockSpan::Manual => false,
                };

                let new_state = mock.state;
                let new_value = mock.value;
                if expired {
                    mock.enabled = false;
                }

                (new_state, new_value)
            } else {
                trace!("updating `{action_name}` from input");

                let dim = actions_data.get(action).map(|(v, ..)| v.dim()).unwrap();
                let actions_data = actions_data.as_readonly();
                let mut tracker = TriggerTracker::new(ActionValue::zero(dim));
                let mut bindings_iter =
                    bindings.iter_many_mut(action_bindings.into_iter().flatten());
                while let Some((
                    binding_entity,
                    &binding,
                    mut first_activation,
                    modifiers,
                    conditions,
                )) = bindings_iter.fetch_next()
                {
                    let new_value = reader.value(binding);
                    if action_settings.require_reset && **first_activation {
                        // Ignore until we read zero for this mapping.
                        if new_value.as_bool() {
                            // Mark the binding input as consumed regardless of the end action state.
                            reader.consume::<S>(binding);
                            continue;
                        } else {
                            **first_activation = false;
                        }
                    }

                    let mut binding_entity = conds_and_mods.get_mut(binding_entity).unwrap();

                    let mut current_tracker = TriggerTracker::new(new_value);
                    trace!("reading value `{new_value:?}`");
                    if let Some(modifiers) = modifiers {
                        current_tracker.apply_modifiers(
                            &mut binding_entity,
                            &actions_data,
                            &time,
                            modifiers,
                        );
                    }
                    if let Some(conditions) = conditions {
                        current_tracker.apply_conditions(
                            &mut binding_entity,
                            &actions_data,
                            &time,
                            conditions,
                        );
                    }

                    let current_state = current_tracker.state();
                    if current_state == ActionState::None {
                        // Ignore non-active trackers to allow the action to fire even if all
                        // input-level conditions return `ActionState::None`. This ensures that an
                        // action-level condition or modifier can still trigger the action.
                        continue;
                    }

                    match current_state.cmp(&tracker.state()) {
                        Ordering::Less => (),
                        Ordering::Equal => {
                            tracker.combine(current_tracker, action_settings.accumulation);
                            if action_settings.consume_input {
                                consume_buffer.push(binding);
                            }
                        }
                        Ordering::Greater => {
                            tracker.overwrite(current_tracker);
                            if action_settings.consume_input {
                                consume_buffer.clear();
                                consume_buffer.push(binding);
                            }
                        }
                    }
                }

                let mut action = conds_and_mods.get_mut(action).unwrap();
                if let Some(modifiers) = modifiers {
                    tracker.apply_modifiers(&mut action, &actions_data, &time, modifiers);
                }
                if let Some(conditions) = conditions {
                    tracker.apply_conditions(&mut action, &actions_data, &time, conditions);
                }

                let new_state = tracker.state();
                let new_value = tracker.value().convert(dim);

                if action_settings.consume_input {
                    if new_state != ActionState::None {
                        for &binding in &consume_buffer {
                            reader.consume::<S>(binding);
                        }
                    }
                    consume_buffer.clear();
                }

                (new_state, new_value)
            };

            trace!("evaluated to `{new_state:?}` with `{new_value:?}`");

            let (mut value, mut state, mut events, mut action_time) =
                actions_data.get_mut(action).unwrap();

            action_time.update(time.delta_secs(), *state);
            events.set_if_neq(ActionEvents::new(*state, new_state));
            state.set_if_neq(new_state);
            value.set_if_neq(new_value);
        }
    }
}

pub type ActionsQuery<'w, 's> = Query<
    'w,
    's,
    (
        &'static ActionValue,
        &'static ActionState,
        &'static ActionEvents,
        &'static ActionTime,
    ),
>;

fn apply<S: ScheduleLabel>(
    mut commands: Commands,
    instances: Res<ContextInstances<S>>,
    contexts: Query<FilteredEntityRef, Without<ActionFns>>,
    mut actions: Query<EntityMut, With<ActionFns>>,
) {
    for instance in &**instances {
        let Ok(context) = contexts.get(instance.entity) else {
            trace!(
                "skipping triggering for `{}` on disabled `{}`",
                instance.name, instance.entity,
            );
            continue;
        };
        let Some(context_actions) = instance.actions(&context) else {
            continue;
        };

        trace!(
            "running triggers for `{}` on `{}`",
            instance.name, instance.entity,
        );

        let mut actions_iter = actions.iter_many_mut(context_actions);
        while let Some(mut action) = actions_iter.fetch_next() {
            let fns = *action.get::<ActionFns>().unwrap();
            let value = *action.get::<ActionValue>().unwrap();
            fns.store_value(&mut action, value);

            let state = *action.get::<ActionState>().unwrap();
            let events = *action.get::<ActionEvents>().unwrap();
            let time = *action.get::<ActionTime>().unwrap();
            fns.trigger(
                &mut commands,
                context.id(),
                action.id(),
                state,
                events,
                value,
                time,
            );
        }
    }
}

/// Enables or disables all action updates from inputs and mocks for context `C`.
///
/// By default, all contexts are active.
///
/// Inserting [`Self::INACTIVE`] is similar to removing the context. It transitions all context action states
/// to [`ActionState::None`] with [`ActionValue::zero`], triggering the corresponding events.
/// For each action where [`ActionSettings::require_reset`] is set, it will require inputs for its bindings
/// to be inactive before they will be visible to actions from other contexts.
///
/// This is analogous to hiding an entity instead of despawning.
/// Use this component when you want to toggle quickly, preserve bindings, or keep entity IDs.
/// Use removal when the context is truly going away and you don't need it back soon.
///
/// Marked as required for `C` on context registration.
#[derive(Component, Reflect, Deref)]
#[component(immutable)]
pub struct ContextActivity<C> {
    #[deref]
    active: bool,
    #[reflect(ignore)]
    marker: PhantomData<C>,
}

impl<C> ContextActivity<C> {
    /// Active context.
    pub const ACTIVE: Self = Self::new(true);

    /// Inactive context.
    pub const INACTIVE: Self = Self::new(false);

    /// Creates a new instance with the given value.
    #[must_use]
    pub const fn new(active: bool) -> Self {
        Self {
            active,
            marker: PhantomData,
        }
    }

    /// Returns a new instance with the value inverted.
    #[must_use]
    pub const fn toggled(self) -> Self {
        if self.active {
            Self::INACTIVE
        } else {
            Self::ACTIVE
        }
    }
}

impl<C> Default for ContextActivity<C> {
    fn default() -> Self {
        Self::ACTIVE
    }
}

impl<C> Clone for ContextActivity<C> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<C> Copy for ContextActivity<C> {}

/// Determines the evaluation order of the input context `C` on the entity.
///
/// Used to control how contexts are layered, as some [`Action<C>`]s may consume inputs.
///
/// The ordering applies per schedule: contexts in schedules that run earlier are evaluated first.
/// Within the same schedule, contexts with a higher priority are evaluated first.
///
/// Ordering matters because actions may "consume" inputs, making them unavailable to other actions
/// until the context that consumed them is evaluated again. This allows contexts layering, where
/// some actions take priority over others. This behavior can be customized per-action by setting
/// [`ActionSettings::consume_input`] to `false`.
///
/// Marked as required for `C` on context registration.
///
/// # Examples
///
/// ```
/// use bevy::prelude::*;
/// use bevy_enhanced_input::prelude::*;
///
/// # let mut world = World::new();
/// world.spawn((
///     OnFoot,
///     InCar,
///     ContextPriority::<InCar>::new(1), // `InCar` context will be evaluated earlier.
///     // Actions...
/// ));
///
/// #[derive(Component)]
/// struct OnFoot;
///
/// #[derive(Component)]
/// struct InCar;
/// ```
#[derive(Component, Reflect, Deref)]
#[component(immutable)]
pub struct ContextPriority<C> {
    #[deref]
    value: usize,
    #[reflect(ignore)]
    marker: PhantomData<C>,
}

impl<C> ContextPriority<C> {
    pub const fn new(value: usize) -> Self {
        Self {
            value,
            marker: PhantomData,
        }
    }
}

impl<C> Default for ContextPriority<C> {
    fn default() -> Self {
        Self::new(0)
    }
}

impl<C> Clone for ContextPriority<C> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<C> Copy for ContextPriority<C> {}

/// Associated gamepad for all input contexts on this entity.
///
/// If not present, input will be read from all connected gamepads.
#[derive(Component, Reflect, Debug, Default, Hash, PartialEq, Eq, Clone, Copy)]
#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serialize", reflect(Serialize, Deserialize))]
pub enum GamepadDevice {
    /// Matches input from any gamepad.
    ///
    /// For an axis, the [`ActionValue`] will be calculated as the sum of inputs from all gamepads.
    /// For a button, the [`ActionValue`] will be `true` if any gamepad has this button pressed.
    #[default]
    Any,
    /// Matches input from specific gamepad.
    Single(Entity),
    /// Ignores all gamepad input.
    None,
}

impl From<Entity> for GamepadDevice {
    fn from(value: Entity) -> Self {
        Self::Single(value)
    }
}

impl From<Option<Entity>> for GamepadDevice {
    fn from(value: Option<Entity>) -> Self {
        match value {
            Some(entity) => GamepadDevice::Single(entity),
            None => GamepadDevice::None,
        }
    }
}

/// Helper for tests to simplify [`InputTime`] and [`ActionsQuery`] creation.
#[cfg(test)]
pub(crate) fn init_world<'w, 's>() -> (World, SystemState<(ContextTime<'w>, ActionsQuery<'w, 's>)>)
{
    let mut world = World::new();
    world.init_resource::<Time>();
    world.init_resource::<Time<Real>>();

    let state = SystemState::<(ContextTime, ActionsQuery)>::new(&mut world);

    (world, state)
}
