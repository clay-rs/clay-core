use std::collections::HashSet;
use crate::DataBuilder;


pub trait View: DataBuilder {
    fn source(cache: &mut HashSet<u64>) -> String;
}
