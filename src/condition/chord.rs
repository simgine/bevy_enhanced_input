use bevy::prelude::*;
use log::warn;
use smallvec::{SmallVec, smallvec};

use crate::prelude::*;

/// Returns [`ActionState::Fired`] if all given actions fire, otherwise returns their maximum [`ActionState`], capped at [`ActionState::Ongoing`].
///
/// Useful for defining a composite action that fires only when all listed actions are active.
///
/// # Examples
///
/// To get action entities during spawning, you could use [`SpawnWith`].
///
/// ```
/// use bevy::{ecs::spawn::SpawnWith, prelude::*};
/// use bevy_enhanced_input::prelude::*;
///
/// Actions::<TestContext>::spawn(SpawnWith(|context: &mut ActionSpawner<_>| {
///     let combo1 = context
///         .spawn((Action::<Combo1>::new(), bindings![KeyCode::KeyQ]))
///         .id();
///     let combo2 = context
///         .spawn((Action::<Combo2>::new(), bindings![KeyCode::KeyX]))
///         .id();
///
///     // Will trigger when both `Combo1` and `Combo2` fire.
///     context.spawn((Action::<SuperCombo>::new(), Chord::new([combo1, combo2])));
/// }));
///
/// #[derive(Component)]
/// struct TestContext;
///
/// #[derive(InputAction)]
/// #[action_output(bool)]
/// struct Combo1;
///
/// #[derive(InputAction)]
/// #[action_output(bool)]
/// struct Combo2;
///
/// #[derive(InputAction)]
/// #[action_output(bool)]
/// struct SuperCombo;
/// ```
#[derive(Component, Reflect, Debug, Clone)]
pub struct Chord {
    /// Actions whose state will be inherited when they are firing.
    pub actions: SmallVec<[Entity; 2]>,
}

impl Chord {
    /// Creates a new instance for a single action.
    #[must_use]
    pub fn single(action: Entity) -> Self {
        Self::new(smallvec![action])
    }

    /// Creates a new instance for multiple actions.
    #[must_use]
    pub fn new(actions: impl Into<SmallVec<[Entity; 2]>>) -> Self {
        Self {
            actions: actions.into(),
        }
    }
}

impl InputCondition for Chord {
    fn evaluate(
        &mut self,
        actions: &ActionsQuery,
        _time: &ContextTime,
        _value: ActionValue,
    ) -> ActionState {
        // Inherit state from the most significant chorded action.
        let mut max_state = Default::default();
        let mut all_fired = true;
        for &action in &self.actions {
            let Ok((_, &state, ..)) = actions.get(action) else {
                // TODO: use `warn_once` when `bevy_log` becomes `no_std` compatible.
                warn!("`{action}` is not a valid action");
                continue;
            };

            if state != ActionState::Fired {
                all_fired = false;
            }

            if state > max_state {
                max_state = state;
            }
        }

        if !all_fired {
            max_state = max_state.min(ActionState::Ongoing);
        }

        max_state
    }

    fn kind(&self) -> ConditionKind {
        ConditionKind::Implicit
    }
}

#[cfg(test)]
mod tests {
    use bevy_enhanced_input_macros::InputAction;

    use super::*;
    use crate::context;

    #[test]
    fn fired() {
        let (mut world, mut state) = context::init_world();
        let action = world
            .spawn((Action::<Test>::new(), ActionState::Fired))
            .id();
        let (time, actions) = state.get(&world);

        let mut condition = Chord::single(action);
        assert_eq!(
            condition.evaluate(&actions, &time, true.into()),
            ActionState::Fired,
        );
    }

    #[test]
    fn ongoing() {
        let (mut world, mut state) = context::init_world();
        let action1 = world
            .spawn((Action::<Test>::new(), ActionState::Fired))
            .id();
        let action2 = world.spawn((Action::<Test>::new(), ActionState::None)).id();
        let (time, actions) = state.get(&world);

        let mut condition = Chord::new([action1, action2]);
        assert_eq!(
            condition.evaluate(&actions, &time, true.into()),
            ActionState::Ongoing,
        );
    }

    #[test]
    fn none() {
        let (mut world, mut state) = context::init_world();
        let action1 = world.spawn((Action::<Test>::new(), ActionState::None)).id();
        let action2 = world.spawn((Action::<Test>::new(), ActionState::None)).id();
        let (time, actions) = state.get(&world);

        let mut condition = Chord::new([action1, action2]);
        assert_eq!(
            condition.evaluate(&actions, &time, true.into()),
            ActionState::None,
        );
    }

    #[test]
    fn missing_action() {
        let (world, mut state) = context::init_world();
        let (time, actions) = state.get(&world);

        let mut condition = Chord::single(Entity::PLACEHOLDER);
        assert_eq!(
            condition.evaluate(&actions, &time, true.into()),
            ActionState::None,
        );
    }

    #[derive(InputAction)]
    #[action_output(bool)]
    struct Test;
}
