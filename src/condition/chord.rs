use bevy::prelude::*;
use log::warn;
use smallvec::{SmallVec, smallvec};

use crate::prelude::*;

/**
Returns [`ActionState::Fired`] if all given actions fire, otherwise returns their maximum
[`ActionState`], capped at [`ActionState::Ongoing`].

Useful for defining a composite action that fires only when all listed actions are active.

Requires using [`SpawnRelated::spawn`] or separate spawning with [`ActionOf`]/[`BindingOf`]
because you need to pass [`Entity`] for step and cancel actions.

# Examples

```
use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;

# let mut world = World::new();
world.spawn((
    Player,
    Actions::<Player>::spawn(SpawnWith(|context: &mut ActionSpawner<_>| {
        let modifier = context
            .spawn((Action::<Modifier>::new(), bindings![GamepadButton::LeftTrigger]))
            .id();

        // Use `Heal` if `Modifier` is active.
        context.spawn((
            Action::<Heal>::new(),
            Chord::single(modifier),
            bindings![GamepadButton::West],
        ));

        // Otherwise use `Attack`.
        context.spawn((Action::<Attack>::new(), bindings![GamepadButton::West]));
    })),
));

#[derive(Component)]
struct Player;

#[derive(InputAction)]
#[action_output(bool)]
struct Attack;

#[derive(InputAction)]
#[action_output(bool)]
struct Modifier;

#[derive(InputAction)]
#[action_output(bool)]
struct Heal;
```
*/
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
