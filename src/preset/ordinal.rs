use bevy::{
    ecs::spawn::SpawnableList,
    prelude::*,
    ptr::{MovingPtr, move_as_ptr},
};

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
        Self::new(
            KeyCode::Numpad8,
            KeyCode::Numpad9,
            KeyCode::Numpad6,
            KeyCode::Numpad3,
            KeyCode::Numpad2,
            KeyCode::Numpad1,
            KeyCode::Numpad4,
            KeyCode::Numpad7,
        )
    }

    /// Maps 8 bindings as 2-dimensional input.
    ///
    /// ```text
    /// NW  N   NE
    ///   🡴 🡱 🡵
    /// W 🡰 · 🡲 E
    ///   🡷 🡳 🡶
    /// SW  S   SE
    /// ```
    #[must_use]
    #[expect(clippy::too_many_arguments)]
    pub fn new(
        north: impl Into<Binding>,
        north_east: impl Into<Binding>,
        east: impl Into<Binding>,
        south_east: impl Into<Binding>,
        south: impl Into<Binding>,
        south_west: impl Into<Binding>,
        west: impl Into<Binding>,
        north_west: impl Into<Binding>,
    ) -> Self {
        Self {
            north: north.into(),
            north_east: north_east.into(),
            east: east.into(),
            south_east: south_east.into(),
            south: south.into(),
            south_west: south_west.into(),
            west: west.into(),
            north_west: north_west.into(),
        }
    }

    /// Applies keyboard modifiers to all bindings.
    #[must_use]
    pub fn with_mod_keys(self, mod_keys: ModKeys) -> Self {
        Self {
            north: self.north.with_mod_keys(mod_keys),
            north_east: self.north_east.with_mod_keys(mod_keys),
            east: self.east.with_mod_keys(mod_keys),
            south_east: self.south_east.with_mod_keys(mod_keys),
            south: self.south.with_mod_keys(mod_keys),
            south_west: self.south_west.with_mod_keys(mod_keys),
            west: self.west.with_mod_keys(mod_keys),
            north_west: self.north_west.with_mod_keys(mod_keys),
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

        move_as_ptr!(cardinal);
        SpawnableList::spawn(cardinal, world, entity);

        world.spawn((BindingOf(entity), ordinal.north_east, SwizzleAxis::XXZ));
        world.spawn((
            BindingOf(entity),
            SwizzleAxis::XXZ,
            Negate::y(),
            ordinal.south_east,
        ));
        world.spawn((
            BindingOf(entity),
            SwizzleAxis::XXZ,
            Negate::all(),
            ordinal.south_west,
        ));
        world.spawn((
            BindingOf(entity),
            SwizzleAxis::XXZ,
            Negate::x(),
            ordinal.north_west,
        ));
    }

    fn size_hint(&self) -> usize {
        8
    }
}
