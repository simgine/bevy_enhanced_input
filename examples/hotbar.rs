//! An Example showing how to implement a Minecraft-like hotbar.
//! Press keys from the number row on the keyboard to equip different items.

use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, EnhancedInputPlugin))
        .add_input_context::<Player>()
        .add_observer(equip)
        .add_systems(Startup, spawn)
        .run();
}

fn spawn(mut commands: Commands) {
    commands.spawn((
        Hotbar {
            inventory: vec![
                Some(Item::Torch),
                Some(Item::Sword),
                Some(Item::Bow),
                None,
                Some(Item::Potion),
                Some(Item::Potion),
                Some(Item::Food),
                Some(Item::Map),
                None,
            ],
            equipped: 0,
        },
        Player,
        actions!(Player[
            (Action::<EquipItem>::new(), EquipHotbarIndex(0), bindings![KeyCode::Digit1]),
            (Action::<EquipItem>::new(), EquipHotbarIndex(1), bindings![KeyCode::Digit2]),
            (Action::<EquipItem>::new(), EquipHotbarIndex(2), bindings![KeyCode::Digit3]),
            (Action::<EquipItem>::new(), EquipHotbarIndex(3), bindings![KeyCode::Digit4]),
            (Action::<EquipItem>::new(), EquipHotbarIndex(4), bindings![KeyCode::Digit5]),
            (Action::<EquipItem>::new(), EquipHotbarIndex(5), bindings![KeyCode::Digit6]),
            (Action::<EquipItem>::new(), EquipHotbarIndex(6), bindings![KeyCode::Digit7]),
            (Action::<EquipItem>::new(), EquipHotbarIndex(7), bindings![KeyCode::Digit8]),
            (Action::<EquipItem>::new(), EquipHotbarIndex(8), bindings![KeyCode::Digit9]),
            ]
        ),
    ));
}

fn equip(
    trigger: Trigger<Started<EquipItem>>,
    actions: Query<&EquipHotbarIndex>,
    mut hotbars: Query<&mut Hotbar, With<Player>>,
) {
    let equip_index = actions.get(trigger.event().action).unwrap();
    let mut hotbar = hotbars.get_mut(trigger.target()).unwrap();

    hotbar.equipped = equip_index.0;

    if let Some(item) = &hotbar.inventory[hotbar.equipped] {
        println!("equipped item: {item:?}");
    } else {
        println!("equipped nothing");
    }
}

#[derive(Component)]
struct Player;

#[derive(InputAction)]
#[action_output(bool)]
struct EquipItem;

#[derive(Component)]
struct EquipHotbarIndex(usize);

#[derive(Component)]
struct Hotbar {
    inventory: Vec<Option<Item>>,
    equipped: usize,
}

#[derive(Debug)]
enum Item {
    Torch,
    Sword,
    Bow,
    Potion,
    Food,
    Map,
}
