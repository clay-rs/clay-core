use ocl;
use rand::{Rng, thread_rng};
use crate::Context;


/// Buffer that stores necessary data for rendering (e.g. collected statistics, rng seeds, etc).
pub struct RenderBuffer {
    context: Context,
    random: ocl::Buffer<u32>,
    color: ocl::Buffer<f32>,
    n_passes: usize,
    dims: (usize, usize),
}

impl RenderBuffer {
    pub fn new(context: &Context, dims: (usize, usize)) -> crate::Result<Self> {
        let len = dims.0*dims.1;

        let random = ocl::Buffer::<u32>::builder()
        .queue(context.queue().clone())
        .flags(ocl::flags::MEM_READ_WRITE)
        .len(len)
        .fill_val(0 as u32)
        .build()?;

        let mut seed = vec![0u32; len];
        thread_rng().fill(&mut seed[..]);
        
        random.cmd()
        .offset(0)
        .write(&seed)
        .enq()?;

        let color = ocl::Buffer::<f32>::builder()
        .queue(context.queue().clone())
        .flags(ocl::flags::MEM_READ_WRITE)
        .len(3*len)
        .fill_val(0 as f32)
        .build()?;

        Ok(Self {
            context: context.clone(),
            random, color,
            n_passes: 0,
            dims,
        })
    }

    pub fn pass(&mut self) {
        self.n_passes += 1;
    }
    pub fn clear(&mut self) -> crate::Result<()> {
        self.color.cmd()
        .offset(0)
        .fill(0f32, None)
        .enq()?;

        self.n_passes = 0;
        Ok(())
    }
    
    pub fn context(&self) -> &Context {
        &self.context
    }

    pub fn random(&self) -> &ocl::Buffer<u32> {
        &self.random
    }
    pub fn random_mut(&mut self) -> &mut ocl::Buffer<u32> {
        &mut self.random
    }
    pub fn color(&self) -> &ocl::Buffer<f32> {
        &self.color
    }
    pub fn color_mut(&mut self) -> &mut ocl::Buffer<f32> {
        &mut self.color
    }
    pub fn n_passes(&self) -> usize {
        self.n_passes
    }

    pub fn dims(&self) -> (usize, usize) {
        self.dims
    }
    pub fn len(&self) -> usize {
        self.dims.0*self.dims.1
    }
}
