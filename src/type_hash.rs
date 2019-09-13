use std::{
    hash::{Hash, Hasher},
    any::TypeId,
    collections::hash_map::DefaultHasher,
};

/// Trait that allows to get the hash of the Rust type.
/// The only requirements is the type shoud have a static lifetime.
pub trait TypeHash: 'static {
    fn type_hash() -> u64 {
        let mut hasher = DefaultHasher::new();
        TypeId::of::<Self>().hash(&mut hasher);
        hasher.finish()
    }
}

impl<T: 'static> TypeHash for T {}
