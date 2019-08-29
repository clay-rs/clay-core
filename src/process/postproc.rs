use std::{
    mem::swap,
    ops::Deref,
    path::Path,
    collections::HashSet,
    marker::PhantomData,
};
use ocl::{self, prm};
use ocl_include::{Hook, MemHook, ListHook};
use crate::{Context, Program, Filter, RenderBuffer, Image};


pub struct PostprocCollector<F: Filter> {
    list_hook: ListHook,
    phantom: PhantomData<F>,
}

pub struct PostprocBuilder<F: Filter> {
    program: Program,
    phantom: PhantomData<F>,
}

pub struct Postproc<F: Filter> {
    context: Context,
    k_mean: ocl::Kernel,
    k_filt: ocl::Kernel,
    k_pack: ocl::Kernel,
    host_buffer: Vec<f32>,
    buffers: (ocl::Buffer<f32>, ocl::Buffer<f32>),
    image: Image,
    dims: (usize, usize),
    pub filter: F,
}

impl<F: Filter> PostprocBuilder<F> {
    pub fn program(&self) -> &Program {
        &self.program
    }
}

impl<F: Filter> Postproc<F> {
    pub fn builder() -> PostprocCollector<F> {
        PostprocCollector {
            list_hook:
                ListHook::builder()
                .add_hook(crate::source())
                .build(),
            phantom: PhantomData,
        }
    }
}

impl<F: Filter> PostprocCollector<F> {
    pub fn add_hook<H: Hook + 'static>(&mut self, hook: H) {
        self.list_hook.add_hook(hook);
    }

    fn source(cache: &mut HashSet<u64>) -> String {
        let cpref = F::inst_name().to_uppercase();
        [
            format!("#define __FILTER_ARGS_DEF {}_ARGS_DEF", cpref),
            format!("#define __FILTER_ARGS {}_ARGS", cpref),
            format!("#define __filter_apply {}_apply", F::inst_name()),
            F::source(cache)
        ].join("\n")
    }

    pub fn collect(mut self) -> crate::Result<PostprocBuilder<F>> {
        let mut cache = HashSet::<u64>::new();
        self.list_hook.add_hook(
            MemHook::builder()
            .add_file(
                &Path::new("__gen/filter.h"),
                Self::source(&mut cache),
            )?
            .build()
        );
        let program = Program::new(
            &self.list_hook,
            &Path::new("clay_core/filter.c"),
        )?;

        Ok(PostprocBuilder { program, phantom: PhantomData })
    }
}

impl<F: Filter> PostprocBuilder<F> {
    pub fn build(
        self,
        context: &Context,
        dims: (usize, usize),
        filter: F,
    ) -> crate::Result<(Postproc<F>, String)> {
        Postproc::new(context, dims, filter, self.program)
    }
}

impl<F: Filter> Postproc<F> {
    fn build_mean(context: &Context) -> crate::Result<(ocl::Kernel, String)> {
        let queue = context.queue().clone();

        let program = Program::new(
            &crate::source(),
            &Path::new("clay_core/mean.c"),
        )?;

        let (ocl_prog, message) = program.build(context)?;

        let kernel = ocl::Kernel::builder()
        .program(&ocl_prog)
        .name("mean")
        .queue(queue.clone())
        .arg(prm::Int2::zero()) // screen size
        .arg(0i32) // dst passes
        .arg(0i32) // src passes
        .arg(None::<&ocl::Buffer<f32>>) // dst buffer
        .arg(None::<&ocl::Buffer<f32>>) // src buffer
        .build()?;

        Ok((kernel, message))
    }

    fn build_pack(context: &Context) -> crate::Result<(ocl::Kernel, String)> {
        let queue = context.queue().clone();

        let program = Program::new(
            &crate::source(),
            &Path::new("clay_core/pack.c"),
        )?;

        let (ocl_prog, message) = program.build(context)?;

        let kernel = ocl::Kernel::builder()
        .program(&ocl_prog)
        .name("pack")
        .queue(queue.clone())
        .arg(prm::Int2::zero()) // screen size
        .arg(None::<&ocl::Buffer<u32>>) // image buffer
        .arg(None::<&ocl::Buffer<f32>>) // color buffer
        .build()?;

        Ok((kernel, message))
    }

    fn create_buffer(context: &Context, dims: (usize, usize)) -> crate::Result<ocl::Buffer<f32>> {
        ocl::Buffer::<f32>::builder()
        .queue(context.queue().clone())
        .flags(ocl::flags::MEM_READ_WRITE)
        .len(3*dims.0*dims.1)
        .fill_val(0 as f32)
        .build()
        .map_err(|e| e.into())
    }

    pub fn new(
        context: &Context, dims: (usize, usize),
        filter: F, program: Program,
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

        let k_filt = kb.build()?;

        let (k_mean, _msg_mean) = Self::build_mean(context)?;
        //println!("Build log (mean.c):\n{}", _msg_mean);
        let (k_pack, _msg_pack) = Self::build_pack(context)?;
        //println!("Build log (pack.c):\n{}", _msg_pack);

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

    pub fn resize(&mut self, dims: (usize, usize)) -> crate::Result<()> {
        self.buffers = (
            Self::create_buffer(&self.context, dims)?,
            Self::create_buffer(&self.context, dims)?,
        );
        self.image = Image::new(&self.context, dims)?;
        self.dims = dims;
        Ok(())
    }

    fn dims_prm(&self) -> prm::Int2 {
        let dims = self.dims;
        prm::Int2::new(dims.0 as i32, dims.1 as i32)
    }

    fn apply_collect(&mut self, n_passes: usize, screen: &RenderBuffer) -> crate::Result<()> {
        if *screen.context() != self.context {
            let len = 3*self.dims.0*self.dims.1;
            if self.host_buffer.len() != len {
                self.host_buffer.resize(len, 0f32);
            }

            screen.color().cmd()
            .offset(0)
            .read(&mut self.host_buffer)
            .enq()?;

            self.buffers.1.cmd()
            .offset(0)
            .write(&self.host_buffer)
            .enq()?;

            self.context.queue().finish()?;
        };

        let d = self.dims_prm();
        let k = &mut self.k_mean;
        k.set_arg(0, &d)?;
        k.set_arg(1, &(n_passes as i32))?;
        k.set_arg(2, &(screen.n_passes() as i32))?;
        k.set_arg(3, &mut self.buffers.0)?;
        if *screen.context() != self.context {
            k.set_arg(4, &self.buffers.1)?;
        } else {
            k.set_arg(4, screen.color())?;
        }

        unsafe {
            k.cmd()
            .global_work_size(self.dims)
            .enq()?;
        }

        Ok(())
    }

    fn apply_filter(&mut self) -> crate::Result<()> {
        let d = self.dims_prm();
        let k = &mut self.k_filt;
        k.set_arg(0, &d)?;
        k.set_arg(1, &mut self.buffers.1)?;
        k.set_arg(2, &self.buffers.0)?;

        unsafe {
            k.cmd()
            .global_work_size(self.dims)
            .enq()?;
        }

        swap(&mut self.buffers.1, &mut self.buffers.0);
        Ok(())
    }

    pub fn process<'a, I: Iterator<Item=&'a RenderBuffer>>(
        &mut self, screens: I,
    ) -> crate::Result<()> {
        let mut n_passes = 0;
        for screen in screens {
            self.apply_collect(n_passes, screen.deref())?;
            n_passes += screen.n_passes();
        }

        self.apply_filter()?;

        self.context.queue().finish()?;
        Ok(())
    }

    pub fn process_one(&mut self, screen: &RenderBuffer) -> crate::Result<()> {
        self.process([screen].into_iter().map(|s| *s))
    }

    pub fn make_image(&mut self) -> crate::Result<()> {
        let d = self.dims_prm();
        let k = &mut self.k_pack;
        k.set_arg(0, &d)?;
        k.set_arg(1, self.image.bytes_mut())?;
        k.set_arg(2, &self.buffers.0)?;

        unsafe {
            k.cmd()
            .global_work_size(self.dims)
            .enq()?;
        }

        swap(&mut self.buffers.1, &mut self.buffers.0);
        Ok(())
    }

    pub fn buffer(&self) -> &ocl::Buffer<f32> {
        &self.buffers.0
    }
    pub fn image(&self) -> &Image {
        &self.image
    }
    pub fn dims(&self) -> (usize, usize) {
        self.dims
    } 
}
