use alloc::vec::Vec;
use core::{any::TypeId, hash::Hash, iter, mem};

use bevy::{
    ecs::{schedule::ScheduleLabel, system::SystemParam},
    input::mouse::{AccumulatedMouseMotion, AccumulatedMouseScroll},
    platform::collections::HashSet,
    prelude::*,
    utils::TypeIdMap,
};
use log::{debug, trace};

use crate::prelude::*;

pub(crate) fn update_pending(mut reader: InputReader) {
    reader.update_pending();
}

/// Input state for actions.
///
/// Actions can read binding values and optionally consume them without affecting Bevy input resources.
#[derive(SystemParam)]
pub(crate) struct InputReader<'w, 's> {
    keys: Res<'w, ButtonInput<KeyCode>>,
    mouse_buttons: Res<'w, ButtonInput<MouseButton>>,
    mouse_motion: Res<'w, AccumulatedMouseMotion>,
    mouse_scroll: Res<'w, AccumulatedMouseScroll>,
    gamepads: Query<'w, 's, &'static Gamepad>,
    action_sources: Res<'w, ActionSources>,
    consumed: ResMut<'w, ConsumedInputs>,
    pending: ResMut<'w, PendingBindings>,
    gamepad_device: Local<'s, GamepadDevice>,
    skip_ignore_check: Local<'s, bool>,
}

impl InputReader<'_, '_> {
    /// Updates list of inputs that are waiting for reset.
    pub(crate) fn update_pending(&mut self) {
        // Updated before context-consumed inputs,
        // which may still reference inputs added to the pending.
        *self.skip_ignore_check = true;

        // Temporary take the original value to avoid issues with the borrow checker.
        let mut pending = mem::take(&mut *self.pending);
        pending.ignored.clear();
        pending.bindings.retain(|&binding| {
            if self.value(binding).as_bool() {
                pending.ignored.add(binding, *self.gamepad_device);
                true
            } else {
                trace!("'{binding}' reset and no longer ignored");
                false
            }
        });
        *self.pending = pending;

        *self.skip_ignore_check = false
    }

    /// Clears all consumed values from the given schedule.
    pub(crate) fn clear_consumed<S: ScheduleLabel>(&mut self) {
        self.consumed.entry(TypeId::of::<S>()).or_default().clear();
    }

    /// Assigns a gamepad from which [`Self::value`] should read input.
    pub(crate) fn set_gamepad(&mut self, gamepad: impl Into<GamepadDevice>) {
        *self.gamepad_device = gamepad.into();
    }

    /// Returns the [`ActionValue`] for the given [`Binding`].
    ///
    /// See also [`Self::consume`] and [`Self::set_gamepad`].
    pub(crate) fn value(&self, binding: impl Into<Binding>) -> ActionValue {
        let binding = binding.into();
        match binding {
            Binding::Keyboard { key, mod_keys } => {
                let pressed = self.action_sources.keyboard
                    && self.keys.pressed(key)
                    && self.mod_keys_pressed(mod_keys)
                    && !self.ignored(binding);

                pressed.into()
            }
            Binding::MouseButton { button, mod_keys } => {
                let pressed = self.action_sources.mouse_buttons
                    && self.mouse_buttons.pressed(button)
                    && self.mod_keys_pressed(mod_keys)
                    && !self.ignored(binding);

                pressed.into()
            }
            Binding::MouseMotion { mod_keys } => {
                if !self.action_sources.mouse_motion
                    || !self.mod_keys_pressed(mod_keys)
                    || self.ignored(binding)
                {
                    return Vec2::ZERO.into();
                }

                self.mouse_motion.delta.into()
            }
            Binding::MouseWheel { mod_keys } => {
                if !self.action_sources.mouse_wheel
                    || !self.mod_keys_pressed(mod_keys)
                    || self.ignored(binding)
                {
                    return Vec2::ZERO.into();
                }

                self.mouse_scroll.delta.into()
            }
            Binding::GamepadButton(button) => {
                if !self.action_sources.gamepad_button || self.ignored(binding) {
                    return 0.0.into();
                }

                let value = match *self.gamepad_device {
                    GamepadDevice::Any => self
                        .gamepads
                        .iter()
                        .filter_map(|gamepad| gamepad.get(button))
                        .find(|&value| value != 0.0),
                    GamepadDevice::Single(entity) => self
                        .gamepads
                        .get(entity)
                        .ok()
                        .and_then(|gamepad| gamepad.get(button)),
                    GamepadDevice::None => return 0.0.into(),
                };

                value.unwrap_or_default().into()
            }
            Binding::GamepadAxis(axis) => {
                if !self.action_sources.gamepad_axis || self.ignored(binding) {
                    return 0.0.into();
                }

                let value = match *self.gamepad_device {
                    GamepadDevice::Any => self
                        .gamepads
                        .iter()
                        .filter_map(|gamepad| gamepad.get_unclamped(axis))
                        .reduce(|acc, v| acc + v),
                    GamepadDevice::Single(entity) => self
                        .gamepads
                        .get(entity)
                        .ok()
                        .and_then(|gamepad| gamepad.get(axis)),
                    GamepadDevice::None => return 0.0.into(),
                };

                let value = value.unwrap_or_default();
                value.into()
            }
            Binding::None => false.into(),
            Binding::AnyKey => {
                if self.ignored(Binding::AnyKey) {
                    return false.into();
                }

                if self.action_sources.keyboard
                    && self.keys.get_pressed().any(|&k| !self.ignored(k))
                {
                    return true.into();
                }

                if self.action_sources.mouse_buttons
                    && self.mouse_buttons.get_pressed().any(|&b| !self.ignored(b))
                {
                    return true.into();
                }

                if self.action_sources.gamepad_button {
                    match *self.gamepad_device {
                        GamepadDevice::Single(entity) => {
                            if let Ok(gamepad) = self.gamepads.get(entity) {
                                if gamepad.get_pressed().any(|&b| !self.ignored(b)) {
                                    return true.into();
                                }
                            }
                        }
                        GamepadDevice::Any => {
                            for gamepad in &self.gamepads {
                                if gamepad.get_pressed().any(|&b| !self.ignored(b)) {
                                    return true.into();
                                }
                            }
                        }
                        GamepadDevice::None => (),
                    };
                }

                false.into()
            }
        }
    }

    fn mod_keys_pressed(&self, mod_keys: ModKeys) -> bool {
        if !mod_keys.is_empty() && !self.action_sources.keyboard {
            return false;
        }

        for keys in mod_keys.iter_keys() {
            if !self.keys.any_pressed(keys) {
                return false;
            }
        }

        true
    }

    fn ignored(&self, binding: impl Into<Binding>) -> bool {
        if *self.skip_ignore_check {
            return false;
        }

        let keys_ignored =
            self.pending.ignored.any_key || self.consumed.values().any(|ignored| ignored.any_key);
        let mut iter = iter::once(&self.pending.ignored).chain(self.consumed.values());
        match binding.into() {
            Binding::Keyboard { key, mod_keys } => {
                iter.any(|i| i.keys.contains(&key) || i.mod_keys.intersects(mod_keys))
                    || keys_ignored
            }
            Binding::MouseButton { button, mod_keys } => {
                iter.any(|i| i.mouse_buttons.contains(&button) || i.mod_keys.intersects(mod_keys))
                    || keys_ignored
            }
            Binding::MouseMotion { mod_keys } => {
                iter.any(|inputs| inputs.mouse_motion || inputs.mod_keys.intersects(mod_keys))
            }
            Binding::MouseWheel { mod_keys } => {
                iter.any(|inputs| inputs.mouse_wheel || inputs.mod_keys.intersects(mod_keys))
            }
            Binding::GamepadButton(button) => {
                let input = GamepadInput {
                    gamepad: *self.gamepad_device,
                    input: button,
                };
                iter.any(|inputs| inputs.gamepad_buttons.contains(&input)) || keys_ignored
            }
            Binding::GamepadAxis(axis) => {
                let input = GamepadInput {
                    gamepad: *self.gamepad_device,
                    input: axis,
                };
                iter.any(|inputs| inputs.gamepad_axes.contains(&input))
            }
            Binding::AnyKey => keys_ignored,
            Binding::None => false,
        }
    }

    /// Consumes the binding input, making it unavailable for [`Self::value`].
    ///
    /// Clears for this schedule with [`Self::clear_consumed`].
    pub(crate) fn consume<S: ScheduleLabel>(&mut self, binding: impl Into<Binding>) {
        self.consumed
            .entry(TypeId::of::<S>())
            .or_default()
            .add(binding.into(), *self.gamepad_device);
    }
}

/// Configures which input sources are visible to actions.
///
/// Defaults to `true` for all values.
///
/// Could be used to prevent actions from being triggered
/// while interacting with the UI.
///
/// # Examples
///
/// Disables mouse buttons for actions when the cursor hovers a node with
/// an `Interaction` component. It's a required component for `Button`,
/// but you can add it to any UI node to disable specific actions on hover.
///
/// ```
/// use bevy::prelude::*;
/// use bevy_enhanced_input::prelude::*;
///
/// # let mut app = App::new();
/// app.add_systems(PreUpdate, disable_mouse.before(EnhancedInputSet::Update));
///
/// fn disable_mouse(
///     mut action_sources: ResMut<ActionSources>,
///     interactions: Query<&Interaction>,
/// ) {
///     let mouse_unused = interactions.iter().all(|&interaction| interaction == Interaction::None);
///     action_sources.mouse_buttons = mouse_unused;
///     action_sources.mouse_wheel = mouse_unused;
/// }
/// ```
#[derive(Resource, Reflect)]
pub struct ActionSources {
    pub keyboard: bool,
    pub mouse_buttons: bool,
    pub mouse_motion: bool,
    pub mouse_wheel: bool,
    pub gamepad_button: bool,
    pub gamepad_axis: bool,
}

impl Default for ActionSources {
    fn default() -> Self {
        Self {
            keyboard: true,
            mouse_buttons: true,
            mouse_motion: true,
            mouse_wheel: true,
            gamepad_button: true,
            gamepad_axis: true,
        }
    }
}

/// Inputs consumed by actions in each schedule.
///
/// These inputs will be ignored by [`InputReader::value`] in all schedules.
/// When a schedule runs, previously consumed inputs in it will be cleared.
///
/// This allows schedules like [`FixedPreUpdate`], which may run multiple times per frame,
/// to correctly handle inputs it consumes itself, while still treating inputs consumed in
/// [`PreUpdate`] as already consumed for all runs within the same frame.
#[derive(Resource, Default, Deref, DerefMut)]
pub(crate) struct ConsumedInputs(TypeIdMap<IgnoredInputs>);

/// Bindings from actions with [`ActionSettings::require_reset`] enabled that were removed.
///
/// Their inputs will be ignored by [`InputReader::value`] until they become inactive.
/// Once inactive, they will be automatically removed and no longer ignored.
#[derive(Resource, Default)]
pub(crate) struct PendingBindings {
    bindings: Vec<Binding>,

    /// Computed from [`Self::bindings`].
    ignored: IgnoredInputs,
}

impl PendingBindings {
    pub(crate) fn extend(&mut self, iter: impl Iterator<Item = Binding>) {
        self.bindings
            .extend(iter.inspect(|binding| debug!("ignoring '{binding}' until reset")));
    }
}

#[derive(Default)]
pub(crate) struct IgnoredInputs {
    keys: HashSet<KeyCode>,
    mod_keys: ModKeys,
    mouse_buttons: HashSet<MouseButton>,
    mouse_motion: bool,
    mouse_wheel: bool,
    gamepad_buttons: HashSet<GamepadInput<GamepadButton>>,
    gamepad_axes: HashSet<GamepadInput<GamepadAxis>>,
    any_key: bool,
}

impl IgnoredInputs {
    fn add(&mut self, binding: Binding, gamepad: GamepadDevice) {
        match binding {
            Binding::Keyboard { key, mod_keys } => {
                self.keys.insert(key);
                self.mod_keys.insert(mod_keys);
            }
            Binding::MouseButton { button, mod_keys } => {
                self.mouse_buttons.insert(button);
                self.mod_keys.insert(mod_keys);
            }
            Binding::MouseMotion { mod_keys } => {
                self.mouse_motion = true;
                self.mod_keys.insert(mod_keys);
            }
            Binding::MouseWheel { mod_keys } => {
                self.mouse_wheel = true;
                self.mod_keys.insert(mod_keys);
            }
            Binding::GamepadButton(button) => {
                let input = GamepadInput {
                    gamepad,
                    input: button,
                };

                self.gamepad_buttons.insert(input);
            }
            Binding::GamepadAxis(axis) => {
                let input = GamepadInput {
                    gamepad,
                    input: axis,
                };

                self.gamepad_axes.insert(input);
            }
            Binding::None => (),
            Binding::AnyKey => self.any_key = true,
        }
    }

    fn clear(&mut self) {
        self.keys.clear();
        self.mod_keys = ModKeys::empty();
        self.mouse_buttons.clear();
        self.mouse_motion = false;
        self.mouse_wheel = false;
        self.gamepad_buttons.clear();
        self.gamepad_axes.clear();
        self.any_key = false;
    }
}

/// Input and associated device.
#[derive(Hash, PartialEq, Eq)]
struct GamepadInput<T: Hash + Eq> {
    gamepad: GamepadDevice,
    input: T,
}

#[cfg(test)]
mod tests {
    use bevy::{
        ecs::system::SystemState,
        input::mouse::{MouseMotion, MouseScrollUnit, MouseWheel},
    };

    use super::*;

    #[test]
    fn keyboard() {
        let (mut world, mut state) = init_world();

        let key = KeyCode::Space;
        world.resource_mut::<ButtonInput<KeyCode>>().press(key);

        let mut reader = state.get_mut(&mut world);
        assert_eq!(reader.value(key), true.into());
        assert_eq!(reader.value(Binding::AnyKey), true.into());
        assert_eq!(reader.value(KeyCode::Escape), false.into());
        assert_eq!(reader.value(key.with_mod_keys(ModKeys::ALT)), false.into());

        reader.consume::<PreUpdate>(key);
        assert_eq!(reader.value(key), false.into());
        assert_eq!(reader.value(Binding::AnyKey), false.into());
    }

    #[test]
    fn mouse_button() {
        let (mut world, mut state) = init_world();

        let button = MouseButton::Left;
        world
            .resource_mut::<ButtonInput<MouseButton>>()
            .press(button);

        let mut reader = state.get_mut(&mut world);
        assert_eq!(reader.value(button), true.into());
        assert_eq!(reader.value(Binding::AnyKey), true.into());
        assert_eq!(reader.value(MouseButton::Right), false.into());
        assert_eq!(
            reader.value(button.with_mod_keys(ModKeys::CONTROL)),
            false.into()
        );

        reader.consume::<PreUpdate>(button);
        assert_eq!(reader.value(button), false.into());
        assert_eq!(reader.value(Binding::AnyKey), false.into());
    }

    #[test]
    fn mouse_motion() {
        let (mut world, mut state) = init_world();

        let value = Vec2::ONE;
        world.insert_resource(AccumulatedMouseMotion { delta: value });

        let binding = Binding::mouse_motion();
        let mut reader = state.get_mut(&mut world);
        reader.clear_consumed::<PreUpdate>();
        assert_eq!(reader.value(binding), value.into());
        assert_eq!(
            reader.value(binding.with_mod_keys(ModKeys::SHIFT)),
            Vec2::ZERO.into()
        );

        reader.consume::<PreUpdate>(binding);
        assert_eq!(reader.value(binding), Vec2::ZERO.into());
    }

    #[test]
    fn mouse_wheel() {
        let (mut world, mut state) = init_world();

        let value = Vec2::ONE;
        world.insert_resource(AccumulatedMouseScroll {
            unit: MouseScrollUnit::Line,
            delta: value,
        });

        let binding = Binding::mouse_wheel();
        let mut reader = state.get_mut(&mut world);
        reader.clear_consumed::<PreUpdate>();
        assert_eq!(reader.value(binding), value.into());
        assert_eq!(
            reader.value(binding.with_mod_keys(ModKeys::SUPER)),
            Vec2::ZERO.into()
        );

        reader.consume::<PreUpdate>(binding);
        assert_eq!(reader.value(binding), Vec2::ZERO.into());
    }

    #[test]
    fn gamepad_button() {
        let (mut world, mut state) = init_world();

        let value = 1.0;
        let button1 = GamepadButton::South;
        let mut gamepad1 = Gamepad::default();
        gamepad1.analog_mut().set(button1, value);
        gamepad1.digital_mut().press(button1);
        let gamepad_entity = world.spawn(gamepad1).id();

        let button2 = GamepadButton::East;
        let mut gamepad2 = Gamepad::default();
        gamepad2.analog_mut().set(button2, value);
        gamepad2.digital_mut().press(button2);
        world.spawn(gamepad2);

        let mut reader = state.get_mut(&mut world);
        reader.set_gamepad(gamepad_entity);
        assert_eq!(reader.value(button1), value.into());
        assert_eq!(
            reader.value(button2),
            0.0.into(),
            "should read only from `{gamepad_entity:?}`"
        );
        assert_eq!(reader.value(Binding::AnyKey), true.into());
        assert_eq!(reader.value(GamepadButton::North), 0.0.into());

        reader.consume::<PreUpdate>(button1);
        assert_eq!(reader.value(button1), 0.0.into());
        assert_eq!(reader.value(Binding::AnyKey), false.into());
    }

    #[test]
    fn any_gamepad_button() {
        let (mut world, mut state) = init_world();

        let value = 1.0;
        let button1 = GamepadButton::South;
        let mut gamepad1 = Gamepad::default();
        gamepad1.analog_mut().set(button1, value);
        gamepad1.digital_mut().press(button1);
        world.spawn(gamepad1);

        let button2 = GamepadButton::East;
        let mut gamepad2 = Gamepad::default();
        gamepad2.analog_mut().set(button2, value);
        gamepad2.digital_mut().press(button2);
        world.spawn(gamepad2);

        let mut reader = state.get_mut(&mut world);
        assert_eq!(reader.value(button1), value.into());
        assert_eq!(reader.value(button2), value.into());
        assert_eq!(reader.value(Binding::AnyKey), true.into());
        assert_eq!(reader.value(GamepadButton::North), 0.0.into());

        reader.consume::<PreUpdate>(button1);
        assert_eq!(reader.value(button1), 0.0.into());
        assert_eq!(reader.value(Binding::AnyKey), true.into());

        reader.consume::<PreUpdate>(button2);
        assert_eq!(reader.value(button2), 0.0.into());
        assert_eq!(reader.value(Binding::AnyKey), false.into());
    }

    #[test]
    fn gamepad_axis() {
        let (mut world, mut state) = init_world();

        let value = 1.0;
        let axis1 = GamepadAxis::LeftStickX;
        let mut gamepad1 = Gamepad::default();
        gamepad1.analog_mut().set(axis1, value);
        let gamepad_entity = world.spawn(gamepad1).id();

        let axis2 = GamepadAxis::LeftStickY;
        let mut gamepad2 = Gamepad::default();
        gamepad2.analog_mut().set(axis2, value);
        world.spawn(gamepad2);

        let mut reader = state.get_mut(&mut world);
        reader.set_gamepad(gamepad_entity);
        assert_eq!(reader.value(axis1), value.into());
        assert_eq!(
            reader.value(axis2),
            0.0.into(),
            "should read only from `{gamepad_entity:?}`"
        );
        assert_eq!(reader.value(GamepadAxis::RightStickX), 0.0.into());

        reader.consume::<PreUpdate>(axis1);
        assert_eq!(reader.value(axis1), 0.0.into());
    }

    #[test]
    fn any_gamepad_axis() {
        let (mut world, mut state) = init_world();

        let value = 1.0;
        let axis1 = GamepadAxis::LeftStickX;
        let mut gamepad1 = Gamepad::default();
        gamepad1.analog_mut().set(axis1, value);
        world.spawn(gamepad1);

        let axis2 = GamepadAxis::LeftStickY;
        let mut gamepad2 = Gamepad::default();
        gamepad2.analog_mut().set(axis2, value);
        world.spawn(gamepad2);

        let mut reader = state.get_mut(&mut world);
        assert_eq!(reader.value(axis1), value.into());
        assert_eq!(reader.value(axis2), value.into());
        assert_eq!(reader.value(GamepadAxis::RightStickX), 0.0.into());

        reader.consume::<PreUpdate>(axis1);
        assert_eq!(reader.value(axis1), 0.0.into());

        reader.consume::<PreUpdate>(axis2);
        assert_eq!(reader.value(axis2), 0.0.into());
    }

    #[test]
    fn no_gamepad() {
        let (mut world, mut state) = init_world();

        let value = 1.0;
        let axis = GamepadAxis::LeftStickX;
        let button = GamepadButton::South;
        let mut gamepad = Gamepad::default();
        gamepad.analog_mut().set(axis, value);
        gamepad.analog_mut().set(button, value);
        gamepad.digital_mut().press(button);
        world.spawn(gamepad);

        let mut reader = state.get_mut(&mut world);
        reader.set_gamepad(None);
        assert_eq!(reader.value(button), 0.0.into());
        assert_eq!(reader.value(axis), 0.0.into());
        assert_eq!(reader.value(Binding::AnyKey), false.into());
    }

    #[test]
    fn any_gamepad_axis_sum() {
        let (mut world, mut state) = init_world();

        let axis = GamepadAxis::LeftStickX;
        let mut gamepad1 = Gamepad::default();
        gamepad1.analog_mut().set(axis, 0.001);
        world.spawn(gamepad1);

        let mut gamepad2 = Gamepad::default();
        gamepad2.analog_mut().set(axis, 0.002);
        world.spawn(gamepad2);

        let mut reader = state.get_mut(&mut world);
        assert_eq!(reader.value(axis), 0.003.into());
        assert_eq!(reader.value(GamepadAxis::RightStickX), 0.0.into());

        reader.consume::<PreUpdate>(axis);
        assert_eq!(reader.value(axis), 0.0.into());
    }

    #[test]
    fn keyboard_with_modifier() {
        let (mut world, mut state) = init_world();

        let key = KeyCode::Space;
        let modifier = KeyCode::ControlLeft;
        let mut keys = world.resource_mut::<ButtonInput<KeyCode>>();
        keys.press(modifier);
        keys.press(key);

        let binding = key.with_mod_keys(modifier.into());
        let mut reader = state.get_mut(&mut world);
        assert_eq!(reader.value(binding), true.into());
        assert_eq!(reader.value(key), true.into());
        assert_eq!(reader.value(Binding::AnyKey), true.into());
        assert_eq!(
            reader.value(binding.with_mod_keys(ModKeys::ALT)),
            false.into()
        );
        assert_eq!(
            reader.value(binding.with_mod_keys(ModKeys::CONTROL | ModKeys::ALT)),
            false.into()
        );

        reader.consume::<PreUpdate>(binding);
        assert_eq!(reader.value(binding), false.into());
        assert_eq!(
            reader.value(Binding::AnyKey),
            true.into(),
            "should still be pressed due to modifier"
        );

        // Try another key, but with the same modifier that was consumed.
        let other_key = KeyCode::Enter;
        world
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(other_key);
        let other_input = other_key.with_mod_keys(modifier.into());
        let reader = state.get_mut(&mut world);
        assert_eq!(reader.value(other_input), false.into());
        assert_eq!(reader.value(other_key), true.into());
    }

    #[test]
    fn mouse_button_with_modifier() {
        let (mut world, mut state) = init_world();

        let button = MouseButton::Left;
        let modifier = KeyCode::AltLeft;
        world.resource_mut::<ButtonInput<KeyCode>>().press(modifier);
        world
            .resource_mut::<ButtonInput<MouseButton>>()
            .press(button);

        let binding = button.with_mod_keys(modifier.into());
        let mut reader = state.get_mut(&mut world);
        assert_eq!(reader.value(binding), true.into());
        assert_eq!(reader.value(button), true.into());
        assert_eq!(reader.value(Binding::AnyKey), true.into());
        assert_eq!(
            reader.value(binding.with_mod_keys(ModKeys::CONTROL)),
            false.into()
        );
        assert_eq!(
            reader.value(binding.with_mod_keys(ModKeys::CONTROL | ModKeys::ALT)),
            false.into()
        );

        reader.consume::<PreUpdate>(binding);
        assert_eq!(reader.value(binding), false.into());
        assert_eq!(
            reader.value(Binding::AnyKey),
            true.into(),
            "should still be pressed due to modifier"
        );
    }

    #[test]
    fn mouse_motion_with_modifier() {
        let (mut world, mut state) = init_world();

        let value = Vec2::ONE;
        let modifier = KeyCode::ShiftLeft;
        world.resource_mut::<ButtonInput<KeyCode>>().press(modifier);
        world.insert_resource(AccumulatedMouseMotion { delta: value });

        let binding = Binding::mouse_motion().with_mod_keys(modifier.into());
        let mut reader = state.get_mut(&mut world);
        reader.clear_consumed::<PreUpdate>();
        assert_eq!(reader.value(binding), value.into());
        assert_eq!(reader.value(binding.without_mod_keys()), value.into());
        assert_eq!(
            reader.value(binding.with_mod_keys(ModKeys::SUPER)),
            Vec2::ZERO.into()
        );
        assert_eq!(
            reader.value(binding.with_mod_keys(ModKeys::SHIFT | ModKeys::SUPER)),
            Vec2::ZERO.into()
        );

        reader.consume::<PreUpdate>(binding);
        assert_eq!(reader.value(binding), Vec2::ZERO.into());
    }

    #[test]
    fn mouse_wheel_with_modifier() {
        let (mut world, mut state) = init_world();

        let value = Vec2::ONE;
        let modifier = KeyCode::SuperLeft;
        world.resource_mut::<ButtonInput<KeyCode>>().press(modifier);
        world.insert_resource(AccumulatedMouseScroll {
            unit: MouseScrollUnit::Line,
            delta: value,
        });

        let binding = Binding::mouse_wheel().with_mod_keys(modifier.into());
        let mut reader = state.get_mut(&mut world);
        reader.clear_consumed::<PreUpdate>();
        assert_eq!(reader.value(binding), value.into());
        assert_eq!(reader.value(binding.without_mod_keys()), value.into());
        assert_eq!(
            reader.value(binding.with_mod_keys(ModKeys::SHIFT)),
            Vec2::ZERO.into()
        );
        assert_eq!(
            reader.value(binding.with_mod_keys(ModKeys::SHIFT | ModKeys::SUPER)),
            Vec2::ZERO.into()
        );

        reader.consume::<PreUpdate>(binding);
        assert_eq!(reader.value(binding), Vec2::ZERO.into());
    }

    #[test]
    fn sources() {
        let (mut world, mut state) = init_world();

        let key = KeyCode::Space;
        let mouse_button = MouseButton::Left;
        let gamepad_button = GamepadButton::South;
        let axis = GamepadAxis::LeftStickX;

        world.resource_mut::<ButtonInput<KeyCode>>().press(key);
        world
            .resource_mut::<ButtonInput<MouseButton>>()
            .press(mouse_button);

        world.insert_resource(AccumulatedMouseMotion { delta: Vec2::ONE });
        world.insert_resource(AccumulatedMouseScroll {
            unit: MouseScrollUnit::Line,
            delta: Vec2::ONE,
        });

        let mut gamepad = Gamepad::default();
        gamepad.analog_mut().set(axis, 1.0);
        gamepad.analog_mut().set(gamepad_button, 1.0);
        world.spawn(gamepad);

        let mut action_sources = world.resource_mut::<ActionSources>();
        action_sources.keyboard = false;
        action_sources.mouse_buttons = false;
        action_sources.mouse_motion = false;
        action_sources.mouse_wheel = false;
        action_sources.gamepad_button = false;
        action_sources.gamepad_axis = false;

        let mut reader = state.get_mut(&mut world);
        reader.clear_consumed::<PreUpdate>();

        assert_eq!(reader.value(key), false.into());
        assert_eq!(reader.value(mouse_button), false.into());
        assert_eq!(reader.value(Binding::mouse_motion()), Vec2::ZERO.into());
        assert_eq!(reader.value(Binding::mouse_wheel()), Vec2::ZERO.into());
        assert_eq!(reader.value(gamepad_button), 0.0.into());
        assert_eq!(reader.value(axis), 0.0.into());
    }

    fn init_world<'w, 's>() -> (World, SystemState<InputReader<'w, 's>>) {
        let mut world = World::new();
        world.init_resource::<ButtonInput<KeyCode>>();
        world.init_resource::<ButtonInput<MouseButton>>();
        world.init_resource::<Events<MouseMotion>>();
        world.init_resource::<Events<MouseWheel>>();
        world.init_resource::<ButtonInput<GamepadButton>>();
        world.init_resource::<Axis<GamepadAxis>>();
        world.init_resource::<AccumulatedMouseMotion>();
        world.init_resource::<AccumulatedMouseScroll>();
        world.init_resource::<ConsumedInputs>();
        world.init_resource::<PendingBindings>();
        world.init_resource::<ActionSources>();

        let state = SystemState::<InputReader>::new(&mut world);

        (world, state)
    }
}
