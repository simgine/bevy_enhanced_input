use bevy::prelude::*;
use log::warn;

use crate::prelude::*;

/// Produces accumulated value when another action is fired within the same context.
///
/// Continuously adds input values together as long as action `A` is [`TriggerState::Fired`].
/// When the action is inactive, it resets the accumulation with the current frame's input value.
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
///     let accelerate = context
///         .spawn((
///             Action::<Accelerate>::new(),
///             bindings![KeyCode::ShiftLeft, KeyCode::ShiftRight],
///         ))
///         .id();
///
///     // Sums movement when `Accelerate` is pressed.
///     context.spawn((
///         Action::<Movement>::new(),
///         AccumulateBy::new(accelerate),
///         Bindings::spawn(Cardinal::wasd_keys().with(DeadZone::default())),
///     ));
/// }));
///
/// #[derive(Component)]
/// struct TestContext;
///
/// #[derive(InputAction)]
/// #[action_output(bool)]
/// struct Accelerate;
///
/// #[derive(InputAction)]
/// #[action_output(f32)]
/// struct Movement;
/// ```
#[derive(Component, Reflect, Debug, Clone, Copy)]
pub struct AccumulateBy {
    /// Action that activates accumulation.
    action: Entity,

    /// The accumulated value across frames.
    value: Vec3,
}

impl AccumulateBy {
    #[must_use]
    pub const fn new(action: Entity) -> Self {
        Self {
            action,
            value: Vec3::ZERO,
        }
    }
}

impl InputModifier for AccumulateBy {
    fn transform(
        &mut self,
        actions: &ActionsQuery,
        _time: &ContextTime,
        value: ActionValue,
    ) -> ActionValue {
        if let Ok((_, &state, ..)) = actions.get(self.action) {
            if state == TriggerState::Fired {
                self.value += value.as_axis3d();
            } else {
                self.value = value.as_axis3d();
            }
            ActionValue::Axis3D(self.value).convert(value.dim())
        } else {
            // TODO: use `warn_once` when `bevy_log` becomes `no_std` compatible.
            warn!("`{}` is not a valid action", self.action);
            value
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy_enhanced_input_macros::InputAction;

    use super::*;
    use crate::context;

    #[test]
    fn accumulation_active() {
        let (mut world, mut state) = context::init_world();
        let action = world
            .spawn((
                Action::<Test>::new(),
                TriggerState::Fired,
                ActionValue::from(true),
            ))
            .id();
        let (time, actions) = state.get(&world);

        let mut modifier = AccumulateBy::new(action);
        assert_eq!(modifier.transform(&actions, &time, 1.0.into()), 1.0.into());
        assert_eq!(modifier.transform(&actions, &time, 1.0.into()), 2.0.into());
    }

    #[test]
    fn accumulation_inactive() {
        let (mut world, mut state) = context::init_world();
        let action = world.spawn(Action::<Test>::new()).id();
        let (time, actions) = state.get(&world);

        let mut modifier = AccumulateBy::new(action);
        assert_eq!(modifier.transform(&actions, &time, 1.0.into()), 1.0.into());
        assert_eq!(modifier.transform(&actions, &time, 1.0.into()), 1.0.into());
    }

    #[test]
    fn missing_action() {
        let (world, mut state) = context::init_world();
        let (time, actions) = state.get(&world);

        let mut modifier = AccumulateBy::new(Entity::PLACEHOLDER);
        assert_eq!(modifier.transform(&actions, &time, 1.0.into()), 1.0.into());
        assert_eq!(modifier.transform(&actions, &time, 1.0.into()), 1.0.into());
    }

    #[derive(InputAction)]
    #[action_output(bool)]
    struct Test;
}
