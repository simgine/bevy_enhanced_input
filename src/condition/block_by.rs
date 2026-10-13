use bevy::{ecs::template::EntityTemplate, log::warn_once, prelude::*};
use smallvec::{SmallVec, smallvec};

use crate::prelude::*;

/// Returns [`TriggerState::None`] when specific actions are active.
///
/// # Examples
///
/// Use named entity references in BSN to refer to the blocking actions:
///
/// ```
/// use bevy::prelude::*;
/// use bevy_enhanced_input::prelude::*;
///
/// let scene = bsn! {
///     TestContext
///     Actions<TestContext> [
///         #fly
///         Action::<Fly>::new()
///         Bindings [
///             Binding(KeyCode::Space)
///         ]
///         --
///         // Requires `Fly` to be inactive in order to trigger.
///         Action::<Melee>::new()
///         BlockBy::single(#fly)
///         Bindings [
///             Binding(KeyCode::KeyF)
///         ]
///     ]
/// };
///
/// #[derive(Component, Default, Clone)]
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
#[derive(Component, Debug, Clone, FromTemplate)]
#[cfg_attr(feature = "reflect", derive(Reflect), reflect(Clone, Component, Debug))]
pub struct BlockBy {
    /// Actions that block this action when they are firing.
    #[template(SmallVecTemplate<EntityTemplate, 2>)]
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

/// Constructors that mirror [`BlockBy`] to allow passing entity references in BSN.
impl BlockByTemplate {
    /// Creates a template for a single action.
    ///
    /// See also [`BlockBy::single`].
    #[must_use]
    pub fn single(action: impl Into<EntityTemplate>) -> Self {
        let action = action.into();
        Self::new([action])
    }

    /// Creates a template for multiple actions.
    ///
    /// See also [`BlockBy::new`].
    #[must_use]
    pub fn new(actions: impl Into<SmallVecTemplate<EntityTemplate, 2>>) -> Self {
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
                warn_once!("`{action}` is not a valid action");
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
        let (time, actions) = state.get(&world).unwrap();

        let mut condition = BlockBy::single(action);
        assert_eq!(
            condition.evaluate(&actions, &time, true.into()),
            TriggerState::None,
        );
    }

    #[test]
    fn missing_action() {
        let (world, mut state) = context::init_world();
        let (time, actions) = state.get(&world).unwrap();

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
