use std::collections::HashSet;
use crate::DataBuilder;


pub trait Scene: DataBuilder {
    fn source(cache: &mut HashSet<u64>) -> String;
}
