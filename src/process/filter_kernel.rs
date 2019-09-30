use ocl;
use crate::filter::Filter;


pub struct FilterKernel<F: Filter> {
    filter: F,
    kernel: ocl::Kernel,
}

impl<F: Filter> {
    pub fn new(
        context: &Context, filter: F, program: Program,
    ) -> crate::Result<(Self, String)> {
        let queue = context.queue().clone();

        let (ocl_prog, message) = program.build(context)?;

        let mut kb = ocl::Kernel::builder();
        kb.program(&ocl_prog)
        .name("filter")
        .queue(queue.clone())
        .arg(prm::Int2::zero()) // screen size
        .arg(None::<&ocl::Buffer<f32>>) // dst buffer
        .arg(None::<&ocl::Buffer<f32>>); // src buffer
        F::args_def(&mut kb);

        let kernel = kb.build()?;

        Ok((Postproc {
            context: context.clone(),
            k_mean, k_filt, k_pack,
            host_buffer: Vec::new(),
            buffers: (
                Self::create_buffer(context, dims)?,
                Self::create_buffer(context, dims)?,
            ),
            image: Image::new(context, dims)?,
            dims: dims,
            filter,
        }, message))
    }
}
