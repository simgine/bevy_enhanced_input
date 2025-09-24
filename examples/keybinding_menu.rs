use core::{error::Error, fmt::Write};
use std::fs;

use bevy::{
    ecs::{
        relationship::RelatedSpawner,
        spawn::{SpawnIter, SpawnWith, SpawnableList},
    },
    input::{ButtonState, common_conditions::*, keyboard::KeyboardInput, mouse::MouseButtonInput},
    log::LogPlugin,
    prelude::*,
    ui::FocusPolicy,
};
use bevy_enhanced_input::prelude::*;
use serde::{Deserialize, Serialize};

fn main() {
    // Setup logging to display triggered events.
    let mut log_plugin = LogPlugin::default();
    log_plugin.filter += ",bevy_enhanced_input=debug";

    App::new()
        .add_plugins((
            DefaultPlugins.set(log_plugin),
            EnhancedInputPlugin,
            KeybindingMenuPlugin,
        ))
        .run();
}

struct KeybindingMenuPlugin;

impl Plugin for KeybindingMenuPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ClearColor(Color::srgb(0.9, 0.9, 0.9)))
            .add_input_context::<Player>()
            .add_observer(reload_bindings)
            .add_systems(Startup, setup)
            .add_systems(
                Update,
                (
                    update_button_text,
                    (
                        cancel_binding.run_if(input_just_pressed(KeyCode::Escape)),
                        bind,
                    )
                        .chain(),
                ),
            )
            .add_systems(PostUpdate, update_button_background);
    }
}

const BINDINGS_COUNT: usize = 3;
/// Number of input columns.
const SETTINGS_PATH: &str = "target/settings.ron";
const GAP: Val = Val::Px(10.0);
const PADDING: UiRect = UiRect::all(Val::Px(15.0));
const PANEL_BACKGROUND: BackgroundColor = BackgroundColor(Color::srgb(0.8, 0.8, 0.8));
const DARK_TEXT: TextColor = TextColor(Color::srgb(0.1, 0.1, 0.1));

fn setup(mut commands: Commands) {
    let settings = match InputSettings::read(SETTINGS_PATH) {
        Ok(settings) => {
            info!("loading settings from '{SETTINGS_PATH}'");
            settings
        }
        Err(e) => {
            info!("unable to load settings from '{SETTINGS_PATH}', switching to defaults: {e}");
            Default::default()
        }
    };

    commands.spawn(player_bundle(settings.clone()));
    commands.spawn(Camera2d);

    // We use separate root node to let dialogs cover the whole UI.
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            ..Default::default()
        },
        children![(
            Node {
                flex_direction: FlexDirection::Column,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                padding: PADDING,
                row_gap: GAP,
                ..Default::default()
            },
            children![
                actions_grid_bundle(settings.clone()),
                (
                    Node {
                        align_items: AlignItems::End,
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        justify_content: JustifyContent::End,
                        ..Default::default()
                    },
                    children![(
                        SettingsButton,
                        Children::spawn(SpawnWith(move |spawner: &mut ChildSpawner| {
                            spawner.spawn(Text::new("Apply")).observe(apply);
                        }))
                    )],
                )
            ]
        )],
    ));

    commands.insert_resource(settings);
}

/// Returns name of the field.
///
/// Strips everything before first `.` in order to turn "settings.field_name" into just "field_name".
macro_rules! field_name {
    ($path:expr) => {{
        let _validate_field = &$path;
        let full_path = stringify!($path);
        full_path
            .split_once('.')
            .map(|(_, s)| s)
            .unwrap_or(full_path)
    }};
}

/// Stores name of the [`KeyboardSettings`] field and its array index for which the binding is associated.
///
/// Used to utilize reflection when applying settings.
#[derive(Component, Clone, Copy)]
struct BindingInfo {
    field_name: &'static str,
    index: usize,
}

fn actions_grid_bundle(settings: InputSettings) -> impl Bundle {
    (
        Node {
            display: Display::Grid,
            column_gap: GAP,
            row_gap: GAP,
            grid_template_columns: vec![GridTrack::auto(); BINDINGS_COUNT + 1],
            ..Default::default()
        },
        // We could utilzie reflection to iterate over fields,
        // but in real application you most likely want to have a nice and translatable text on buttons.
        Children::spawn((
            action_row("Forward", field_name!(settings.forward), settings.forward),
            action_row("Left", field_name!(settings.left), settings.left),
            action_row(
                "Backward",
                field_name!(settings.backward),
                settings.backward,
            ),
            action_row("Right", field_name!(settings.right), settings.right),
            action_row("Jump", field_name!(settings.jump), settings.jump),
            action_row("Run", field_name!(settings.run), settings.run),
        )),
    )
}

fn action_row(
    action_name: &'static str,
    field_name: &'static str,
    bindings: [Binding; BINDINGS_COUNT],
) -> impl SpawnableList<ChildOf> {
    (
        Spawn((Text::new(action_name), DARK_TEXT)),
        SpawnWith(move |spawner: &mut ChildSpawner| {
            for (index, binding) in bindings.into_iter().enumerate() {
                spawner.spawn((
                    Node {
                        column_gap: GAP,
                        align_items: AlignItems::Center,
                        ..Default::default()
                    },
                    Children::spawn(SpawnWith(move |spawner: &mut ChildSpawner| {
                        let binding_button = spawner
                            .spawn((
                                BindingInfo { field_name, index },
                                Name::new(action_name),
                                BindingButton { binding },
                                children![Text::default()], // Will be updated automatically on `BindingButton` insertion
                            ))
                            .observe(show_binding_dialog)
                            .id();
                        spawner
                            .spawn((DeleteButton { binding_button }, children![Text::new("X")]))
                            .observe(delete_binding);
                    })),
                ));
            }
        }),
    )
}

fn delete_binding(
    click: On<Pointer<Click>>,
    mut binding_buttons: Query<(&Name, &mut BindingButton)>,
    delete_buttons: Query<&DeleteButton>,
) {
    let delete_button = delete_buttons.get(click.entity).unwrap();
    let (name, mut binding_button) = binding_buttons
        .get_mut(delete_button.binding_button)
        .expect("delete button should point to a binding button");
    info!("deleting binding for '{name}'");
    binding_button.binding = Binding::None;
}

fn show_binding_dialog(
    click: On<Pointer<Click>>,
    mut commands: Commands,
    root_entity: Single<Entity, (With<Node>, Without<ChildOf>)>,
    names: Query<&Name>,
) {
    let name = names.get(click.entity).unwrap();
    info!("starting binding for '{name}'");

    commands.entity(*root_entity).with_child((
        BindingDialog {
            binding_button: click.entity,
        },
        children![(
            Node {
                flex_direction: FlexDirection::Column,
                padding: PADDING,
                row_gap: GAP,
                ..Default::default()
            },
            PANEL_BACKGROUND,
            children![(
                TextLayout {
                    justify: Justify::Center,
                    ..Default::default()
                },
                DARK_TEXT,
                Text::new(format!(
                    "Binding \"{name}\", \npress any key or Esc to cancel",
                )),
            )]
        )],
    ));
}

fn bind(
    mut commands: Commands,
    mut key_events: EventReader<KeyboardInput>,
    mut mouse_button_events: EventReader<MouseButtonInput>,
    dialog: Single<(Entity, &BindingDialog)>,
    root_entity: Single<Entity, (With<Node>, Without<ChildOf>)>,
    mut buttons: Query<(Entity, &Name, &mut BindingButton)>,
) {
    let keys = key_events
        .read()
        .filter(|event| event.state == ButtonState::Pressed)
        .map(|event| event.key_code.into());
    let mouse_buttons = mouse_button_events
        .read()
        .filter(|event| event.state == ButtonState::Pressed)
        .map(|event| event.button.into());

    let Some(binding) = keys.chain(mouse_buttons).next() else {
        return;
    };

    let (dialog_entity, dialog) = *dialog;

    if let Some((conflict_button, name, _)) = buttons
        .iter()
        .find(|(.., button)| button.binding == binding)
    {
        info!("found conflict with '{name}' for '{binding}'");

        commands.entity(*root_entity).with_child((
            ConflictDialog {
                binding_button: dialog.binding_button,
                conflict_button,
            },
            children![(
                Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    padding: PADDING,
                    row_gap: GAP,
                    ..Default::default()
                },
                PANEL_BACKGROUND,
                children![
                    (
                        DARK_TEXT,
                        Text::new(format!("\"{binding}\" is already used by \"{name}\"",)),
                    ),
                    (
                        Node {
                            column_gap: GAP,
                            ..Default::default()
                        },
                        Children::spawn(SpawnWith(|spawner: &mut RelatedSpawner<_>| {
                            spawner
                                .spawn((SettingsButton, children![Text::new("Replace")]))
                                .observe(replace_binding);
                            spawner
                                .spawn((SettingsButton, children![Text::new("Cancel")]))
                                .observe(cancel_replace_binding);
                        }))
                    )
                ]
            )],
        ));
    } else {
        let (_, name, mut button) = buttons
            .get_mut(dialog.binding_button)
            .expect("binding dialog should point to a button with binding");
        info!("assigning '{binding}' to '{name}'");
        button.binding = binding;
    }

    commands.entity(dialog_entity).despawn();
}

fn cancel_binding(mut commands: Commands, dialog: Single<Entity, With<BindingDialog>>) {
    info!("cancelling binding");
    commands.entity(*dialog).despawn();
}

fn replace_binding(
    _on: On<Pointer<Click>>,
    mut commands: Commands,
    dialog: Single<(Entity, &ConflictDialog)>,
    mut buttons: Query<(&Name, &mut BindingButton)>,
) {
    let (dialog_entity, dialog) = *dialog;
    let (_, mut conflict_button) = buttons
        .get_mut(dialog.conflict_button)
        .expect("binding conflict should point to a button");
    let binding = conflict_button.binding;
    conflict_button.binding = Binding::None;

    let (name, mut binding_button) = buttons
        .get_mut(dialog.binding_button)
        .expect("binding should point to a button");
    binding_button.binding = binding;

    info!("reassigning binding to '{name}'");
    commands.entity(dialog_entity).despawn();
}

fn cancel_replace_binding(
    _on: On<Pointer<Click>>,
    mut commands: Commands,
    dialog: Single<Entity, With<ConflictDialog>>,
) {
    info!("cancelling replace binding");
    commands.entity(*dialog).despawn();
}

fn apply(
    _on: On<Pointer<Click>>,
    mut commands: Commands,
    mut settings: ResMut<InputSettings>,
    buttons: Query<(&BindingButton, &BindingInfo)>,
) {
    settings.clear();
    for (button, info) in &buttons {
        // Utilize reflection to write by field name.
        let field_value = settings
            .path_mut::<[Binding; BINDINGS_COUNT]>(info.field_name)
            .expect("fields with bindings should be stored as Vec");
        field_value[info.index] = button.binding;
    }

    commands.trigger(SettingsChanged);

    match settings.write(SETTINGS_PATH) {
        Ok(()) => info!("writing settings to '{SETTINGS_PATH}'"),
        Err(e) => error!("unable to write settings to '{SETTINGS_PATH}': {e}"),
    }
}

fn update_button_text(
    buttons: Query<(&BindingButton, &Children), Changed<BindingButton>>,
    mut text: Query<&mut Text>,
) {
    for (button, children) in &buttons {
        let mut iter = text.iter_many_mut(children);
        let mut text = iter.fetch_next().unwrap();
        text.clear();
        write!(text, "{}", button.binding).unwrap();
    }
}

fn update_button_background(
    mut buttons: Query<(&Interaction, &mut BackgroundColor), (Changed<Interaction>, With<Button>)>,
) {
    for (&interaction, mut background) in &mut buttons {
        *background = match interaction {
            Interaction::Pressed => Color::srgb(0.35, 0.75, 0.35).into(),
            Interaction::Hovered => Color::srgb(0.25, 0.25, 0.25).into(),
            Interaction::None => Color::srgb(0.15, 0.15, 0.15).into(),
        };
    }
}

fn reload_bindings(
    _on: On<SettingsChanged>,
    mut commands: Commands,
    settings: Res<InputSettings>,
    player: Single<Entity, With<Player>>,
) {
    commands
        .entity(*player)
        .despawn_related::<Actions<Player>>()
        .insert(player_bundle(settings.clone()));
}

#[derive(Component, Default)]
#[require(
    Button,
    Node {
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        width: Val::Px(160.0),
        height: Val::Px(35.0),
        ..Default::default()
    },
)]
struct SettingsButton;

/// Button associated with a binding.
#[derive(Component)]
#[require(SettingsButton)]
struct BindingButton {
    /// Assigned binding.
    binding: Binding,
}

/// Button that clears the associated [`BindingButton`].
#[derive(Component)]
#[require(
    Button,
    Node {
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        width: Val::Px(35.0),
        height: Val::Px(35.0),
        ..Default::default()
    },
)]
struct DeleteButton {
    /// Entity with [`BindingButton`].
    binding_button: Entity,
}

#[derive(Component, Default)]
#[require(
    Node {
        position_type: PositionType::Absolute,
        width: Val::Percent(100.0),
        height: Val::Percent(100.0),
        align_items: AlignItems::Center,
        justify_content: JustifyContent::Center,
        ..Default::default()
    },
    FocusPolicy::Block,
    BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.3)),
)]
struct Dialog;

#[derive(Component)]
#[require(Dialog)]
struct BindingDialog {
    /// Entity with [`BindingButton`] for which the dialog was triggered.
    binding_button: Entity,
}

#[derive(Component)]
#[require(Dialog)]
struct ConflictDialog {
    /// Entity with [`BindingButton`].
    binding_button: Entity,
    /// Entity with [`BindingButton`] that conflicts with [`Self::binding_button`].
    conflict_button: Entity,
}

/// Keyboard and mouse settings.
///
/// Most games assign bindings for different input sources (keyboard + mouse, gamepads, etc.) separately or
/// even only allow rebinding for keyboard and mouse.
/// For example, gamepads use sticks for movement, which are bidirectional, so it doesn't make sense to assign
/// actions like "forward" to [`GamepadAxis::LeftStickX`].
///
/// If you want to assign a specific part of the axis, such as the positive part of [`GamepadAxis::LeftStickX`],
/// you need to create your own binding enum. However, this approach is mostly used in emulators rather than games.
///
/// So in this example we assign only keyboard and mouse bindings.
#[derive(Resource, Reflect, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct InputSettings {
    pub forward: [Binding; BINDINGS_COUNT],
    pub left: [Binding; BINDINGS_COUNT],
    pub backward: [Binding; BINDINGS_COUNT],
    pub right: [Binding; BINDINGS_COUNT],
    pub jump: [Binding; BINDINGS_COUNT],
    pub run: [Binding; BINDINGS_COUNT],
    pub fire: [Binding; BINDINGS_COUNT],
}

impl InputSettings {
    fn read(path: &str) -> Result<Self, Box<dyn Error>> {
        let string = fs::read_to_string(path)?;
        let settings = ron::from_str(&string)?;
        Ok(settings)
    }

    fn write(&self, path: &str) -> Result<(), Box<dyn Error>> {
        let string = ron::ser::to_string_pretty(self, Default::default())?;
        fs::write(path, string)?;
        Ok(())
    }

    fn clear(&mut self) {
        self.forward.fill(Binding::None);
        self.left.fill(Binding::None);
        self.backward.fill(Binding::None);
        self.right.fill(Binding::None);
        self.jump.fill(Binding::None);
        self.run.fill(Binding::None);
        self.fire.fill(Binding::None);
    }
}

impl Default for InputSettings {
    fn default() -> Self {
        Self {
            forward: [KeyCode::KeyW.into(), KeyCode::ArrowUp.into(), Binding::None],
            left: [
                KeyCode::KeyA.into(),
                KeyCode::ArrowLeft.into(),
                Binding::None,
            ],
            backward: [
                KeyCode::KeyS.into(),
                KeyCode::ArrowDown.into(),
                Binding::None,
            ],
            right: [
                KeyCode::KeyD.into(),
                KeyCode::ArrowRight.into(),
                Binding::None,
            ],
            jump: [KeyCode::Space.into(), Binding::None, Binding::None],
            run: [KeyCode::ShiftLeft.into(), Binding::None, Binding::None],
            fire: [MouseButton::Left.into(), Binding::None, Binding::None],
        }
    }
}

/// Event that indicates settings changed.
///
/// We could reload bindings directly in confirmation system,
/// but in most games you need to update multiple separate things,
/// which usually nicer expressed via event.
#[derive(Event)]
struct SettingsChanged;

#[derive(Component)]
struct Player;

fn player_bundle(settings: InputSettings) -> impl Bundle {
    (
        Player,
        actions!(
            Player[
                (
                    Action::<Move>::new(),
                    Bindings::spawn((
                        Cardinal {
                            north: settings.forward[0],
                            east: settings.right[0],
                            south: settings.backward[0],
                            west: settings.left[0],
                        },
                        Cardinal {
                            north: settings.forward[1],
                            east: settings.right[1],
                            south: settings.backward[1],
                            west: settings.left[1],
                        },
                        Cardinal {
                            north: settings.forward[2],
                            east: settings.right[2],
                            south: settings.backward[2],
                            west: settings.left[2],
                        },
                    )),
                ),
                (
                    Action::<Jump>::new(),
                    Bindings::spawn(SpawnIter(settings.jump.into_iter())),
                ),
                (
                    Action::<Run>::new(),
                    Bindings::spawn(SpawnIter(settings.run.into_iter())),
                ),
                (
                    Action::<Fire>::new(),
                    Bindings::spawn(SpawnIter(settings.fire.into_iter())),
                ),
            ]
        ),
    )
}

#[derive(InputAction)]
#[action_output(Vec2)]
struct Move;

#[derive(InputAction)]
#[action_output(bool)]
struct Jump;

#[derive(InputAction)]
#[action_output(bool)]
struct Run;

#[derive(InputAction)]
#[action_output(bool)]
struct Fire;
