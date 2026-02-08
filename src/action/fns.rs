use bevy::prelude::*;
use log::{debug, warn};

use crate::prelude::{Cancel, *};

/// Functions for type `A` associated with [`Action<A>`] component.
///
/// Used to trigger events for it and update its value.
#[derive(Component, Clone, Copy)]
#[component(immutable)]
pub(crate) struct ActionFns {
    store_value: fn(&mut EntityMut, ActionValue),
    trigger: fn(&mut Commands, Entity, Entity, TriggerState, ActionEvents, ActionValue, ActionTime),
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
        state: TriggerState,
        events: ActionEvents,
        value: ActionValue,
        time: ActionTime,
    ) {
        (self.trigger)(commands, context, action, state, events, value, time);
    }
}

fn store_value<A: InputAction>(action: &mut EntityMut, value: ActionValue) {
    let dim = value.dim();
    if dim != A::Output::DIM {
        warn!(
            "action `{}` (`{}`) expects `{:?}`, but got `{dim:?}`",
            ShortName::of::<A>(),
            action.id(),
            A::Output::DIM
        );
    }

    let mut action = action
        .get_mut::<Action<A>>()
        .expect("entity should be an action");

    **action = value.into();
}

fn trigger<A: InputAction>(
    commands: &mut Commands,
    context: Entity,
    action: Entity,
    state: TriggerState,
    events: ActionEvents,
    value: ActionValue,
    time: ActionTime,
) {
    for (name, event) in events.iter_names() {
        debug!(
            "triggering `{name}` for `{}` (`{action}`) for context `{context}`",
            ShortName::of::<A>()
        );

        match event {
            ActionEvents::START => {
                let event = Start::<A> {
                    context,
                    action,
                    value: value.into(),
                    state,
                };
                commands.trigger(event);
            }
            ActionEvents::ONGOING => {
                let event = Ongoing::<A> {
                    context,
                    action,
                    value: value.into(),
                    state,
                    elapsed_secs: time.elapsed_secs,
                };
                commands.trigger(event);
            }
            ActionEvents::FIRE => {
                let event = Fire::<A> {
                    context,
                    action,
                    value: value.into(),
                    state,
                    fired_secs: time.fired_secs,
                    elapsed_secs: time.elapsed_secs,
                };
                commands.trigger(event);
            }
            ActionEvents::CANCEL => {
                let event = Cancel::<A> {
                    context,
                    action,
                    value: value.into(),
                    state,
                    elapsed_secs: time.elapsed_secs,
                };
                commands.trigger(event);
            }
            ActionEvents::COMPLETE => {
                let event = Complete::<A> {
                    context,
                    action,
                    value: value.into(),
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
        let events = transition(TriggerState::None, TriggerState::None);
        assert!(events.is_empty());
    }

    #[test]
    fn none_ongoing() {
        let events = transition(TriggerState::None, TriggerState::Ongoing);
        assert_eq!(events, ActionEvents::START | ActionEvents::ONGOING);
    }

    #[test]
    fn none_fired() {
        let events = transition(TriggerState::None, TriggerState::Fired);
        assert_eq!(events, ActionEvents::START | ActionEvents::FIRE);
    }

    #[test]
    fn ongoing_none() {
        let events = transition(TriggerState::Ongoing, TriggerState::None);
        assert_eq!(events, ActionEvents::CANCEL);
    }

    #[test]
    fn ongoing_ongoing() {
        let events = transition(TriggerState::Ongoing, TriggerState::Ongoing);
        assert_eq!(events, ActionEvents::ONGOING);
    }

    #[test]
    fn ongoing_fired() {
        let events = transition(TriggerState::Ongoing, TriggerState::Fired);
        assert_eq!(events, ActionEvents::FIRE);
    }

    #[test]
    fn fired_none() {
        let events = transition(TriggerState::Fired, TriggerState::None);
        assert_eq!(events, ActionEvents::COMPLETE);
    }

    #[test]
    fn fired_ongoing() {
        let events = transition(TriggerState::Fired, TriggerState::Ongoing);
        assert_eq!(events, ActionEvents::ONGOING);
    }

    #[test]
    fn fired_fired() {
        let events = transition(TriggerState::Fired, TriggerState::Fired);
        assert_eq!(events, ActionEvents::FIRE);
    }

    fn transition(initial_state: TriggerState, target_state: TriggerState) -> ActionEvents {
        let mut world = World::new();

        world.init_resource::<TriggeredEvents>();
        world.add_observer(|_: On<Fire<Test>>, mut events: ResMut<TriggeredEvents>| {
            events.insert(ActionEvents::FIRE);
        });
        world.add_observer(|_: On<Start<Test>>, mut events: ResMut<TriggeredEvents>| {
            events.insert(ActionEvents::START);
        });
        world.add_observer(
            |_: On<Ongoing<Test>>, mut events: ResMut<TriggeredEvents>| {
                events.insert(ActionEvents::ONGOING);
            },
        );
        world.add_observer(
            |_: On<Complete<Test>>, mut events: ResMut<TriggeredEvents>| {
                events.insert(ActionEvents::COMPLETE);
            },
        );
        world.add_observer(|_: On<Cancel<Test>>, mut events: ResMut<TriggeredEvents>| {
            events.insert(ActionEvents::CANCEL);
        });

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
