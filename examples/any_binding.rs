use bevy::{color::palettes::tailwind::*, input::gamepad::GamepadConnectionEvent, prelude::*};
use bevy_enhanced_input::prelude::*;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, EnhancedInputPlugin))
        .init_resource::<Players>()
        .init_resource::<AllColor>()
        .add_input_context::<AllContext>()
        .add_input_context::<AssignContext>()
        .add_input_context::<GamepadContext>()
        .add_systems(Startup, setup)
        .add_systems(Update, gamepad_event)
        .run();
}

//
// Systems
//
fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);

    commands
        .spawn((
            ui_root(),
            // This context isn't device specific so will receive input from
            // the keyboard, mouse, and all gamepads connected.
            AllContext,
            actions!(AllContext[
                (
                    Action::<ClearGamepads>::new(),
                    bindings![KeyCode::KeyC],
                ),
                (
                    Action::<ChangeAllColor>::new(),
                    // This binding should always be last. See [`Binding::AnyDigital`]
                    // in the docs for more information.
                    bindings![Binding::AnyDigital],
                ),
            ]),
        ))
        .observe(clear_gamepads)
        .observe(all_color_change);
}

fn gamepad_event(
    mut connection_events: EventReader<GamepadConnectionEvent>,
    gamepad: Query<&Player, With<Gamepad>>,
    observers: Query<(Entity, &Observer)>,
    mut commands: Commands,
) {
    for connection_event in connection_events.read() {
        if connection_event.connected() {
            // Make sure that no player is currently assigned to the gamepad in
            // case the gamepad had disconnected and reconnected. If no player
            // is assigned, then add the assignment Context if the gamepad doesn't
            // have it.
            if gamepad.get(connection_event.gamepad).is_err() {
                commands
                    .entity(connection_event.gamepad)
                    .insert_if_new(assign_gamepad(connection_event.gamepad));
            }
            // Add observers to the gamepad entity when it is connected so that
            // they only trigger with the associated gamepad.
            commands
                .entity(connection_event.gamepad)
                .observe(assign_player)
                .observe(player_color_change);
        } else if connection_event.disconnected() {
            // So multiple observers don't exist on the same gamepad if it is
            // reconnected we need to remove the observers that are watching the
            // gamepad currently.
            for (entity, observer) in observers {
                if observer
                    .descriptor()
                    .entities()
                    .contains(&connection_event.gamepad)
                {
                    commands.entity(entity).try_despawn();
                }
            }
        }
    }
}

//
// Gamepad context and action bundles
//
fn assign_gamepad(gamepad: Entity) -> impl Bundle {
    (
        AssignContext,
        GamepadDevice::Single(gamepad),
        actions!(AssignContext[
            (
                Action::<AssignPlayer>::new(),
                bindings![Binding::AnyDigital],
            ),
        ]),
    )
}

fn player_gamepad(player: Player) -> impl Bundle {
    (
        player,
        GamepadContext,
        actions!(GamepadContext[
            (
                Action::<ChangePlayerColor>::new(),
                bindings![Binding::AnyDigital],
            ),
        ]),
    )
}

//
// Observers
//
fn all_color_change(
    _: Trigger<Started<ChangeAllColor>>,
    mut resource: ResMut<AllColor>,
    mut color: Single<&mut BackgroundColor, With<AllColorMarker>>,
) {
    let index = if let Some(current) = resource.current
        && (current + 1) < resource.choices.len()
    {
        current + 1
    } else {
        0
    };
    resource.current = Some(index);
    let new = resource.choices[index];
    color.0 = new;
}

fn clear_gamepads(
    _: Trigger<Started<ClearGamepads>>,
    mut players: ResMut<Players>,
    gamepads: Query<Entity, With<Gamepad>>,
    status: Query<&mut Text, With<PlayerStatusMarker>>,
    color: Query<&mut BackgroundColor, With<PlayerColor>>,
    mut commands: Commands,
) {
    info!("Clearing gamepad assignments.");
    players.current = 0;
    for mut text in status {
        *text = Text::new("Status: Inactive");
    }
    for mut background in color {
        *background = BackgroundColor(GRAY_500.into());
    }
    for gamepad in gamepads {
        // Remove the GamepadContext to prevent those actions from
        // triggering when no player is assigned to the gamepad and restore
        // assignment Context so that the next button press by the gamepad
        // will assign a player if available.
        commands
            .entity(gamepad)
            .remove_with_requires::<GamepadContext>()
            .despawn_related::<Actions<GamepadContext>>()
            .insert_if_new(assign_gamepad(gamepad));
    }
}

fn assign_player(
    trigger: Trigger<Started<AssignPlayer>>,
    gamepads: Query<Entity, With<Gamepad>>,
    status: Query<(&mut Text, &Player), With<PlayerStatusMarker>>,
    mut players: ResMut<Players>,
    mut commands: Commands,
) {
    if let Ok(gamepad) = gamepads.get(trigger.target()) {
        let next_player = match players.current + 1 {
            1 => Some(Player::One),
            2 => Some(Player::Two),
            3 => Some(Player::Three),
            4 => Some(Player::Four),
            _ => None,
        };
        if let Some(player) = next_player {
            info!("Assigning Player {:?} to Gamepad: {}", &player, &gamepad);
            commands
                .entity(gamepad)
                .remove_with_requires::<AssignContext>()
                .despawn_related::<Actions<AssignContext>>()
                .insert(player_gamepad(player));
            players.current += 1;
            for (mut text, player_num) in status {
                if *player_num == player {
                    *text = Text::new("Status: Active")
                }
            }
        }
    }
}

fn player_color_change(
    trigger: Trigger<Started<ChangePlayerColor>>,
    gamepad: Query<&Player, With<Gamepad>>,
    node: Query<(&Player, &mut BackgroundColor, &mut PlayerColor)>,
) {
    if let Ok(gamepad_player) = gamepad.get(trigger.target()) {
        for (player, mut background, mut color) in node {
            if player == gamepad_player {
                let index = if color.current + 1 < color.choices.len() {
                    color.current + 1
                } else {
                    0
                };
                color.current = index;
                let new = color.choices[index];
                *background = BackgroundColor(new);
            }
        }
    }
}

//
// Resources and components
//

#[derive(Default, Resource)]
struct Players {
    current: u8,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
enum Player {
    One,
    Two,
    Three,
    Four,
}

#[derive(Component)]
struct AllContext;

#[derive(Debug, InputAction)]
#[action_output(bool)]
struct ClearGamepads;

#[derive(Debug, InputAction)]
#[action_output(bool)]
struct ChangeAllColor;

#[derive(Component)]
struct AssignContext;

#[derive(Debug, InputAction)]
#[action_output(bool)]
struct AssignPlayer;

#[derive(Component)]
struct GamepadContext;

#[derive(Debug, InputAction)]
#[action_output(bool)]
struct ChangePlayerColor;

//
// Resources and components for UI
//

#[derive(Resource)]
struct AllColor {
    current: Option<usize>,
    choices: Vec<Color>,
}

impl Default for AllColor {
    fn default() -> Self {
        Self {
            current: None,
            choices: vec![
                RED_700.into(),
                BLUE_700.into(),
                GREEN_700.into(),
                YELLOW_500.into(),
            ],
        }
    }
}

#[derive(Component)]
struct AllColorMarker;

#[derive(Component)]
struct PlayerStatusMarker;

#[derive(Component)]
struct PlayerColor {
    current: usize,
    choices: Vec<Color>,
}

//
// UI Bundles
//
fn ui_root() -> impl Bundle {
    (
        Node {
            position_type: PositionType::Absolute,
            left: Val::VMin(2.0),
            right: Val::VMin(2.0),
            top: Val::VMin(2.0),
            bottom: Val::VMin(2.0),
            justify_content: JustifyContent::SpaceBetween,
            column_gap: Val::VMin(4.0),
            ..default()
        },
        children![ui_left_pane(), ui_right_pane()],
    )
}

fn ui_left_pane() -> impl Bundle {
    (
        Node {
            row_gap: Val::VMin(2.0),
            flex_direction: FlexDirection::Column,
            ..default()
        },
        children![ui_instructions(), ui_any_box()],
    )
}

fn ui_instructions() -> impl Bundle {
    (
        Text::new(
            "Any key or mouse button click will change the background color \
            below.\n\n\
            Press any button on a gamepad to assign the gamepad to the first \
            available player. Gamepads will keep their assignment if they are \
            disconnected. Press 'C' to clear all gamepad assignments.\n\n\
            After a gamepad is assigned to a player any button will change the \
            background color of the player's box and it will also change the \
            background color below.",
        ),
        Node {
            flex_wrap: FlexWrap::Wrap,
            flex_grow: 3.0,
            ..default()
        },
    )
}

fn ui_any_box() -> impl Bundle {
    (
        AllColorMarker,
        Node {
            height: Val::Percent(50.0),
            min_height: Val::VMin(10.0),
            border: UiRect::all(Val::VMin(1.0)),
            ..default()
        },
        BackgroundColor(GRAY_500.into()),
        BorderColor(GRAY_300.into()),
    )
}

fn ui_right_pane() -> impl Bundle {
    (
        Node {
            min_width: Val::Vw(30.0),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::SpaceBetween,
            row_gap: Val::VMin(2.0),
            ..default()
        },
        children![
            ui_gamepads(Player::One),
            ui_gamepads(Player::Two),
            ui_gamepads(Player::Three),
            ui_gamepads(Player::Four),
        ],
    )
}

fn ui_gamepads(player: Player) -> impl Bundle {
    let player_num = match player {
        Player::One => "One",
        Player::Two => "Two",
        Player::Three => "Three",
        Player::Four => "Four",
    };
    (
        Node {
            flex_direction: FlexDirection::Column,
            row_gap: Val::VMin(2.0),
            ..default()
        },
        children![
            (
                Node {
                    justify_content: JustifyContent::SpaceBetween,
                    column_gap: Val::VMin(2.0),
                    ..default()
                },
                children![
                    (
                        Text::new(format!("Player {}", player_num)),
                        Node { ..default() }
                    ),
                    (
                        player,
                        PlayerStatusMarker,
                        Text::new("Status: Inactive"),
                        Node { ..default() }
                    ),
                ]
            ),
            (
                player,
                Node {
                    height: Val::Vh(15.0),
                    min_height: Val::Vh(5.0),
                    min_width: Val::Vw(30.0),
                    border: UiRect::all(Val::VMin(1.0)),
                    ..default()
                },
                BackgroundColor(GRAY_500.into()),
                BorderColor(GRAY_300.into()),
                PlayerColor {
                    current: 0,
                    choices: match player {
                        Player::One => vec![
                            RED_900.into(),
                            RED_700.into(),
                            RED_600.into(),
                            RED_500.into(),
                        ],
                        Player::Two => vec![
                            BLUE_900.into(),
                            BLUE_700.into(),
                            BLUE_600.into(),
                            BLUE_500.into(),
                        ],
                        Player::Three => vec![
                            GREEN_900.into(),
                            GREEN_700.into(),
                            GREEN_600.into(),
                            GREEN_500.into(),
                        ],
                        Player::Four => vec![
                            YELLOW_600.into(),
                            YELLOW_500.into(),
                            YELLOW_400.into(),
                            YELLOW_300.into(),
                        ],
                    },
                }
            )
        ],
    )
}
