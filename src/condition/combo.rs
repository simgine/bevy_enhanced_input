use core::time::Duration;

use bevy::prelude::*;
use log::warn;

use crate::prelude::*;

/**
Sequence of actions that needs to be triggered in specific order.

The combo resets if a step is triggered out of order or by any defined
cancel action.

After the first step, returns [`ActionState::Ongoing`] until the last step.
Once all steps are completed, returns [`ActionState::Fired`] once, then resets.

Requires using [`SpawnRelated::spawn`] or separate spawning with [`ActionOf`]/[`BindingOf`]
because you need to pass [`Entity`] for step and cancel actions.

# Examples

Double click:

```
use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;

# let mut world = World::new();
world.spawn((
    Menu,
    Actions::<Menu>::spawn(SpawnWith(|context: &mut ActionSpawner<_>| {
        let click = context
            .spawn((Action::<Click>::new(), bindings![MouseButton::Left]))
            .id();

        context.spawn((
            Action::<DoubleClick>::new(),
            Combo::default().with_step(click).with_step(click),
        ));
    })),
));

#[derive(InputAction)]
#[action_output(bool)]
struct Click;

#[derive(InputAction)]
#[action_output(bool)]
struct DoubleClick;

#[derive(Component)]
struct Menu;
```
*/
#[derive(Component, Reflect, Default, Debug, Clone)]
pub struct Combo {
    /// Ordered sequence of steps that define the combo.
    ///
    /// Each step is satisfied when its [`ComboStep::events`] occur,
    /// and steps must be completed in order.
    pub steps: Vec<ComboStep>,

    /// Actions that can cancel the combo.
    ///
    /// If a cancel action matches the action from the current step, it will be ignored.
    pub cancel_actions: Vec<CancelAction>,

    /// The type of time used to advance the timer.
    pub time_kind: TimeKind,

    /// Index of the current step in the combo.
    step_index: usize,

    /// Tracks timeout for completing the current step.
    timer: Timer,
}

impl Combo {
    /// Adds an action step to the combo.
    ///
    /// If you don't need to configure the step, you can just pass the action's [`Entity`].
    pub fn with_step(mut self, step: impl Into<ComboStep>) -> Self {
        self.steps.push(step.into());
        self
    }

    /// Adds an action that cancels the combo.
    ///
    /// If you don't need to configure the events, you can just pass the action's [`Entity`].
    pub fn with_cancel(mut self, action: impl Into<CancelAction>) -> Self {
        self.cancel_actions.push(action.into());
        self
    }

    fn reset(&mut self) {
        self.step_index = 0;
        self.timer.reset();

        let duration = self.steps.first().map(|s| s.timeout).unwrap_or_default();
        self.timer.set_duration(Duration::from_secs_f32(duration));
    }

    fn is_cancelled(&self, actions: &ActionsQuery) -> bool {
        let current_step = &self.steps[self.step_index];
        for condition in &self.cancel_actions {
            if condition.action == current_step.action {
                continue;
            }

            let Ok((.., events, _)) = actions.get(condition.action) else {
                // TODO: use `warn_once` when `bevy_log` becomes `no_std` compatible.
                warn!(
                    "cancel condition references an invalid action `{}`",
                    condition.action
                );
                continue;
            };

            if events.intersects(condition.events) {
                return true;
            }
        }

        // Check if any other step is also triggered, breaking the order.
        for step in &self.steps {
            if step.action == current_step.action {
                continue;
            }
            let Ok((.., events, _)) = actions.get(step.action) else {
                continue;
            };

            if events.intersects(step.events) {
                return true;
            }
        }

        false
    }
}

impl InputCondition for Combo {
    fn evaluate(
        &mut self,
        actions: &ActionsQuery,
        time: &ContextTime,
        _value: ActionValue,
    ) -> ActionState {
        if self.steps.is_empty() {
            // TODO: use `warn_once` when `bevy_log` becomes `no_std` compatible.
            warn!("combo has no steps");
            return ActionState::None;
        }

        if self.is_cancelled(actions) {
            // We don't early-return since the first step could be triggered.
            self.reset();
        }

        if self.step_index > 0 {
            self.timer.tick(time.delta_kind(self.time_kind));

            if self.timer.is_finished() {
                self.reset();
            }
        }

        let current_step = &self.steps[self.step_index];
        let Ok((_, &state, events, _)) = actions.get(current_step.action) else {
            // TODO: use `warn_once` when `bevy_log` becomes `no_std` compatible.
            warn!(
                "step {} references an invalid action `{}`",
                self.step_index, current_step.action
            );
            self.reset();
            return ActionState::None;
        };

        if events.contains(current_step.events) {
            self.step_index += 1;

            if self.step_index >= self.steps.len() {
                // Completed all combo actions.
                self.reset();
                return ActionState::Fired;
            } else {
                let next_step = &self.steps[self.step_index];
                self.timer.reset();
                self.timer
                    .set_duration(Duration::from_secs_f32(next_step.timeout));
            }
        }

        if self.step_index > 0 || state > ActionState::None {
            return ActionState::Ongoing;
        }

        ActionState::None
    }

    fn kind(&self) -> ConditionKind {
        ConditionKind::Implicit
    }
}

/// An action with associated events that progress [`Combo`].
#[derive(Reflect, Debug, Clone, Copy)]
pub struct ComboStep {
    /// Associated action.
    pub action: Entity,

    /// Events for the action to complete this step.
    pub events: ActionEvents,

    /// Time in seconds to trigger [`Self::events`] before the combo is cancelled.
    ///
    /// Starts once the previous step in the combo is completed.
    /// Ignored for the first action in the combo.
    pub timeout: f32,
}

impl ComboStep {
    /// Creates a new instance with [`Self::events`] set to [`ActionEvents::COMPLETED`]
    /// and [`Self::timeout`] set to 0.5.
    #[must_use]
    pub fn new(action: Entity) -> Self {
        Self {
            action,
            events: ActionEvents::COMPLETED,
            timeout: 0.5,
        }
    }

    /// Sets [`Self::with_events`].
    #[must_use]
    pub fn with_events(mut self, events: ActionEvents) -> Self {
        self.events = events;
        self
    }

    /// Sets [`Self::timeout`].
    #[must_use]
    pub fn with_timeout(mut self, timeout: f32) -> Self {
        self.timeout = timeout;
        self
    }
}

impl From<Entity> for ComboStep {
    fn from(action: Entity) -> Self {
        Self::new(action)
    }
}

/// An action with associated events that cancel a [`Combo`].
#[derive(Reflect, Debug, Clone, Copy)]
pub struct CancelAction {
    /// Associated action.
    pub action: Entity,

    /// Events that cancel the combo if any occur.
    pub events: ActionEvents,
}

impl CancelAction {
    /// Creates a new instance with events set to [`ActionEvents::ONGOING`] and [`ActionEvents::FIRED`].
    #[must_use]
    fn new(action: Entity) -> Self {
        Self {
            action,
            events: ActionEvents::ONGOING | ActionEvents::FIRED,
        }
    }
}

impl From<Entity> for CancelAction {
    fn from(action: Entity) -> Self {
        Self::new(action)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context;

    #[test]
    fn empty() {
        let (world, mut state) = context::init_world();
        let (time, actions) = state.get(&world);

        let mut condition = Combo::default();
        assert_eq!(
            condition.evaluate(&actions, &time, 0.0.into()),
            ActionState::None
        );
    }

    #[test]
    fn invalid_step() {
        let (world, mut state) = context::init_world();
        let (time, actions) = state.get(&world);

        let mut condition = Combo::default().with_step(Entity::PLACEHOLDER);
        assert_eq!(
            condition.evaluate(&actions, &time, 0.0.into()),
            ActionState::None
        );
        assert_eq!(condition.step_index, 0);
    }

    #[test]
    fn timeout() {
        let (mut world, mut state) = context::init_world();
        let action_a = world
            .spawn((Action::<A>::new(), ActionEvents::COMPLETED))
            .id();
        let action_b = world.spawn(Action::<B>::new()).id();
        world
            .resource_mut::<Time>()
            .advance_by(Duration::from_secs(1));
        let (time, actions) = state.get(&world);

        let mut condition = Combo::default().with_step(action_a).with_step(action_b);

        assert_eq!(
            condition.evaluate(&actions, &time, 0.0.into()),
            ActionState::Ongoing,
            "first step shouldn't be affected by time"
        );
        assert_eq!(condition.step_index, 1);

        world
            .resource_mut::<Time>()
            .advance_by(Duration::from_secs(1));
        world.entity_mut(action_a).insert(ActionEvents::empty()); // Clear `Completed` event.
        world.entity_mut(action_b).insert(ActionEvents::COMPLETED);
        let (time, actions) = state.get(&world);

        assert_eq!(
            condition.evaluate(&actions, &time, 0.0.into()),
            ActionState::None
        );
        assert_eq!(condition.step_index, 0);
    }

    #[test]
    fn first_step_ongoing() {
        let (mut world, mut state) = context::init_world();
        let action_a = world.spawn((Action::<A>::new(), ActionState::Ongoing)).id();
        let (time, actions) = state.get(&world);

        let mut condition = Combo::default().with_step(action_a);
        assert_eq!(
            condition.evaluate(&actions, &time, 0.0.into()),
            ActionState::Ongoing
        );
    }

    #[test]
    fn steps() {
        let (mut world, mut state) = context::init_world();
        let action_a = world
            .spawn((Action::<A>::new(), ActionEvents::ONGOING))
            .id();
        let action_b = world.spawn(Action::<B>::new()).id();
        let action_c = world.spawn(Action::<C>::new()).id();
        let (time, actions) = state.get(&world);

        let mut condition = Combo::default()
            .with_step(ComboStep::new(action_a).with_events(ActionEvents::ONGOING))
            .with_step(ComboStep::new(action_b).with_timeout(0.6))
            .with_step(ComboStep::new(action_c).with_timeout(0.3));

        assert_eq!(
            condition.evaluate(&actions, &time, 0.0.into()),
            ActionState::Ongoing
        );
        assert_eq!(condition.step_index, 1);

        world
            .resource_mut::<Time>()
            .advance_by(Duration::from_secs_f32(0.5));
        world.entity_mut(action_a).insert(ActionEvents::empty());
        world.entity_mut(action_b).insert(ActionEvents::COMPLETED);
        let (time, actions) = state.get(&world);

        assert_eq!(
            condition.evaluate(&actions, &time, 0.0.into()),
            ActionState::Ongoing
        );
        assert_eq!(condition.step_index, 2);

        world
            .resource_mut::<Time>()
            .advance_by(Duration::from_secs_f32(0.2));
        world.entity_mut(action_b).insert(ActionEvents::empty());
        world.entity_mut(action_c).insert(ActionEvents::COMPLETED);
        let (time, actions) = state.get(&world);

        assert_eq!(
            condition.evaluate(&actions, &time, 0.0.into()),
            ActionState::Fired
        );
        assert_eq!(condition.step_index, 0);

        assert_eq!(
            condition.evaluate(&actions, &time, 0.0.into()),
            ActionState::None
        );
        assert_eq!(condition.step_index, 0);
    }

    #[test]
    fn same_action() {
        let (mut world, mut state) = context::init_world();
        let action_a = world
            .spawn((Action::<A>::new(), ActionEvents::COMPLETED))
            .id();
        let (time, actions) = state.get(&world);

        let mut condition = Combo::default().with_step(action_a).with_step(action_a);

        assert_eq!(
            condition.evaluate(&actions, &time, 0.0.into()),
            ActionState::Ongoing
        );
        assert_eq!(condition.step_index, 1);

        assert_eq!(
            condition.evaluate(&actions, &time, 0.0.into()),
            ActionState::Fired
        );
        assert_eq!(condition.step_index, 0);
    }

    #[test]
    fn out_of_order() {
        let (mut world, mut state) = context::init_world();
        let action_a = world.spawn(Action::<A>::new()).id();
        let action_b = world
            .spawn((Action::<B>::new(), ActionEvents::COMPLETED))
            .id();
        let action_c = world.spawn(Action::<C>::new()).id();
        let (time, actions) = state.get(&world);

        let mut condition = Combo::default()
            .with_step(action_a)
            .with_step(action_b)
            .with_step(action_c);

        assert_eq!(
            condition.evaluate(&actions, &time, 0.0.into()),
            ActionState::None
        );
        assert_eq!(condition.step_index, 0);

        world.entity_mut(action_b).insert(ActionEvents::empty()); // Clear `Completed` event.
        world.entity_mut(action_a).insert(ActionEvents::COMPLETED);
        let (time, actions) = state.get(&world);

        assert_eq!(
            condition.evaluate(&actions, &time, 0.0.into()),
            ActionState::Ongoing
        );
        assert_eq!(condition.step_index, 1);

        world.entity_mut(action_a).insert(ActionEvents::empty()); // Clear `Completed` event.
        world.entity_mut(action_c).insert(ActionEvents::COMPLETED);
        let (time, actions) = state.get(&world);

        assert_eq!(
            condition.evaluate(&actions, &time, 0.0.into()),
            ActionState::None
        );
        assert_eq!(condition.step_index, 0);
    }

    #[test]
    fn ignore_same_cancel_action() {
        let (mut world, mut state) = context::init_world();
        let action_a = world
            .spawn((Action::<A>::new(), ActionEvents::COMPLETED))
            .id();
        let (time, actions) = state.get(&world);

        let mut condition = Combo::default().with_step(action_a).with_cancel(action_a);
        assert_eq!(
            condition.evaluate(&actions, &time, 0.0.into()),
            ActionState::Fired
        );
        assert_eq!(condition.step_index, 0);
    }

    #[test]
    fn missing_cancel_action() {
        let (mut world, mut state) = context::init_world();
        let action_a = world
            .spawn((Action::<A>::new(), ActionEvents::COMPLETED))
            .id();
        let (time, actions) = state.get(&world);

        let mut condition = Combo::default()
            .with_step(action_a)
            .with_cancel(Entity::PLACEHOLDER);
        assert_eq!(
            condition.evaluate(&actions, &time, 0.0.into()),
            ActionState::Fired
        );
        assert_eq!(condition.step_index, 0);
    }

    #[test]
    fn cancel() {
        let (mut world, mut state) = context::init_world();
        let action_a = world
            .spawn((Action::<A>::new(), ActionEvents::COMPLETED))
            .id();
        let action_b = world.spawn(Action::<B>::new()).id();
        let action_c = world.spawn(Action::<C>::new()).id();
        let (time, actions) = state.get(&world);

        let mut condition = Combo::default()
            .with_step(action_a)
            .with_step(action_b)
            .with_cancel(action_c);

        assert_eq!(
            condition.evaluate(&actions, &time, 0.0.into()),
            ActionState::Ongoing
        );
        assert_eq!(condition.step_index, 1);

        world.entity_mut(action_a).insert(ActionEvents::empty());
        world.entity_mut(action_b).insert(ActionEvents::COMPLETED);
        world.entity_mut(action_c).insert(ActionEvents::FIRED);
        let (time, actions) = state.get(&world);

        assert_eq!(
            condition.evaluate(&actions, &time, 0.0.into()),
            ActionState::None
        );
        assert_eq!(condition.step_index, 0);
    }

    #[derive(Debug, InputAction)]
    #[action_output(bool)]
    struct A;

    #[derive(Debug, InputAction)]
    #[action_output(bool)]
    struct B;

    #[derive(Debug, InputAction)]
    #[action_output(bool)]
    struct C;
}
