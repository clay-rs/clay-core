use ocl;
use crate::Context;


pub struct Image {
    bytes: ocl::Buffer<u8>,
    dims: (usize, usize),
}

impl Image {
    pub fn new(context: &Context, dims: (usize, usize)) -> crate::Result<Self> {
        let len = dims.0*dims.1;

        let bytes = ocl::Buffer::<u8>::builder()
        .queue(context.queue().clone())
        .flags(ocl::flags::MEM_WRITE_ONLY)
        .len(3*len)
        .fill_val(0 as u8)
        .build()?;

        Ok(Image {
            bytes, dims,
        })
    }
    
    pub fn read(&self) -> crate::Result<Vec<u8>> {
        let mut vec = vec![0 as u8; self.bytes.len()];

        self.bytes.cmd()
        .offset(0)
        .read(&mut vec)
        .enq()?;

        Ok(vec)
    }

    pub fn bytes(&self) -> &ocl::Buffer<u8> {
        &self.bytes
    }
    pub fn bytes_mut(&mut self) -> &mut ocl::Buffer<u8> {
        &mut self.bytes
    }

    pub fn dims(&self) -> (usize, usize) {
        self.dims
    }
    pub fn len(&self) -> usize {
        self.dims.0*self.dims.1
    }
}
