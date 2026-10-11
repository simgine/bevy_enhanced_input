use alloc::vec::Vec;

use bevy::ecs::{
    error::Result,
    template::{Template, TemplateContext},
};
use smallvec::{Array, SmallVec};

/// Like [`VecTemplate`](bevy::ecs::template::VecTemplate), but for [`SmallVec`].
///
/// Can be constructed from arrays, vectors, and small vectors whose elements convert into `T`.
#[derive(Debug)]
pub struct SmallVecTemplate<T, const N: usize>(pub SmallVec<[T; N]>)
where
    [T; N]: Array<Item = T>;

impl<T, const N: usize> Default for SmallVecTemplate<T, N>
where
    [T; N]: Array<Item = T>,
{
    fn default() -> Self {
        Self(SmallVec::new())
    }
}

impl<T, const N: usize> Template for SmallVecTemplate<T, N>
where
    T: Template,
    [T; N]: Array<Item = T>,
    [T::Output; N]: Array<Item = T::Output>,
{
    type Output = SmallVec<[T::Output; N]>;

    fn build_template(&self, context: &mut TemplateContext) -> Result<Self::Output> {
        self.0
            .iter()
            .map(|template| template.build_template(context))
            .collect()
    }

    fn clone_template(&self) -> Self {
        Self(self.0.iter().map(Template::clone_template).collect())
    }
}

impl<T, U, const N: usize, const M: usize> From<[U; M]> for SmallVecTemplate<T, N>
where
    U: Into<T>,
    [T; N]: Array<Item = T>,
{
    fn from(values: [U; M]) -> Self {
        Self(values.into_iter().map(Into::into).collect())
    }
}

impl<T, U, const N: usize> From<Vec<U>> for SmallVecTemplate<T, N>
where
    U: Into<T>,
    [T; N]: Array<Item = T>,
{
    fn from(values: Vec<U>) -> Self {
        Self(values.into_iter().map(Into::into).collect())
    }
}

impl<T, A, const N: usize> From<SmallVec<A>> for SmallVecTemplate<T, N>
where
    A: Array,
    A::Item: Into<T>,
    [T; N]: Array<Item = T>,
{
    fn from(values: SmallVec<A>) -> Self {
        Self(values.into_iter().map(Into::into).collect())
    }
}
