use bevy::prelude::*;
use log::warn;
use smallvec::{SmallVec, smallvec};

use crate::prelude::*;

/// Returns [`TriggerState::None`] when specific actions are active.
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
///     let fly = context
///         .spawn((
///             Action::<Fly>::new(),
///             bindings![KeyCode::Space],
///         ))
///         .id();
///
///     // Requires `Fly` to be inactive in order to trigger.
///     context.spawn((
///         Action::<Melee>::new(),
///         BlockBy::single(fly),
///         bindings![KeyCode::KeyF],
///     ));
/// }));
///
/// #[derive(Component)]
/// struct TestContext;
///
/// #[derive(InputAction)]
/// #[action_output(bool)]
/// struct Fly;
///
/// #[derive(InputAction)]
/// #[action_output(bool)]
/// struct Melee;
/// ```
#[derive(Component, Reflect, Debug, Clone)]
pub struct BlockBy {
    /// Actions that block this action when they are firing.
    pub actions: SmallVec<[Entity; 2]>,
}

impl BlockBy {
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

impl InputCondition for BlockBy {
    fn evaluate(
        &mut self,
        actions: &ActionsQuery,
        _time: &ContextTime,
        _value: ActionValue,
    ) -> TriggerState {
        for &action in &self.actions {
            let Ok((_, &state, ..)) = actions.get(action) else {
                // TODO: use `warn_once` when `bevy_log` becomes `no_std` compatible.
                warn!("`{action}` is not a valid action");
                continue;
            };

            if state == TriggerState::Fired {
                return TriggerState::None;
            }
        }

        TriggerState::Fired
    }

    fn kind(&self) -> ConditionKind {
        ConditionKind::Blocker
    }
}

#[cfg(test)]
mod tests {
    use bevy_enhanced_input_macros::InputAction;

    use super::*;
    use crate::context;

    #[test]
    fn block() {
        let (mut world, mut state) = context::init_world();
        let action = world
            .spawn((Action::<Test>::new(), TriggerState::Fired))
            .id();
        let (time, actions) = state.get(&world);

        let mut condition = BlockBy::single(action);
        assert_eq!(
            condition.evaluate(&actions, &time, true.into()),
            TriggerState::None,
        );
    }

    #[test]
    fn missing_action() {
        let (world, mut state) = context::init_world();
        let (time, actions) = state.get(&world);

        let mut condition = BlockBy::single(Entity::PLACEHOLDER);
        assert_eq!(
            condition.evaluate(&actions, &time, true.into()),
            TriggerState::Fired,
        );
    }

    #[derive(InputAction)]
    #[action_output(bool)]
    struct Test;
}
