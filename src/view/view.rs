use std::collections::HashSet;
use crate::prelude::*;


/// View is an entity the represents virtual camera.
///
/// It produce initial rays (we use backward ray propagation) and defines their origin point and direction.
pub trait View: Store {
    /// Source code of the view.
    fn source(cache: &mut HashSet<u64>) -> String;
}
