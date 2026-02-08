use bevy::ecs::world::FilteredEntityMut;
use log::trace;

use crate::{condition::fns::ConditionFns, modifier::fns::ModifierFns, prelude::*};

/// Helper for computing [`TriggerState`] and [`ActionValue`] based on modifiers and conditions.
///
/// Can be used at both the input level and the action level.
pub(super) struct TriggerTracker {
    value: ActionValue,
    found_explicit: bool,
    any_explicit_fired: bool,
    found_active: bool,
    found_implicit: bool,
    all_implicits_fired: bool,
    blocked: bool,
}

impl TriggerTracker {
    #[must_use]
    pub(super) fn new(value: ActionValue) -> Self {
        Self {
            value,
            found_explicit: false,
            any_explicit_fired: false,
            found_active: false,
            found_implicit: false,
            all_implicits_fired: true,
            blocked: false,
        }
    }

    pub(super) fn apply_modifiers(
        &mut self,
        entity: &mut FilteredEntityMut,
        actions: &ActionsQuery,
        time: &ContextTime,
        fns: &ModifierFns,
    ) {
        for get_modifier in &**fns {
            let modifier = get_modifier(entity);
            let new_value = modifier.transform(actions, time, self.value);
            trace!(
                "`{modifier:?}` changes `{:?}` to `{new_value:?}`",
                self.value
            );

            self.value = new_value;
        }
    }

    pub(super) fn apply_conditions(
        &mut self,
        entity: &mut FilteredEntityMut,
        actions: &ActionsQuery,
        time: &ContextTime,
        conditions: &ConditionFns,
    ) {
        // Note: No early outs permitted!
        // All conditions must be evaluated to update their internal state/delta time.
        for get_condition in &**conditions {
            let condition = get_condition(entity);
            let state = condition.evaluate(actions, time, self.value);
            trace!("`{condition:?}` returns state `{state:?}`");
            match condition.kind() {
                ConditionKind::Explicit => {
                    self.found_explicit = true;
                    self.any_explicit_fired |= state == TriggerState::Fired;
                    self.found_active |= state != TriggerState::None;
                }
                ConditionKind::Implicit => {
                    self.found_implicit = true;
                    self.all_implicits_fired &= state == TriggerState::Fired;
                    self.found_active |= state != TriggerState::None;
                }
                ConditionKind::Blocker => {
                    self.blocked |= state == TriggerState::None;
                }
            }
        }
    }

    pub(super) fn state(&self) -> TriggerState {
        if self.blocked {
            return TriggerState::None;
        }

        if !self.found_explicit && !self.found_implicit {
            if self.value.as_bool() {
                return TriggerState::Fired;
            } else {
                return TriggerState::None;
            }
        }

        if (!self.found_explicit || self.any_explicit_fired) && self.all_implicits_fired {
            TriggerState::Fired
        } else if self.found_active {
            TriggerState::Ongoing
        } else {
            TriggerState::None
        }
    }

    pub(super) fn value(&self) -> ActionValue {
        self.value
    }

    /// Replaces the state with `other`.
    ///
    /// Preserves the value dimension.
    pub(super) fn overwrite(&mut self, other: TriggerTracker) {
        let dim = self.value.dim();
        *self = other;
        self.value = self.value.convert(dim);
    }

    /// Merges two trackers.
    ///
    /// Preserves the value dimension.
    pub(super) fn combine(&mut self, other: Self, accumulation: Accumulation) {
        let accumulated = match accumulation {
            Accumulation::MaxAbs => {
                let mut value = self.value.as_axis3d().to_array();
                let other_value = other.value.as_axis3d().to_array();
                for (axis, other_axis) in value.iter_mut().zip(other_value) {
                    if axis.abs() < other_axis.abs() {
                        *axis = other_axis;
                    }
                }
                value.into()
            }
            Accumulation::Cumulative => self.value.as_axis3d() + other.value.as_axis3d(),
        };

        self.value = ActionValue::Axis3D(accumulated).convert(self.value.dim());
        self.found_explicit |= other.found_explicit;
        self.any_explicit_fired |= other.any_explicit_fired;
        self.found_active |= other.found_active;
        self.found_implicit |= other.found_implicit;
        self.all_implicits_fired &= other.all_implicits_fired;
        self.blocked |= other.blocked;
    }
}
