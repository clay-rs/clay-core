use vecmat::vec::*;

use crate::{Pack, Shape};


#[derive(Clone, Debug, Default)]
/// Axis-aligned rectangular box
pub struct AlignedBox {
    /// Position of the center of the box
    pub pos: Vec3<f64>,
    /// Half-length of each side of the box
    pub size: Vec3<f64>,
}

#[derive(Clone, Debug, Default)]
/// Parallelepiped shape
pub struct Parallelepiped {
    /// Position of the center of the parallelepiped
    pub pos: Vec3<f64>,
    /// Half-length of each side of the cuboid
    pub size: Vec3<f64>,
    /// 
}


impl Sphere {
    /// OpenCL code associated with the sphere.
    pub fn ocl_code() -> String {
        [
            format!("#define SPHERE_SIZE_INT {}", Self::size_int()),
            format!("#define SPHERE_SIZE_FLOAT {}", Self::size_float()),
            "#include <object/sphere.h>".to_string(),
        ].join("\n")
    }
}

impl Pack for Sphere {
    fn size_int() -> usize { 0 }
    fn size_float() -> usize { 4 }

    fn pack(&self, _buffer_int: &mut [i32], buffer_float: &mut [f32]) {
        for (dst, src) in buffer_float[0..3].iter_mut().zip(self.pos.data.iter()) {
            *dst = *src as f32;
        }
        buffer_float[3] = self.rad as f32;
    }

    fn unpack(_buffer_int: &[i32], buffer_float: &[f32]) -> Self {
        let mut sphere = Self::default();
        for (dst, src) in sphere.pos.data.iter_mut().zip(buffer_float[0..3].iter()) {
            *dst = *src as f64;
        }
        sphere.rad = buffer_float[3] as f64;
        sphere
    }
}

impl Shape for Sphere {
    fn ocl_shape_code() -> String {
        Self::ocl_code()
    }
    fn ocl_shape_fn() -> String {
        "sphere_hit".to_string()
    }
}

impl Bound for Sphere {
    fn ocl_bound_code() -> String {
        Self::ocl_code()
    }
    fn ocl_bound_fn() -> String {
        "sphere_bound".to_string()
    }
}
