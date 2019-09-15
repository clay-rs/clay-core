use crate::{
    prelude::*,
    shape::*,
};


/// Bounding shape that contains the whole object inside.
pub trait Bound: Pack + Instance<BoundClass> {}

/// Device interface for bound.
pub enum BoundClass {}
impl Class for BoundClass {
    fn name() -> String {
        "bound".to_string()
    }
    fn methods() -> Vec<String> {
        vec!["bound".to_string()]
    }
}

/// The shape that could be put inside the specified bound.
pub trait Bounded<B: Bound> {
    /// Returns bounding shape instance.
    ///
    /// If the shape is borderless and doesn't fit into any bounding shape
    /// then `None` should be returned.
    fn bound(&self) -> Option<B>;
}

impl<T: Bound + Shape + Clone> Bounded<T> for T {
    fn bound(&self) -> Option<T> {
        Some(self.clone())
    }
}
