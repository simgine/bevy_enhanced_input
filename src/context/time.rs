use core::time::Duration;

use bevy::{ecs::system::SystemParam, prelude::*};

/// Time resources used for input conditions and modifier evaluation.
///
/// Dereferences to [`Self::virt`], which is the default time resource
/// based on the current schedule. But you can optionally use [`Self::real`]
/// if you want the time to be unaffected by time dilation.
#[derive(SystemParam, Deref)]
pub struct ContextTime<'w> {
    #[deref]
    pub virt: Res<'w, Time>,
    pub real: Res<'w, Time<Real>>,
}

impl ContextTime<'_> {
    /// Returns the delta of the time resource corresponding to the given [`TimeKind`].
    #[must_use]
    pub fn delta_kind(&self, kind: TimeKind) -> Duration {
        match kind {
            TimeKind::Virtual => self.virt.delta(),
            TimeKind::Real => self.real.delta(),
        }
    }
}

/// Type of time resource to use.
#[derive(Reflect, Debug, Default, Clone, Copy)]
pub enum TimeKind {
    /// Corresponds to [`Time<Real>`].
    ///
    /// Real wall-clock time elapsed, not affected by pausing or scaling.
    #[default]
    Real,
    /// Corresponds to [`Time`], which contains [`Time<Virtual>`], except in the fixed schedule,
    /// where it's [`Time<Fixed>`].
    ///
    /// Virtual game time, affected by [`Time::pause`] and [`Time::relative_speed`].
    Virtual,
}
