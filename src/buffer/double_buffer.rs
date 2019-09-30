use std::mem::swap;
use ocl::{self, OclScl as Scl};
use crate::Context;


pub struct DoubleBuffer<T: Scl> {
    dims: (usize, usize),
    src: ocl::Buffer<T>,
    dst: ocl::Buffer<T>,
}

impl<T: Scl> DoubleBuffer<T> {
    pub fn new(context: &Context, dims: (usize, usize)) -> crate::Result<Self> {
        let len = 3*dims.0*dims.1;

        let mkbuf = || {
            ocl::Buffer::<T>::builder()
            .queue(context.queue().clone())
            .flags(ocl::flags::MEM_READ_WRITE)
            .len(len)
            .fill_val(T::zero())
            .build()
        };

        Ok(DoubleBuffer {
            dims,
            src: mkbuf()?, dst: mkbuf()?,
        })
    }

    pub fn swap(&mut self) {
        swap(&mut self.src, &mut self.dst);
    }
    
    pub fn src_buffer(&self) -> &ocl::Buffer<T> {
        &self.buffer
    }
    pub fn src_buffer_mut(&mut self) -> &mut ocl::Buffer<T> {
        &mut self.buffer
    }

    pub fn dst_buffer(&self) -> &ocl::Buffer<T> {
        &self.buffer
    }
    pub fn dst_buffer_mut(&mut self) -> &mut ocl::Buffer<T> {
        &mut self.buffer
    }

    pub fn dims(&self) -> (usize, usize) {
        self.dims
    }
    pub fn len(&self) -> usize {
        3*self.dims.0*self.dims.1
    }
}
