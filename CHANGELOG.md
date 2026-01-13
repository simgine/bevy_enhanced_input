# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

- Added 'Serialize/Deserialize' trait bounds to `Action<C>`

## [0.20.1] - 2025-12-29

### Changed

- All preset modifiers are now applied before user modifiers from `with` to improve intuitiveness of construction.
- `Default`, `Debug`, `Clone`, `PartialEq`, and `Eq` for `Actions<C>` no longer impose trait bounds on `C`.

## [0.20.0] - 2025-11-15

### Added

- `Axial::new`, `Bidirectional::new`, `Cardinal::new`, `Ordinal::new` and `Spatial::new`.

### Removed

- `Bidirectional::ad_keys`, `Bidirectional::ws_keys`, `Bidirectional::left_right_arrow`, `Bidirectional::up_down_arrow`, `Bidirectional::left_right_dpad`, `Bidirectional::up_down_dpad`, `Ordinal::hjklyubn_keys`, `Spatial::wasd_and` and `Spatial::arrows_and`. Use `new` instead.

## [0.19.3] - 2025-11-05

### Fixed

- Links in docs.

## [0.19.2] - 2025-10-30

### Added

- `Combo` input condition.
- `Pulse::with_initial_delay` and `Pulse::initial_delay` to add special delay before the first repeat.
- `with_mod_keys()` method to all preset types (`Cardinal`, `Bidirectional`, `Ordinal`, `Spatial`, `Axial`) to apply keyboard modifiers to all bindings in the preset.

### Changed

- Warn about bindings without associated actions.

## [0.19.1] - 2025-10-13

### Changed

- Use `ShortName` for logging.

### Fixed

- `Bidirectional::up_down_arrow()` was using left and right arrows.

## [0.19.0] - 2025-10-02

### Added

- `From` impl to primitive types for `ActionValue`.

### Changed

- Update to Bevy 0.17.0.
- Rename `EnhancedInputSet` to `EnhancedInputSystems`.
- Use present tense for all event names:
  - `Pressed` -> `Press`.
  - `Released` -> `Release`.
  - `Fired` -> `Fire`.
  - `Cancelled` -> `Cancel`.
  - `Completed` -> `Complete`.
  - `Started` -> `Start`.
  Note that `Press`, `Release`, and `Cancel` collide with names from `bevy_picking` and present in both `bevy::prelude::*` and `bevy_enhanced_input::prelude::*`.
  To disambiguate, import `bevy_enhanced_input::prelude::{*, Press, Release, Cancel}`.
- Serde integration is now gated behind the `serialize` feature.
- Warn on dimension mismatch instead of panicking.

### Removed

- `ActionOutput::unwrap_value`. Use `From` impl instead.

## [0.18.2] - 2025-09-10

### Fixed

- `DuplicateRegistration` error when `ContextActivity` or `ContextPriority` are set as required context components by user.

## [0.18.1] - 2025-09-01

### Added

- Added a component `ExternallyMocked` to exclude actions from being updated. These actions are updated manually by the user.

### Changed

- Made `ActionTime::update` public

## [0.18.0] - 2025-08-26

### Added

- `Binding::AnyKey` to assign any button.
- `Cooldown` input condition.
- `timer` getter for conditions that contain one.

### Changed

- Print an error instead of panicking in `InputModKeys::with_mod_keys` when keyboard modifiers can't be applied.
- Mocking can be used without `InputPlugin`.

## [0.17.0] - 2025-08-18

### Added

- All events now include an `action` field with the entity of the action that triggered them.

### Fixed

- Target context entity on events triggered by action removals.

## [0.16.0] - 2025-08-11

### Added

- `ContextActivity<C>` component to activate or deactivate context `C`.
- `Bidirectional::left_right_dpad` and `Bidirectional::up_down_dpad`.

### Changed

- `bindings!` now properly works with trailing commas and no longer requires wrapping single elements in braces when mixed with tuples.
- `Clone`, `PartialEq`, `Eq` and `Debug` are implemented for `ActionOf<C>` even if `C` doesn't implement them.
- Don't trigger change detection on `Actions<C>` on sorting if it's already sorted.
- `InputAction` now requires `PartialEq`, and `Action<A>` implements `PartialEq`.
- Rename `Bidirectional::horizontal_arrow_keys` into `Bidirectional::left_right_arrows`.
- Rename `Bidirectional::vertical_arrow_keys` into `Bidirectional::up_down_arrows`.
- Rename `Cardinal::arrow_keys` into `Cardinal::arrows`.
- Rename `Cardinal::dpad_buttons` into `Cardinal::dpad`.
- Rename `Ordinal::numpad_keys` into `Cardinal::numpad`.

### Removed

- `DeadZone::with_lower_threshold` and `DeadZone::with_upper_threshold`. All fields are public, just use struct initialization syntax.

## [0.15.3] - 2025-08-07

### Added

- Helpers for wasd and arrow keys for `Bidirectional` preset.

## [0.15.2] - 2025-07-28

### Added

- More derives to `ActionTime`.

### Fixed

- `Chord` now caps at `ActionState::Ongoing` if any of the actions aren't `ActionState::Fired`.

## [0.15.1] - 2025-07-23

### Added

- `LinearStep` modifier.

## [0.15.0] - 2025-07-23

This update features a big rewrite into a component-based API. The core concepts remain the same, but are now expressed through ECS. It's recommended to revisit the quick start guide.

### Changed

- Input contexts are now regular components. Instead of inserting `Actions<C>`, you insert the `C` component directly.
- Actions for contexts are now represented by entities with an `ActionOf<C>` relation, where `Actions<C>` is the target located on the context entity (which now only stores entities).
- `Action` now holds only the action value and its fields, with `ActionState`, `ActionValue`, and `ActionEvents` now being components. Timing is now stored in a new `ActionTime` component.
- `InputAction` now only contains the associated `Output` type. All action settings are now expressed by the `ActionSettings` component, which can be modified at runtime.
- Derive macro for `InputAction` now uses `action_output` attribute that accepts only the type.
- Bindings for actions now represented by entities with `BindingOf<A>` relations, where `Bindings<C>` is the target located on an action entity.
- Rename `Input` into `Binding` which now a component that represents the assigned binding.
- Modifiers and conditions now regular components on action and binding entities. Custom modifiers and conditions now needs to be registered using `InputModifierAppExt::add_input_modifier` and `InputConditionAppExt::add_input_condition` respectively. To access other actions, these traits now accept a query instead of `TypeIdMap<UntypedAction>`.
- Mocking now represented by `ActionMock` component that can be added to action entities.
- Presets now represented by `SpawnableList`s and store bundles. To assign multiple items as before, just spawn multiple presets. For empty bindings inside presets we now provide convenient `Binding::None`.
- `GamepadDevice` is now a component on entities with input contexts. It now includes `GamepadDevice::None` variant to conveniently disable input from any gamepad.
- Rename `InputTime` to `ContextTime`.
- Rename `EnhancedInputSet::Trigger` to `EnhancedInputSet::Apply` since we now also update `Action<C>` from `ActionValue` here.
- Rename `InputModifier::apply` to `InputModifier::transform` to avoid name collision with `Reflect::apply`.
- Mark the input as consumed on first actuation regardless of the end action state. This produces a more expected behavior.

### Removed

- `InputContext`. The schedule now can be set during registration via `InputContextAppExt::add_input_context_to`. Priority can be dynamically controlled by `ContextPriority` component, which you can set as a required component with specific value.
- `RebindAll` and `Bind`. If you already have settings and want to reload them, you probably have an event for that anyway, since you'll likely need to change multiple things - not just bindings.

## [0.14.1] - 2025-06-26

### Fixed

- Input consumption now works properly for contexts with different schedules.
- `Pulse` condition triggered `Fired` only on the first actuation.
- `HoldAndRelease` condition fired only on exact duration match.

## [0.14.0] - 2025-06-25

### Changed

- `Action<A>` now implements `Clone` and `Copy` for any `A`.
- `ActionEvents` now implements `Serialize` and `Deserialize`.
- Split `EnhancedInputSystem` into `EnhancedInputSet::Update` (reads new inputs from the `InputReader` and updates the `Actions` components) and `EnhancedInputSet::Trigger` (triggers the events corresponding to how the `Actions` components changed).
- Move most of the `Actions<C>` functionality to an untyped struct `UntypedActions`. `Actions<C>` derefs to `UntypedActions`, so you don't have to change any call sites.

## [0.13.0] - 2025-06-19

### Added

- `Actions::mock` and related methods to mock actions.
- Add duplicative swizzles to `SwizzleAxis.`
- `Actions::bindings` to get bindings access.
- `Actions::iter` and `Actions::iter_mut` to read and write actions data.

### Changed

- Rename `Binding` event into `Bind`.
- Rename `RebuildBindings` event into `RebindAll`.
- Rename `action_instance` module into `input_context` and move `InputContext` to it.
- Conditions and modifiers now accept the newly added `InputTime` system parameter, which dereferences to `Time`. From it, you can also access `Time<Real>` if you need time that is not affected by time dilation.
- Rename `relative_speed` into `with_time_kind` and accept the newly added `TimeKind` enum instead of boolean.
- All conditions with timer no longer implement `Copy`.
- Rename `input_condition::press` into `input_condition::down` and `input_condition::just_press` into `input_condition::press`. Their structs were renamed in the previous release, but the modules weren't.
- Merge `acton_map` module into `input_action`.
- Move `action_binding`, `actions`, `events`, `input_action`, `input_binding`, `input_condition`, `input_modifier` and `preset` modules under `input_context` module.
- Make all data fields of `Action` public.
- Return the strongly typed output of an action from `Actions::value`, similar to triggers.
- Rename `ActionOutput::as_output` into `ActionOutput::unwrap_value`.
- Rename `Action` into `UntypedAction`.
- `Actions::get` now returns a typed `Action<A>`.

### Removed

- `BlockBy::events_only` and `ConditionKind::Blocker::events_only`. This functionality was added before the introduction of the pull-based API and caused inconsistencies in returned values. It was intended to be used with `Chord`. If you need an action to be part of a chord but only want to react to it when the chord is not active, just check its state in the observer.
- `ActionMap::insert`. Use the new action mocking API.
- `Action::new`, `Action::trigger_events` and `Action::update` from the public API. Use the new action mocking API.
- `ConditionTimer`. Use Bevy's `Timer`. Use `InputTime::delta_kind` if you need a configurable time dilation.
- `ActionMap`. Use Bevy's `TypeIdMap` instead.
- Getters for `Action`. Use the fields, which are now public.

## [0.12.0] - 2025-05-25

### Added

- `Actions::state`, `Actions::value` and `Actions::events` helpers to obtain specific information for an action directly.
- `ConditionTimer::with_duration`.

### Changed

- Rename `Press` into `Down`.
- Rename `JustPress` into `Press`.
- Return `Result` from `Actions::bindings` to integrate with Bevy's unified error handling system.
- Rename `Actions::action` into `Actions::get` and return `Result`.

### Fixed

- Dimension cutoff when using `SwizzleAxis` with `Axis2D`.

### Removed

- `Actions::get_action`. Use `Actions::get`.
- `Actions::get_binding`. Use `Actions::binding`.

## [0.11.0] - 2025-04-24

### Added

- `InputContext::Schedule` to control the schedule in which the context will be evaluated.

### Changed

- Update to Bevy 0.16.

## [0.10.0] - 2025-04-24

### Added

- `Clamp` modifier.
- Serde derives for `ActionState`.
- `Axial` preset to map any axis into 2-dimensional input.

### Removed

- `GamepadStick`. Use `Axial::left_stick()` or `Axial::right_stick()` instead.

## [0.9.0] - 2025-04-08

### Added

- `ActionSources` resource to control which input sources are visible to actions.

### Changed

- Rename `InputContext` into `Actions<C>` with its module. Now it's a component. Since a single entity could have multiple actions, the struct now have associated marker `C`. This marker needs to implement the newly added `InputContext` trait. We provide a derive macro for it.
- Rather than auto-inserting `Actions` when specified input context component is added, users now insert `Actions` directly. The type passed into `InputContextAppExt::add_input_context` is now just a marker for `Actions`, not necessarily a component.
- Previously users needed to insert component that was registered as input context. Now context is just a regular struct and users need to insert `Actions<C>` directly.
- Input contexts no longer associated with a component. Users need to manually insert `Actions`.
- `Binding` trigger no longer stores bindings. Just get `Actions` using `Query` and mutate it directly.
- Rename `ActionBind` into `ActionBinding` and move into `action_binding` module.
- Rename `ActionBinding::bindings` into `ActionBindings::inputs`.
- Rename `Actions::action_bind` and `Actions::get_action_bind` into `Actions::binding` and `Actions::get_binding`.
- Rename `input_bind::InputBind` into `input_binding::InputBinding`.
- Rename `InputBindSet` into `IntoBindings` and its `bindings` method into `into_bindings`.
- Rename `InputConditionSet` into `IntoConditions` and its `conditions` method into `into_conditions`.
- Rename `InputModifierSet` into `IntoModifiers` and its `modifiers` method into `into_modifiers`.
- Rename `InputBindModifierEach` and `InputBindConditionEach` into `WithModifiersEach` and `WithConditionsEach` respectively.
- Rename `InputBindModCond` into `BindingBuilder`.
- Rename `ActionsData` into `ActionMap`.
- Rename `ActionData` into `Action`.
- Move `ActionMap`, `Action` and `ActionState` into `action_map` module.

### Removed

- `InputContextRegistry`. Directly access `Actions` from entities.
- `ui_priority` and `egui_priority` features. Use `ActionSources` manually for your UI library of choice.

### Fixed

- Crash on context switch due to unpredictable observers ordering.

## [0.8.0] - 2025-03-17

### Added

- Implement `Display` for `Input` and `ModKeys`.
- `ModKeys::pressed` to get currently active modifiers.

### Changed

- Rename `ContextInstance` into `InputContext`. `InputContext` is no longer a trait. Bindings now configured via observer on `Bindings<C>` (see docs on `InputContextAppExt::add_input_context` for details). Priority now set at runtime via `InputContext::set_priority`.
- Move all child modules from `input_context` under crate root (one level upper).
- Capture gamepad buttons as `Axis1D` because triggers are also buttons and sometimes have analog value.
- Rename `input_context` module into `registry` and `context_instance` into `input_context`.
- Rename `ContextAppExt` into `InputContextAppExt`.
- Rename `ContextInstances` into `InputContextRegistry`.
- Rename `RebuildInputContexts` into `RebuildBindings`.
- Reorder constants in `ModeKeys`.
- Derive `Reflect` for `Input` and `ModKeys`.
- Derive `PartialEq` for `Input`.

### Fixed

- Reversed `west` and `east` directions for `Cardinal`.

## [0.7.2] - 2025-01-24

### Changed

- Update documentation.

## [0.7.1] - 2025-01-21

### Added

- `ContextInstance::get_action_bind` and `ContextInstance::action_bind` to access action bindings.
- `ActionBind::bindings` to access input bindings.

## [0.7.0] - 2025-01-19

### Added

- `InputAction::REQUIRE_RESET` switch to require inputs to be zero before the first activation and continue to consume them even after context removal until inputs become zero again. Previously, this behavior was always enabled but applied only to the first activation. Now it’s disabled by default and also applies to removals. I believe this new default behavior is more intuitive.

### Changed

- Snap to the target in `SmoothNudge` when the current value is close enough.
- Use `debug!` instead of `trace!` and print action name for events logging.

## [0.6.0] - 2025-01-13

### Changed

- Replace `SmoothDelta` modifier with `SmoothNudge`. It uses `StableInterpolate::smooth_nudge`, which properly interpolates inputs across frames.
- Rename `InputContext::get` into `InputContext::get_context`. We now provide `InputContext::context` for panicking version. Handling failing case is unlikely needed and most of the time users know that the context exists.
- Rename `ContextInstance::action` into `ContextInstance::get_action`. We now provide `ContextInstance::action` for panicking version.

### Removed

- `InputContext::MODE`. All contexts now work like `InputContext::Exclusive` (the default). If you want to share the same input across multiple entities, use a separate "controller" entity for a context and apply inputs to the desired entities. This approach is more efficient and explicit, so it's not worth having a special case in the crate for it.

## [0.5.0] - 2024-12-05

### Added

- `InputBindSet::with_modifiers_each` and `InputBindSet::with_conditions_each` to add conditions and modifiers to sets.

### Changed

- `Negate`'s functions `x`, `y`, `z` no longer take an `invert` parameter and assume it is `true`.
- `Negate::all`'s current function has been moved to `Negate::splat`.
- `Negate::all` no longer takes an `invert` parameter and assumes it is `true` (opposite is `Negate::none`).
- Rename `Biderectional` to `Bidirectional` to fix typo.
- Rename `InputConditions` to `InputConditionSet`.
- Rename `InputConditionSet::iter_conditions` to `InputConditionSet::conditions`.
- Rename `InputModifiers` to `InputModifierSet`.
- Rename `InputModifierSet::iter_modifiers` to `InputModifierSet::modifiers`.
- Rename `InputBindings` to `InputBindSet`.
- Rename `InputBindSet::iter_bindings` to `InputBindSet::bindings`.

### Removed

- `Negate::default` - use `Negate::all` instead.

## [0.4.0] - 2024-12-01

### Changed

- Update to Bevy 0.15.

## [0.3.0] - 2024-12-01

### Changed

- Replace `InputAction::DIM` with `InputAction::Output` where you set the type directly (`bool`, `f32`, `Vec2` or `Vec3`) instead of the `ActionValueDim`.
- `InputAction` macro now accepts `output = <Type>` instead of `dim = <ActionValueDim variant>`.
- All events now have typed values, based on `InputAction::Output`.
- Rename `ActionBind::with` into `ActionBind::to`.
- All presets now structs (see `input_context::context_instance::input_preset`) that can be passed directly into `ActionBind::to`.
- Rename `Modifiers` into `ModKeys` to avoid confusion with input modifiers.
- Rename all `modifiers` fields in the `Input` enum into `mod_keys`.
- Rename `Input::with_modifiers` and `Input::without_modifiers` into `Input::with_mod_keys` and `Input::without_mod_keys`, respectively.
- `Input::with_mod_keys` now a trait method which implemented for any type that can be converted into an input to ergonomically assign keyboard modifiers.
- `Input::without_mod_keys` no longer `const`.
- Rename all `with_modifier` and `with_condition` methods into `with_modifiers` and `with_conditions` (plural) because now they accept tuples.
- `InputBind::with_modifiers` and `InputBind::with_conditions` now trait methods which implemented for any type that can be converted into a binding to ergonomically assign modifiers and conditions.
- Move `ActionState`, `ActionData` and `ActionsData` to `context_instance` module.
- Move `InputBind` to newly created `input_context::input_bind` module.

### Fixed

- Macro hygiene.

## [0.2.1] - 2024-11-20

### Fixed

- Correctly handle generics in the derive macro.

### Changed

- Rename `Press` into `JustPress`.
- Rename `Down` into `Press` to avoid collision with `Down` from `bevy_picking`.
- Replace `ContextInstance::with_gamepad` builder with just `ContextInstance::set_gamepad` setter.

## [0.2.0] - 2024-11-03

### Added

- Logging for binding.
- `AccumulateBy` modifier.
- `ActionsData::insert_action` to insert a data for `A`.
- `ActionData::events` to get triggered events from the last update.
- `ActionData::value` to get triggered value from the last update.
- `ActionData::trigger_events` to trigger events based on the last `ActionData::update`.
- `BlockBy::events` to block only events. Could be used for chords to avoid triggering required actions.
- `Deref` for `ActionEvent::kind`.
- `ContextInstances` to public API and methods to get `ActionData` for an action.

### Changed

- All events now separate structs instead of enum.
- Modifiers now accept `ActionsData`.
- Rework `ConditionKind` and the logic around it to make the behavior close to Unreal Engine.
- Consume inputs only if the action state is not equal to `ActionState::None`.
- Remove world access from conditions and modifiers. This means that you no longer can write game-specific conditions or modifiers. But it's much nicer (and faster) to just do it in observers instead.
- Values from `Input` are now converted to the action-level dimension only after applying all input-level modifiers and conditions. This allows things like mapping the Y-axis of `ActionValue::Axis2D` into an action with `ActionValueDim::Axis1D`.
- Rename `ActionBind::with_axis2d` into `ActionBind::with_xy_axis`.
- Rename `ScaleByDelta` into `DeltaScale`.
- Rename `Released` into `Release`.
- Rename `Pressed` into `Press`.
- Rename `BlockedBy` into `BlockBy`.
- Rename `Scalar` into `Scale`.
- `ActionData::update` now accepts a value and no longer trigger events.
- Use `isize` for `InputContext::PRIORITY`.
- Replace `SmoothDelta` with `LerpDelta` that does only linear interpolation. Using easing functions for inputs doesn't make much sense.
- Modifiers are now allowed to change passed value dimensions.
- All built-in modifiers now handle values of any dimension.
- Replace `with_held_timer` with `relative_speed` that just accepts a boolean.
- Rename `HeldTimer` into `ConditionTimer`.
- Use `trace!` instead of `debug!` for triggered events.

### Removed

- `ignore_incompatible!` since no longer needed.
- `SwizzleAxis::XXX`, `SwizzleAxis::YYY` and `SwizzleAxis::ZZZ`. They encourage a bad pattern of defining actions with duplicate data. Duplicate axes inside the trigger if needed.
- `ActionData::trigger_removed`, use `ActionData::trigger_events` instead.
- `Normalize` modifier, use `DeadZone::default` instead to properly work with analogue inputs.

## [0.1.0] - 2024-10-20

Initial release.

[unreleased]: https://github.com/simgine/bevy_replicon/compare/v0.20.1...HEAD
[0.20.1]: https://github.com/simgine/bevy_replicon/compare/v0.20.0...v0.20.1
[0.20.0]: https://github.com/simgine/bevy_replicon/compare/v0.19.3...v0.20.0
[0.19.3]: https://github.com/simgine/bevy_replicon/compare/v0.19.2...v0.19.3
[0.19.2]: https://github.com/simgine/bevy_replicon/compare/v0.19.1...v0.19.2
[0.19.1]: https://github.com/simgine/bevy_replicon/compare/v0.19.0...v0.19.1
[0.19.0]: https://github.com/simgine/bevy_replicon/compare/v0.18.2...v0.19.0
[0.18.2]: https://github.com/simgine/bevy_replicon/compare/v0.18.1...v0.18.2
[0.18.1]: https://github.com/simgine/bevy_replicon/compare/v0.18.0...v0.18.1
[0.18.0]: https://github.com/simgine/bevy_replicon/compare/v0.17.0...v0.18.0
[0.17.0]: https://github.com/simgine/bevy_replicon/compare/v0.16.0...v0.17.0
[0.16.0]: https://github.com/simgine/bevy_replicon/compare/v0.15.3...v0.16.0
[0.15.3]: https://github.com/simgine/bevy_replicon/compare/v0.15.2...v0.15.3
[0.15.2]: https://github.com/simgine/bevy_replicon/compare/v0.15.1...v0.15.2
[0.15.1]: https://github.com/simgine/bevy_replicon/compare/v0.15.0...v0.15.1
[0.15.0]: https://github.com/simgine/bevy_replicon/compare/v0.14.1...v0.15.0
[0.14.1]: https://github.com/simgine/bevy_replicon/compare/v0.14.0...v0.14.1
[0.14.0]: https://github.com/simgine/bevy_replicon/compare/v0.13.0...v0.14.0
[0.13.0]: https://github.com/simgine/bevy_replicon/compare/v0.12.0...v0.13.0
[0.12.0]: https://github.com/simgine/bevy_replicon/compare/v0.11.0...v0.12.0
[0.11.0]: https://github.com/simgine/bevy_replicon/compare/v0.10.0...v0.11.0
[0.10.0]: https://github.com/simgine/bevy_replicon/compare/v0.9.0...v0.10.0
[0.9.0]: https://github.com/simgine/bevy_replicon/compare/v0.8.0...v0.9.0
[0.8.0]: https://github.com/simgine/bevy_replicon/compare/v0.7.2...v0.8.0
[0.7.2]: https://github.com/simgine/bevy_replicon/compare/v0.7.1...v0.7.2
[0.7.1]: https://github.com/simgine/bevy_replicon/compare/v0.7.0...v0.7.1
[0.7.0]: https://github.com/simgine/bevy_replicon/compare/v0.6.0...v0.7.0
[0.6.0]: https://github.com/simgine/bevy_replicon/compare/v0.5.0...v0.6.0
[0.5.0]: https://github.com/simgine/bevy_replicon/compare/v0.4.0...v0.5.0
[0.4.0]: https://github.com/simgine/bevy_replicon/compare/v0.3.0...v0.4.0
[0.3.0]: https://github.com/simgine/bevy_replicon/compare/v0.2.1...v0.3.0
[0.2.1]: https://github.com/simgine/bevy_replicon/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/simgine/bevy_replicon/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/simgine/bevy_replicon/releases/tag/v0.1.0
