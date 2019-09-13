use std::collections::HashSet;
use crate::Store;


/// Scene is the something that could be rendered.
///
/// Scenes designed to be the collection of objects.
/// It is responsible for iterating over objects, handling secondary rays,
/// implementing some specific rendering techniques (like importance sampling), etc.
pub trait Scene: Store {
    fn source(cache: &mut HashSet<u64>) -> String;
}
