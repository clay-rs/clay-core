use crate::{
    prelude::*,
    map::Map, 
    shape::ShapeMapper,
    material::Material, 
    object::Covered,
};


/// Shape of an object.
///
/// It defines the search of the point where ray intersects this shape.
pub trait Shape: Pack + Instance<ShapeClass> {
    /// Creates a new shape by applying some kind of mapping to previous one.
    ///
    /// Most common use case is applying affine transform to some unit shape.
    /// (*see `map::Affine`*)
    fn map<M: Map>(self, map: M) -> ShapeMapper<Self, M> {
        ShapeMapper { shape: self, map }
    }
    /// Transforms the shape in an object by covering it with material.
    fn cover<M: Material>(self, material: M) -> Covered<Self, M> {
        Covered::new(self, material)
    }
}

/// Device interface for shape.
///
/// How to implement in OpenCL:
/// ```c
/// #include <clay_core/shape/shape.h>
///
/// SHAPE_HIT_RET <shape>_hit(
///     SHAPE_HIT_ARGS_DEF
/// ) {
///     ...
/// }
/// ```
pub enum ShapeClass {}
impl Class for ShapeClass {
    fn name() -> String {
        "shape".to_string()
    }
    fn methods() -> Vec<String> {
        vec!["hit".to_string()]
    }
}
