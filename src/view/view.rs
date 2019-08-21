use std::collections::HashSet;
use crate::Customer;


pub trait View: Customer {
    fn source(cache: &mut HashSet<u64>) -> String;
}
