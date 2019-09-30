use ocl::{self, OclScl as Scl};
use image;
use crate::Context;


/// Rendered image is stored here.
pub struct Image<T: Scl = u8> {
    buffer: ocl::Buffer<T>,
    dims: (usize, usize),
}

impl<T: Scl> Image<T> {
    pub fn new(context: &Context, dims: (usize, usize)) -> crate::Result<Self> {
        let len = 3*dims.0*dims.1;

        let buffer = ocl::Buffer::<T>::builder()
        .queue(context.queue().clone())
        .flags(ocl::flags::MEM_READ_WRITE)
        .len(len)
        .fill_val(T::zero())
        .build()?;

        Ok(Image {
            buffer, dims,
        })
    }
    
    pub fn read(&self) -> crate::Result<Vec<T>> {
        let mut vec = vec![T::zero(); self.buffer.len()];

        self.buffer.cmd()
        .offset(0)
        .read(&mut vec)
        .enq()?;

        Ok(vec)
    }

    pub fn buffer(&self) -> &ocl::Buffer<T> {
        &self.buffer
    }
    pub fn buffer_mut(&mut self) -> &mut ocl::Buffer<T> {
        &mut self.buffer
    }

    pub fn dims(&self) -> (usize, usize) {
        self.dims
    }
    pub fn len(&self) -> usize {
        3*self.dims.0*self.dims.1
    }
}

impl Image<u8> {
    pub fn save_to_file(&self, filename: &str) -> crate::Result<()> {
        image::save_buffer(
            &filename,
            &self.read()?,
            self.dims.0 as u32, self.dims.1 as u32,
            image::RGB(8),
        )?;
        Ok(())
    }
}
