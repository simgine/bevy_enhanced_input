use bevy::{ecs::spawn::SpawnableList, prelude::*, ptr::MovingPtr};

use crate::prelude::*;

/// A preset to 8 map buttons as 2-dimensional input.
#[derive(Debug, Clone, Copy)]
pub struct Ordinal<N, NE, E, SE, S, SW, W, NW> {
    pub north: N,
    pub north_east: NE,
    pub east: E,
    pub south_east: SE,
    pub south: S,
    pub south_west: SW,
    pub west: W,
    pub north_west: NW,
}

impl<N, NE, E, SE, S, SW, W, NW, T: Clone> WithBundle<T> for Ordinal<N, NE, E, SE, S, SW, W, NW> {
    type Output = Ordinal<(N, T), (NE, T), (E, T), (SE, T), (S, T), (SW, T), (W, T), (NW, T)>;

    fn with(self, bundle: T) -> Self::Output {
        Ordinal {
            north: (self.north, bundle.clone()),
            north_east: (self.north_east, bundle.clone()),
            east: (self.east, bundle.clone()),
            south_east: (self.south_east, bundle.clone()),
            south: (self.south, bundle.clone()),
            south_west: (self.south_west, bundle.clone()),
            west: (self.west, bundle.clone()),
            north_west: (self.north_west, bundle),
        }
    }
}

impl Ordinal<Binding, Binding, Binding, Binding, Binding, Binding, Binding, Binding> {
    /// Maps numpad keys as 2-dimensional input.
    #[must_use]
    pub fn numpad() -> Self {
        Self {
            north: KeyCode::Numpad8.into(),
            north_east: KeyCode::Numpad9.into(),
            east: KeyCode::Numpad6.into(),
            south_east: KeyCode::Numpad3.into(),
            south: KeyCode::Numpad2.into(),
            south_west: KeyCode::Numpad1.into(),
            west: KeyCode::Numpad4.into(),
            north_west: KeyCode::Numpad7.into(),
        }
    }

    /// Maps HJKLYUBN keys as 2-dimensional input.
    ///
    /// ```text
    /// y   k   u
    ///   🡴 🡱 🡵
    /// h 🡰 · 🡲 l
    ///   🡷 🡳 🡶
    /// b   j   n
    /// ```
    /// Common for roguelikes.
    #[must_use]
    pub fn hjklyubn_keys() -> Self {
        Self {
            north: KeyCode::KeyK.into(),
            north_east: KeyCode::KeyU.into(),
            east: KeyCode::KeyL.into(),
            south_east: KeyCode::KeyN.into(),
            south: KeyCode::KeyJ.into(),
            south_west: KeyCode::KeyB.into(),
            west: KeyCode::KeyH.into(),
            north_west: KeyCode::KeyY.into(),
        }
    }
}

impl<N, NE, E, SE, S, SW, W, NW> SpawnableList<BindingOf> for Ordinal<N, NE, E, SE, S, SW, W, NW>
where
    N: Bundle,
    NE: Bundle,
    E: Bundle,
    SE: Bundle,
    S: Bundle,
    SW: Bundle,
    W: Bundle,
    NW: Bundle,
{
    fn spawn(this: MovingPtr<'_, Self>, world: &mut World, entity: Entity) {
        let ordinal = this.read();
        let cardinal = Cardinal {
            north: ordinal.north,
            east: ordinal.east,
            south: ordinal.south,
            west: ordinal.west,
        };
        cardinal.spawn(world, entity);

        world.spawn((BindingOf(entity), ordinal.north_east, SwizzleAxis::XXZ));
        world.spawn((
            BindingOf(entity),
            ordinal.south_east,
            SwizzleAxis::XXZ,
            Negate::y(),
        ));
        world.spawn((
            BindingOf(entity),
            ordinal.south_west,
            SwizzleAxis::XXZ,
            Negate::all(),
        ));
        world.spawn((
            BindingOf(entity),
            ordinal.north_west,
            SwizzleAxis::XXZ,
            Negate::x(),
        ));
    }

    fn size_hint(&self) -> usize {
        8
    }
}
