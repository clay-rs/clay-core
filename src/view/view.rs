use std::collections::HashSet;
use crate::Store;


pub trait View: Store {
    fn source(cache: &mut HashSet<u64>) -> String;
}
