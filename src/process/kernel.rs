use std::marker::PhantomData;
use ocl;
use crate::{
    prelude::*,
    Context,
    process::Program,
};


pub trait KernelSubtype<T: Push>: Push {}
impl<T: Push> KernelSubtype<T> for T {}

pub struct Kernel<T: Push> {
    context: Context,
    kernel: ocl::Kernel,
    phantom: PhantomData<T>,
}

impl<T: Push> Kernel<T> {
    pub fn new(
        context: &Context,
        program: Program,
        name: &str,
    ) -> crate::Result<(Self, String)> {
        let queue = context.queue().clone();

        let (ocl_program, message) = program.build(context)?;

        let mut kb = ocl::Kernel::builder();
        kb.program(&ocl_program)
        .name(name)
        .queue(queue.clone());
        T::args_def(&mut kb);

        let ocl_kernel = kb.build()?;

        Ok((Kernel {
            context: context.clone(),
            kernel: ocl_kernel,
            phantom: PhantomData,
        }, message))
    }

    pub fn run<D: Into<ocl::SpatialDims>, S: KernelSubtype<T>>(
        &mut self, data: S, dims: D
    ) -> crate::Result<()> {
        let k = &mut self.kernel;
        data.args_set(0, k)?;

        unsafe {
            k.cmd()
            .global_work_size(dims)
            .enq()?;
        }

        Ok(())
    }

    pub fn kernel(&self) -> &ocl::Kernel {
        &self.kernel
    }
    pub fn kernel_mut(&mut self) -> &mut ocl::Kernel {
        &mut self.kernel
    }

    pub fn context(&self) -> &Context {
        &self.context
    }
}
