use std::collections::HashSet;
use crate::TypeHash;


// Something that has associated OpenCL code.
pub trait Source: 'static {
    /// Returns associated OpenCL code.
    ///
    /// Method takes `cache` argument contains hashes of types that are already included.
    /// If current type hash already in `cache` then empty string should be returned.
    fn source() {
        if !cache.insert(Self::type_hash()) {
            String::new()
        } else {
            Self::source_nocheck(cache)
        }
    }

    /// Returns associated OpenCL code.
    ///
    /// This version of method doesn't check `cache`.
    fn source_nocheck(cache: &mut HashSet<u64>) -> String;
}
