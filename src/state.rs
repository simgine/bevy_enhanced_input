/*!
State-context integration for synchronizing Bevy [`States`] with input contexts.

This module provides components that automatically activate/deactivate
input contexts based on the current application state, eliminating manual
[`OnEnter`]/[`DespawnOnExit`] boilerplate.

# Example

```
use bevy::{prelude::*, state::app::StatesPlugin};
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

let mut app = App::new();
app.add_plugins((StatesPlugin, EnhancedInputPlugin))
    .init_state::<GameState>()
    .add_input_context::<Player>()
    .sync_context_to_state::<Player, GameState>()
    .finish();

// When spawning entities, use `ActiveInStates` to declare which states activate the context.
app.world_mut().spawn((
    Player,
    ContextActivity::<Player>::INACTIVE,
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

use crate::prelude::*;

/// Extension trait for synchronizing input contexts with [`States`].
pub trait StateContextAppExt {
    /// Registers automatic synchronization between context `C` and state `S`.
    ///
    /// When [`State<S>`] transitions, entities with [`ActiveInStates<C, S>`]
    /// will have their [`ContextActivity<C>`] updated.
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

fn sync_on_insert<C: Component, S: States>(
    insert: On<Insert, ActiveInStates<C, S>>,
    mut commands: Commands,
    current_state: Res<State<S>>,
    contexts: Query<&ActiveInStates<C, S>>,
    activity: Query<&ContextActivity<C>>,
) {
    let Ok(active_in) = contexts.get(insert.entity) else {
        return;
    };
    set_context_activity(
        &mut commands,
        &activity,
        insert.entity,
        active_in.matches(current_state.get()),
    );
}

fn sync_state_contexts<C: Component, S: States>(
    mut commands: Commands,
    mut transitions: MessageReader<StateTransitionEvent<S>>,
    contexts: Query<(Entity, &ActiveInStates<C, S>)>,
    activity: Query<&ContextActivity<C>>,
) {
    let Some(transition) = transitions.read().last() else {
        return;
    };

    match &transition.entered {
        Some(entered) => {
            for (entity, active_in) in &contexts {
                set_context_activity(&mut commands, &activity, entity, active_in.matches(entered));
            }
        }
        None => {
            for (entity, _) in &contexts {
                set_context_activity(&mut commands, &activity, entity, false);
            }
        }
    }
}

fn set_context_activity<C: Component>(
    commands: &mut Commands,
    activity: &Query<&ContextActivity<C>>,
    entity: Entity,
    active: bool,
) {
    if let Ok(current) = activity.get(entity)
        && **current == active
    {
        return;
    }
    debug!(
        "setting `{}` on `{entity}` to `{active}`",
        ShortName::of::<C>(),
    );
    commands
        .entity(entity)
        .insert(ContextActivity::<C>::new(active));
}

/// Inserts [`ContextActivity::<C>::ACTIVE`] when the state matches one of the
/// specified values, and [`ContextActivity::<C>::INACTIVE`] otherwise.
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

    /// Returns `true` if the current state matches any of the active states.
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
