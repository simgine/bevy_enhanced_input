//! Demonstrates how to implement a Minecraft-style hotbar system using actions.
//!
//! Each hotbar slot is modelled as a distinct entity,
//! sharing a common [`EquipItem`] action but recording which slot they correspond to
//! via the [`EquipHotbarIndex`] component.
//! Each of these actions is bound to a different key from the number row on the keyboard.

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
    commands.spawn_scene(bsn! {
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
        }
        Player
        Actions<Player> [
            Action::<EquipItem>::new()
            EquipHotbarIndex(0)
            Bindings [
                Binding(KeyCode::Digit1)
            ]
            --
            Action::<EquipItem>::new()
            EquipHotbarIndex(1)
            Bindings [
                Binding(KeyCode::Digit2)
            ]
            --
            Action::<EquipItem>::new()
            EquipHotbarIndex(2)
            Bindings [
                Binding(KeyCode::Digit3)
            ]
            --
            Action::<EquipItem>::new()
            EquipHotbarIndex(3)
            Bindings [
                Binding(KeyCode::Digit4)
            ]
            --
            Action::<EquipItem>::new()
            EquipHotbarIndex(4)
            Bindings [
                Binding(KeyCode::Digit5)
            ]
            --
            Action::<EquipItem>::new()
            EquipHotbarIndex(5)
            Bindings [
                Binding(KeyCode::Digit6)
            ]
            --
            Action::<EquipItem>::new()
            EquipHotbarIndex(6)
            Bindings [
                Binding(KeyCode::Digit7)
            ]
            --
            Action::<EquipItem>::new()
            EquipHotbarIndex(7)
            Bindings [
                Binding(KeyCode::Digit8)
            ]
            --
            Action::<EquipItem>::new()
            EquipHotbarIndex(8)
            Bindings [
                Binding(KeyCode::Digit9)
            ]
        ]
    });
}

fn equip(
    equip: On<Start<EquipItem>>,
    actions: Query<&EquipHotbarIndex>,
    mut hotbars: Query<&mut Hotbar, With<Player>>,
) {
    let equip_index = actions.get(equip.event().action).unwrap();
    let mut hotbar = hotbars.get_mut(equip.context).unwrap();

    hotbar.equipped = equip_index.0;

    if let Some(item) = &hotbar.inventory[hotbar.equipped] {
        println!("equipped item: {item:?}");
    } else {
        println!("equipped nothing");
    }
}

#[derive(Component, Default, Clone)]
struct Player;

#[derive(InputAction)]
#[action_output(bool)]
struct EquipItem;

/// The index of the hotbar slot to equip when the [`EquipItem`] action is triggered for this entity.
#[derive(Component, Default, Clone)]
struct EquipHotbarIndex(usize);

#[derive(Component, Default, Clone)]
struct Hotbar {
    inventory: Vec<Option<Item>>,
    equipped: usize,
}

#[derive(Debug, Clone)]
enum Item {
    Torch,
    Sword,
    Bow,
    Potion,
    Food,
    Map,
}
