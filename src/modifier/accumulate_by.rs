use bevy::{ecs::template::EntityTemplate, log::warn_once, prelude::*};

use crate::prelude::*;

/// Produces accumulated value when another action is fired within the same context.
///
/// Continuously adds input values together as long as action `A` is [`TriggerState::Fired`].
/// When the action is inactive, it resets the accumulation with the current frame's input value.
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
///         #accelerate
///         Action::<Accelerate>::new()
///         Bindings [
///             Binding(KeyCode::ShiftLeft)
///             --
///             Binding(KeyCode::ShiftRight)
///         ]
///         --
///         // Sums movement when `Accelerate` is pressed.
///         Action::<Movement>::new()
///         AccumulateBy::new(#accelerate)
///         Bindings [
///             {preset::cardinal::new(
///                 (KeyCode::KeyW, DeadZone::default()),
///                 (KeyCode::KeyA, DeadZone::default()),
///                 (KeyCode::KeyS, DeadZone::default()),
///                 (KeyCode::KeyD, DeadZone::default()),
///             )}
///         ]
///     ]
/// };
///
/// #[derive(Component, Default, Clone)]
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
///
/// When spawning bundles, you could use [`SpawnWith`] to get action entities during spawning:
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
#[derive(Component, Debug, FromTemplate, Clone, Copy)]
#[cfg_attr(feature = "reflect", derive(Reflect), reflect(Clone, Component, Debug))]
pub struct AccumulateBy {
    /// Action that activates accumulation.
    action: Entity,

    /// The accumulated value across frames.
    value: Vec3,
}

impl AccumulateBy {
    /// Creates a modifier with zero accumulated value.
    #[must_use]
    pub const fn new(action: Entity) -> Self {
        Self {
            action,
            value: Vec3::ZERO,
        }
    }
}

/// Constructors that mirror [`AccumulateBy`] to allow passing entity references in BSN.
impl AccumulateByTemplate {
    /// Creates a template with zero accumulated value.
    ///
    /// See also [`AccumulateBy::new`].
    #[must_use]
    pub fn new(action: impl Into<EntityTemplate>) -> Self {
        Self {
            action: action.into(),
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
            warn_once!("`{}` is not a valid action", self.action);
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
        let (time, actions) = state.get(&world).unwrap();

        let mut modifier = AccumulateBy::new(action);
        assert_eq!(modifier.transform(&actions, &time, 1.0.into()), 1.0.into());
        assert_eq!(modifier.transform(&actions, &time, 1.0.into()), 2.0.into());
    }

    #[test]
    fn accumulation_inactive() {
        let (mut world, mut state) = context::init_world();
        let action = world.spawn(Action::<Test>::new()).id();
        let (time, actions) = state.get(&world).unwrap();

        let mut modifier = AccumulateBy::new(action);
        assert_eq!(modifier.transform(&actions, &time, 1.0.into()), 1.0.into());
        assert_eq!(modifier.transform(&actions, &time, 1.0.into()), 1.0.into());
    }

    #[test]
    fn missing_action() {
        let (world, mut state) = context::init_world();
        let (time, actions) = state.get(&world).unwrap();

        let mut modifier = AccumulateBy::new(Entity::PLACEHOLDER);
        assert_eq!(modifier.transform(&actions, &time, 1.0.into()), 1.0.into());
        assert_eq!(modifier.transform(&actions, &time, 1.0.into()), 1.0.into());
    }

    #[derive(InputAction)]
    #[action_output(bool)]
    struct Test;
}
