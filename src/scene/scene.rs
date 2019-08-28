use std::collections::HashSet;
use crate::Store;


pub trait Scene: Store {
    fn source(cache: &mut HashSet<u64>) -> String;
}
