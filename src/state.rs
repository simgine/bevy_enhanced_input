/*!
State-context integration for synchronizing [`bevy_state`](bevy::state) with input contexts.

This module provides components that automatically activate/deactivate
input contexts based on the current application state, eliminating manual
[`OnEnter`]/[`OnExit`] boilerplate.

# Example

```
use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;

#[derive(States, Clone, PartialEq, Eq, Hash, Debug, Default)]
enum GameState {
    #[default]
    Playing,
    Paused,
}

#[derive(Component)]
struct Player;

#[derive(InputAction)]
#[action_output(bool)]
struct Jump;

let mut world = World::new();
world.spawn((
    Player,
    ActiveInStates::<Player, _>::single(GameState::Playing),
    actions!(Player[(Action::<Jump>::new(), bindings![KeyCode::Space])]),
));
```
*/

use core::marker::PhantomData;

use bevy::{
    prelude::*,
    state::state::{StateTransitionEvent, StateTransitionSystems, States},
};
use log::debug;
use smallvec::SmallVec;

use crate::prelude::ContextActivity;

/// Activates input context `C` when state `S` matches any of the specified values.
///
/// When the state matches, [`ContextActivity::<C>::ACTIVE`] is inserted.
/// When the state doesn't match, [`ContextActivity::<C>::INACTIVE`] is inserted.
#[derive(Component, Reflect)]
pub struct ActiveInStates<C: Component, S: States> {
    states: SmallVec<[S; 1]>,
    #[reflect(ignore)]
    _marker: PhantomData<C>,
}

impl<C: Component, S: States> ActiveInStates<C, S> {
    /// Creates a new instance for a single state.
    #[must_use]
    pub fn single(state: S) -> Self {
        Self {
            states: SmallVec::from_buf([state]),
            _marker: PhantomData,
        }
    }

    /// Creates a new instance for multiple states.
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

impl<C: Component, S: States> Clone for ActiveInStates<C, S> {
    fn clone(&self) -> Self {
        Self {
            states: self.states.clone(),
            _marker: PhantomData,
        }
    }
}

/// Extension trait for synchronizing input contexts with [`bevy_state`](bevy::state).
pub trait StateContextAppExt {
    /// Registers automatic synchronization between context `C` and state `S`.
    ///
    /// When [`State<S>`] transitions, entities with [`ActiveInStates<C, S>`]
    /// will have their [`ContextActivity<C>`] updated.
    ///
    /// This assumes [`init_state::<S>()`](bevy::prelude::App::init_state) has
    /// already been called; otherwise contexts won't sync until the first
    /// transition after the state is initialized.
    ///
    /// The sync runs in the [`StateTransition`] schedule, ensuring contexts
    /// are activated before any [`OnEnter`] systems run.
    fn sync_context_to_state<C: Component, S: States>(&mut self) -> &mut Self;
}

impl StateContextAppExt for App {
    fn sync_context_to_state<C: Component, S: States>(&mut self) -> &mut Self {
        debug!(
            "registering state sync for `{}` with `{}`",
            ShortName::of::<C>(),
            ShortName::of::<S>(),
        );

        self.add_observer(sync_on_insert::<C, S>).add_systems(
            StateTransition,
            sync_state_contexts::<C, S>
                .after(StateTransitionSystems::DependentTransitions)
                .before(StateTransitionSystems::ExitSchedules),
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

fn sync_on_insert<C: Component, S: States>(
    insert: On<Insert, ActiveInStates<C, S>>,
    current_state: Option<Res<State<S>>>,
    contexts: Query<&ActiveInStates<C, S>>,
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

fn sync_state_contexts<C: Component, S: States>(
    mut transitions: MessageReader<StateTransitionEvent<S>>,
    contexts: Query<(Entity, &ActiveInStates<C, S>)>,
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
        // State was removed/cleared rather than transitioned — deactivate all contexts.
        None => {
            for (entity, _) in &contexts {
                set_context_activity::<C>(&mut commands, entity, false, activity.get(entity).ok());
            }
        }
    }
}
