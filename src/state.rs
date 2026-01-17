/*!
State-context integration for synchronizing [`bevy_state`](bevy::state) with input contexts.

This module provides components that automatically activate/deactivate
input contexts based on the current application state, eliminating manual
`OnEnter`/`DespawnOnExit` boilerplate.

# Example

```ignore
use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;

#[derive(States, Clone, PartialEq, Eq, Hash, Debug, Default)]
enum GameMode {
    #[default]
    Playing,
    Paused,
}

#[derive(Component)]
struct PlayerContext;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, EnhancedInputPlugin))
        .init_state::<GameMode>()
        .add_input_context::<PlayerContext>()
        .sync_context_to_state::<GameMode, PlayerContext>()
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn((
        PlayerContext,
        ActiveInState::<GameMode, PlayerContext>::new(GameMode::Playing),
        actions!(PlayerContext[...]),
    ));
}
```
*/

use alloc::vec::Vec;
use core::marker::PhantomData;

use bevy::{
    prelude::*,
    state::state::{StateTransitionEvent, States},
};
use log::debug;

use crate::prelude::ContextActivity;

/// Activates input context `C` only when state `S` matches the specified value.
///
/// When the state matches, [`ContextActivity::<C>::ACTIVE`] is inserted.
/// When the state doesn't match, [`ContextActivity::<C>::INACTIVE`] is inserted.
#[derive(Component, Reflect)]
pub struct ActiveInState<S: States, C: Component> {
    state: S,
    #[reflect(ignore)]
    _marker: PhantomData<C>,
}

impl<S: States, C: Component> ActiveInState<S, C> {
    /// Creates a new instance for the given state.
    #[must_use]
    pub fn new(state: S) -> Self {
        Self {
            state,
            _marker: PhantomData,
        }
    }

    /// Returns `true` if the current state matches.
    #[must_use]
    pub fn matches(&self, current: &S) -> bool {
        *current == self.state
    }
}

impl<S: States, C: Component> Clone for ActiveInState<S, C> {
    fn clone(&self) -> Self {
        Self {
            state: self.state.clone(),
            _marker: PhantomData,
        }
    }
}

/// Activates input context `C` when state `S` matches ANY of the specified values.
///
/// Useful for contexts that should be active across multiple states.
#[derive(Component, Reflect)]
pub struct ActiveInStates<S: States, C: Component> {
    states: Vec<S>,
    #[reflect(ignore)]
    _marker: PhantomData<C>,
}

impl<S: States, C: Component> ActiveInStates<S, C> {
    /// Creates a new instance for the given states.
    #[must_use]
    pub fn new(states: impl IntoIterator<Item = S>) -> Self {
        Self {
            states: states.into_iter().collect(),
            _marker: PhantomData,
        }
    }

    /// Returns `true` if the current state matches any of the states.
    #[must_use]
    pub fn matches(&self, current: &S) -> bool {
        self.states.contains(current)
    }
}

impl<S: States, C: Component> Clone for ActiveInStates<S, C> {
    fn clone(&self) -> Self {
        Self {
            states: self.states.clone(),
            _marker: PhantomData,
        }
    }
}

/// Extension trait for synchronizing input contexts with [`bevy_state`](bevy::state).
pub trait StateContextAppExt {
    /// Registers automatic synchronization between state `S` and context `C`.
    ///
    /// When [`State<S>`] transitions, entities with [`ActiveInState<S, C>`] or
    /// [`ActiveInStates<S, C>`] will have their [`ContextActivity<C>`] updated.
    ///
    /// This assumes [`init_state::<S>()`](bevy::prelude::App::init_state) has
    /// already been called; otherwise contexts won't sync until the first
    /// transition after the state is initialized.
    ///
    /// The sync runs in the [`StateTransition`] schedule, ensuring contexts
    /// are activated before any `OnEnter` systems run.
    fn sync_context_to_state<S: States, C: Component>(&mut self) -> &mut Self;
}

impl StateContextAppExt for App {
    fn sync_context_to_state<S: States, C: Component>(&mut self) -> &mut Self {
        debug!(
            "registering state sync for `{}` with `{}`",
            ShortName::of::<S>(),
            ShortName::of::<C>(),
        );

        self.add_observer(sync_on_insert_single::<S, C>)
            .add_observer(sync_on_insert_multi::<S, C>)
            .add_systems(
                StateTransition,
                (
                    sync_single_state_contexts::<S, C>,
                    sync_multi_state_contexts::<S, C>,
                )
                    .chain(),
            )
    }
}

fn set_context_activity<C: Component>(
    commands: &mut Commands,
    entity: Entity,
    active: bool,
    current: Option<&ContextActivity<C>>,
) {
    if let Some(current) = current
        && **current == active
    {
        return;
    }
    debug!(
        "setting `{}` on `{}` to `{}`",
        ShortName::of::<C>(),
        entity,
        active,
    );
    commands
        .entity(entity)
        .insert(ContextActivity::<C>::new(active));
}

fn sync_on_insert_single<S: States, C: Component>(
    insert: On<Insert, ActiveInState<S, C>>,
    current_state: Option<Res<State<S>>>,
    contexts: Query<&ActiveInState<S, C>>,
    activity: Query<&ContextActivity<C>>,
    mut commands: Commands,
) {
    let Some(current_state) = current_state else {
        return;
    };
    let Ok(active_in) = contexts.get(insert.entity) else {
        return;
    };
    set_context_activity::<C>(
        &mut commands,
        insert.entity,
        active_in.matches(current_state.get()),
        activity.get(insert.entity).ok(),
    );
}

fn sync_on_insert_multi<S: States, C: Component>(
    insert: On<Insert, ActiveInStates<S, C>>,
    current_state: Option<Res<State<S>>>,
    contexts: Query<&ActiveInStates<S, C>>,
    activity: Query<&ContextActivity<C>>,
    mut commands: Commands,
) {
    let Some(current_state) = current_state else {
        return;
    };
    let Ok(active_in) = contexts.get(insert.entity) else {
        return;
    };
    set_context_activity::<C>(
        &mut commands,
        insert.entity,
        active_in.matches(current_state.get()),
        activity.get(insert.entity).ok(),
    );
}

fn sync_single_state_contexts<S: States, C: Component>(
    mut transitions: MessageReader<StateTransitionEvent<S>>,
    contexts: Query<(Entity, &ActiveInState<S, C>)>,
    activity: Query<&ContextActivity<C>>,
    mut commands: Commands,
) {
    let Some(transition) = transitions.read().last() else {
        return;
    };

    match &transition.entered {
        Some(entered) => {
            for (entity, active_in) in &contexts {
                set_context_activity::<C>(
                    &mut commands,
                    entity,
                    active_in.matches(entered),
                    activity.get(entity).ok(),
                );
            }
        }
        None => {
            for (entity, _) in &contexts {
                set_context_activity::<C>(&mut commands, entity, false, activity.get(entity).ok());
            }
        }
    }
}

fn sync_multi_state_contexts<S: States, C: Component>(
    mut transitions: MessageReader<StateTransitionEvent<S>>,
    contexts: Query<(Entity, &ActiveInStates<S, C>)>,
    activity: Query<&ContextActivity<C>>,
    mut commands: Commands,
) {
    let Some(transition) = transitions.read().last() else {
        return;
    };

    match &transition.entered {
        Some(entered) => {
            for (entity, active_in) in &contexts {
                set_context_activity::<C>(
                    &mut commands,
                    entity,
                    active_in.matches(entered),
                    activity.get(entity).ok(),
                );
            }
        }
        None => {
            for (entity, _) in &contexts {
                set_context_activity::<C>(&mut commands, entity, false, activity.get(entity).ok());
            }
        }
    }
}
