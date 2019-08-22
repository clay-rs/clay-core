use std::collections::HashSet;
use ocl::{self, builders::KernelBuilder};
use crate::{Push, Filter};


pub struct IdentityFilter {}

impl Filter for IdentityFilter {
    fn name() -> String {
        "identity_filter".to_string()
    }
    fn source(_: &mut HashSet<u64>) -> String {
        "#include <clay_core/filter/identity.h>".to_string()
    }
}

impl Push for IdentityFilter {
    fn args_count() -> usize {
        0
    }
    fn args_def(_kb: &mut KernelBuilder) {
        // pass
    }
    fn args_set(&mut self, _i: usize, _k: &mut ocl::Kernel) -> crate::Result<()> {
        Ok(())
    }
}
