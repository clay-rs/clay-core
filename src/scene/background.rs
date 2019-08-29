use std::collections::HashSet;
use crate::Store;


/// Background of the scene.
pub trait Background: Store {
    fn source(cache: &mut HashSet<u64>) -> String;
}
