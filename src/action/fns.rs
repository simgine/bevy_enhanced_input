use core::any;

use bevy::prelude::*;
use log::debug;

use crate::prelude::*;

/// Functions for type `A` associated with [`Action<A>`] component.
///
/// Used to trigger events for it and update its value.
#[derive(Component, Clone, Copy)]
#[component(immutable)]
pub(crate) struct ActionFns {
    store_value: fn(&mut EntityMut, ActionValue),
    trigger: fn(&mut Commands, Entity, Entity, ActionState, ActionEvents, ActionValue, ActionTime),
}

impl ActionFns {
    /// Creates a new instance with function pointers for action marker `A`.
    pub(super) fn new<A: InputAction>() -> Self {
        Self {
            store_value: store_value::<A>,
            trigger: trigger::<A>,
        }
    }

    /// Stores the given value in the entity's [`Action<A>`] component for which this instance was created.
    pub(crate) fn store_value(&self, action: &mut EntityMut, value: ActionValue) {
        (self.store_value)(action, value);
    }

    /// Triggers events based on [`ActionEvents`] for the action marker `A` for which this instance was created.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn trigger(
        &self,
        commands: &mut Commands,
        context: Entity,
        action: Entity,
        state: ActionState,
        events: ActionEvents,
        value: ActionValue,
        time: ActionTime,
    ) {
        (self.trigger)(commands, context, action, state, events, value, time);
    }
}

fn store_value<A: InputAction>(action: &mut EntityMut, value: ActionValue) {
    let mut action = action
        .get_mut::<Action<A>>()
        .expect("entity should be an action");

    **action = A::Output::unwrap_value(value);
}

fn trigger<A: InputAction>(
    commands: &mut Commands,
    context: Entity,
    action: Entity,
    state: ActionState,
    events: ActionEvents,
    value: ActionValue,
    time: ActionTime,
) {
    for (name, event) in events.iter_names() {
        debug!(
            "triggering `{name}` from `{}` for `{context}`",
            any::type_name::<A>()
        );

        match event {
            ActionEvents::STARTED => {
                let event = Started::<A> {
                    context,
                    action,
                    value: A::Output::unwrap_value(value),
                    state,
                };
                commands.trigger(event);
            }
            ActionEvents::ONGOING => {
                let event = Ongoing::<A> {
                    context,
                    action,
                    value: A::Output::unwrap_value(value),
                    state,
                    elapsed_secs: time.elapsed_secs,
                };
                commands.trigger(event);
            }
            ActionEvents::FIRED => {
                let event = Fired::<A> {
                    context,
                    action,
                    value: A::Output::unwrap_value(value),
                    state,
                    fired_secs: time.fired_secs,
                    elapsed_secs: time.elapsed_secs,
                };
                commands.trigger(event);
            }
            ActionEvents::CANCELED => {
                let event = Canceled::<A> {
                    context,
                    action,
                    value: A::Output::unwrap_value(value),
                    state,
                    elapsed_secs: time.elapsed_secs,
                };
                commands.trigger(event);
            }
            ActionEvents::COMPLETED => {
                let event = Completed::<A> {
                    context,
                    action,
                    value: A::Output::unwrap_value(value),
                    state,
                    fired_secs: time.fired_secs,
                    elapsed_secs: time.elapsed_secs,
                };
                commands.trigger(event);
            }
            _ => unreachable!("iteration should yield only named flags"),
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy_enhanced_input_macros::InputAction;
    use test_log::test;

    use super::*;

    #[test]
    fn none_none() {
        let events = transition(ActionState::None, ActionState::None);
        assert!(events.is_empty());
    }

    #[test]
    fn none_ongoing() {
        let events = transition(ActionState::None, ActionState::Ongoing);
        assert_eq!(events, ActionEvents::STARTED | ActionEvents::ONGOING);
    }

    #[test]
    fn none_fired() {
        let events = transition(ActionState::None, ActionState::Fired);
        assert_eq!(events, ActionEvents::STARTED | ActionEvents::FIRED);
    }

    #[test]
    fn ongoing_none() {
        let events = transition(ActionState::Ongoing, ActionState::None);
        assert_eq!(events, ActionEvents::CANCELED);
    }

    #[test]
    fn ongoing_ongoing() {
        let events = transition(ActionState::Ongoing, ActionState::Ongoing);
        assert_eq!(events, ActionEvents::ONGOING);
    }

    #[test]
    fn ongoing_fired() {
        let events = transition(ActionState::Ongoing, ActionState::Fired);
        assert_eq!(events, ActionEvents::FIRED);
    }

    #[test]
    fn fired_none() {
        let events = transition(ActionState::Fired, ActionState::None);
        assert_eq!(events, ActionEvents::COMPLETED);
    }

    #[test]
    fn fired_ongoing() {
        let events = transition(ActionState::Fired, ActionState::Ongoing);
        assert_eq!(events, ActionEvents::ONGOING);
    }

    #[test]
    fn fired_fired() {
        let events = transition(ActionState::Fired, ActionState::Fired);
        assert_eq!(events, ActionEvents::FIRED);
    }

    fn transition(initial_state: ActionState, target_state: ActionState) -> ActionEvents {
        let mut world = World::new();

        world.init_resource::<TriggeredEvents>();
        world.add_observer(
            |_trigger: Trigger<Fired<Test>>, mut events: ResMut<TriggeredEvents>| {
                events.insert(ActionEvents::FIRED);
            },
        );
        world.add_observer(
            |_trigger: Trigger<Started<Test>>, mut events: ResMut<TriggeredEvents>| {
                events.insert(ActionEvents::STARTED);
            },
        );
        world.add_observer(
            |_trigger: Trigger<Ongoing<Test>>, mut events: ResMut<TriggeredEvents>| {
                events.insert(ActionEvents::ONGOING);
            },
        );
        world.add_observer(
            |_trigger: Trigger<Completed<Test>>, mut events: ResMut<TriggeredEvents>| {
                events.insert(ActionEvents::COMPLETED);
            },
        );
        world.add_observer(
            |_trigger: Trigger<Canceled<Test>>, mut events: ResMut<TriggeredEvents>| {
                events.insert(ActionEvents::CANCELED);
            },
        );

        let events = ActionEvents::new(initial_state, target_state);
        let fns = ActionFns::new::<Test>();
        fns.trigger(
            &mut world.commands(),
            Entity::PLACEHOLDER,
            Entity::PLACEHOLDER,
            target_state,
            events,
            false.into(),
            Default::default(),
        );

        world.flush();

        *world.remove_resource::<TriggeredEvents>().unwrap()
    }

    #[derive(Resource, Default, Deref, DerefMut)]
    struct TriggeredEvents(ActionEvents);

    #[derive(InputAction)]
    #[action_output(bool)]
    struct Test;
}
