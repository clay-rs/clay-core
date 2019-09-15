use crate::{
    prelude::*,
    shape::*,
    material::*,
};


/// An abstract object that could be drawn completely.
pub trait Object: Pack + Instance<ObjectClass> {}

/// Device interface for object.
pub enum ObjectClass {}
impl Class for ObjectClass {
    fn name() -> String {
        "object".to_string()
    }
    fn methods() -> Vec<String> {
        let mut methods = ShapeClass::methods();
        methods.append(&mut MaterialClass::methods());
        methods
    }
}
