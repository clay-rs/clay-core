use ocl;
use crate::{Push, Context, Program, Customer};


#[allow(dead_code)]
pub struct Worker<C: Customer> {
    data: C::Data,
    kernel: ocl::Kernel,
    queue: ocl::Queue,
}

impl<C: Customer> Worker<C> {
    pub fn new(
        context: &Context,
        program: &Program,
        kernel_name: String,
        data: C::Data,
    ) -> crate::Result<(Self, String)> {
        let queue = context.queue().clone();

        let (ocl_prog, message) = program.build(context)?;

        let mut kb = ocl::Kernel::builder();
        kb.program(&ocl_prog)
        .name(kernel_name)
        .queue(queue.clone());
        C::Data::args_def(&mut kb);
        
        let kernel = kb.build()?;

        Ok((Worker { data, kernel, queue }, message))
    }

    pub fn data(&self) -> &C::Data {
        &self.data
    }
    pub fn data_mut(&mut self) -> &mut C::Data {
        &mut self.data
    }

    pub fn kernel(&self) -> &ocl::Kernel {
        &self.kernel
    }
    pub fn kernel_mut(&mut self) -> &mut ocl::Kernel {
        &mut self.kernel
    }

    pub fn run(&mut self) -> crate::Result<()> {
        self.data.args_set(0, &mut self.kernel)?;
        unsafe {
            self.kernel.enq()?;
        }
        self.queue.finish()?;
        Ok(())
    }
}
