use bevy::{
    ecs::template::{EntityTemplate, Template, TemplateContext},
    log::warn_once,
    prelude::*,
};
use smallvec::{SmallVec, smallvec};

use crate::prelude::*;

/**
Set of actions that need to be triggered at the same time.

- [`TriggerState::Fired`] if all chorded actions fire.
- [`TriggerState::Ongoing`] if any are active and [`Self::ongoing`] is `true`.
- [`TriggerState::None`] otherwise.

Useful for defining a composite action that fires only when all listed actions fire.

# Examples

Use named entity references in BSN to refer to the chorded actions:

```
use bevy::{asset::AssetPlugin, prelude::*, scene::ScenePlugin};
use bevy_enhanced_input::prelude::*;

# let mut app = App::new();
# app.add_plugins((MinimalPlugins, AssetPlugin::default(), ScenePlugin));
# let world = app.world_mut();
world.spawn_scene(bsn! {
    Player
    Actions<Player> [
        #modifier
        Action::<Modifier>::new()
        Bindings [
            Binding(GamepadButton::LeftTrigger)
        ]
        --
        // Use `Heal` if `Modifier` is active.
        Action::<Heal>::new()
        Chord::single(#modifier)
        Bindings [
            Binding(GamepadButton::West)
        ]
        --
        // Otherwise use `Attack`.
        Action::<Attack>::new()
        Bindings [
            Binding(GamepadButton::West)
        ]
    ]
}).unwrap();

#[derive(Component, Default, Clone)]
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

When spawning bundles, use [`SpawnRelated::spawn`] or separate spawning with [`ActionOf`]/[`BindingOf`]
because you need to pass [`Entity`] for step and cancel actions:

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
#[derive(Component, Debug, Clone)]
#[cfg_attr(feature = "reflect", derive(Reflect), reflect(Clone, Component, Debug))]
pub struct Chord {
    /// Actions whose state will be inherited when they are firing.
    pub actions: SmallVec<[Entity; 2]>,

    /// Enables returning [`TriggerState::Ongoing`] when any action is active
    /// but not all have fired.
    ///
    /// When disabled, partial activation results in [`TriggerState::None`].
    ///
    /// Defaults to `true`.
    pub ongoing: bool,
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
            ongoing: true,
        }
    }

    /// Sets [`Self::ongoing`].
    #[must_use]
    pub fn with_ongoing(mut self, enable: bool) -> Self {
        self.ongoing = enable;
        self
    }
}

// We can't derive the template because `Chord` defaults to
// `ongoing: true`, and we can't override this default in the `FromTemplate` derive.
impl FromTemplate for Chord {
    type Template = ChordTemplate;
}

/// BSN template for [`Chord`] that accepts entity references.
#[derive(Debug)]
pub struct ChordTemplate {
    /// Actions whose state will be inherited when they are firing.
    pub actions: SmallVecTemplate<EntityTemplate, 2>,

    /// Whether partial activation returns [`TriggerState::Ongoing`].
    pub ongoing: bool,
}

impl Default for ChordTemplate {
    fn default() -> Self {
        Self {
            actions: SmallVecTemplate::default(),
            ongoing: true,
        }
    }
}

/// Constructors and methods that mirror [`Chord`] to allow passing entity references in BSN.
impl ChordTemplate {
    /// Creates a template for a single action.
    ///
    /// See also [`Chord::single`].
    #[must_use]
    pub fn single(action: impl Into<EntityTemplate>) -> Self {
        let action = action.into();
        Self::new([action])
    }

    /// Creates a template for multiple actions.
    ///
    /// See also [`Chord::new`].
    #[must_use]
    pub fn new(actions: impl Into<SmallVecTemplate<EntityTemplate, 2>>) -> Self {
        Self {
            actions: actions.into(),
            ongoing: true,
        }
    }

    /// Sets [`Self::ongoing`].
    ///
    /// See also [`Chord::with_ongoing`].
    #[must_use]
    pub fn with_ongoing(mut self, enable: bool) -> Self {
        self.ongoing = enable;
        self
    }
}

impl Template for ChordTemplate {
    type Output = Chord;

    fn build_template(&self, context: &mut TemplateContext) -> Result<Chord> {
        Ok(Chord::new(self.actions.build_template(context)?).with_ongoing(self.ongoing))
    }

    fn clone_template(&self) -> Self {
        Self {
            actions: self.actions.clone_template(),
            ongoing: self.ongoing,
        }
    }
}

impl InputCondition for Chord {
    fn evaluate(
        &mut self,
        actions: &ActionsQuery,
        _time: &ContextTime,
        _value: ActionValue,
    ) -> TriggerState {
        let mut has_active = false;
        let mut all_fired = true;
        for &action in &self.actions {
            let Ok((_, &state, ..)) = actions.get(action) else {
                warn_once!("`{action}` is not a valid action");
                continue;
            };

            if state != TriggerState::None {
                has_active = true;
            }

            if state != TriggerState::Fired {
                all_fired = false;
            }
        }

        if has_active && all_fired {
            TriggerState::Fired
        } else if has_active && self.ongoing {
            TriggerState::Ongoing
        } else {
            TriggerState::None
        }
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
            .spawn((Action::<Test>::new(), TriggerState::Fired))
            .id();
        let (time, actions) = state.get(&world).unwrap();

        let mut condition = Chord::single(action);
        assert_eq!(
            condition.evaluate(&actions, &time, true.into()),
            TriggerState::Fired,
        );
    }

    #[test]
    fn with_ongoing() {
        let (mut world, mut state) = context::init_world();
        let action1 = world
            .spawn((Action::<Test>::new(), TriggerState::Fired))
            .id();
        let action2 = world
            .spawn((Action::<Test>::new(), TriggerState::None))
            .id();
        let (time, actions) = state.get(&world).unwrap();

        let mut condition = Chord::new([action1, action2]).with_ongoing(true);
        assert_eq!(
            condition.evaluate(&actions, &time, true.into()),
            TriggerState::Ongoing,
        );
    }

    #[test]
    fn without_ongoing() {
        let (mut world, mut state) = context::init_world();
        let action1 = world
            .spawn((Action::<Test>::new(), TriggerState::Fired))
            .id();
        let action2 = world
            .spawn((Action::<Test>::new(), TriggerState::None))
            .id();
        let (time, actions) = state.get(&world).unwrap();

        let mut condition = Chord::new([action1, action2]).with_ongoing(false);
        assert_eq!(
            condition.evaluate(&actions, &time, true.into()),
            TriggerState::None,
        );
    }

    #[test]
    fn none() {
        let (mut world, mut state) = context::init_world();
        let action1 = world
            .spawn((Action::<Test>::new(), TriggerState::None))
            .id();
        let action2 = world
            .spawn((Action::<Test>::new(), TriggerState::None))
            .id();
        let (time, actions) = state.get(&world).unwrap();

        let mut condition = Chord::new([action1, action2]);
        assert_eq!(
            condition.evaluate(&actions, &time, true.into()),
            TriggerState::None,
        );
    }

    #[test]
    fn missing_action() {
        let (world, mut state) = context::init_world();
        let (time, actions) = state.get(&world).unwrap();

        let mut condition = Chord::single(Entity::PLACEHOLDER);
        assert_eq!(
            condition.evaluate(&actions, &time, true.into()),
            TriggerState::None,
        );
    }

    #[derive(InputAction)]
    #[action_output(bool)]
    struct Test;
}
