use std::collections::HashSet;
use crate::{Push, TypeHash};


pub trait Filter: Push + TypeHash + 'static {
    fn name() -> String;
    fn source(cache: &mut HashSet<u64>) -> String;
}
