use std::{
    path::Path,
    collections::HashSet,
    marker::PhantomData,
};
use ocl::{self, prm};
use ocl_include::{Hook, MemHook, ListHook};
use crate::{Context, PostProc, RenderBuffer};
use super::{Program};


pub struct PostprocCollector<P: PostProc = > {
    hooks: ListHook,
    phantom: PhantomData<(S, V)>,
}

pub struct PostprocBuilder<S: Scene, V: View> {
    program: Program,
    phantom: PhantomData<(S, V)>,
}

#[allow(dead_code)]
pub struct Postproc<S: Scene, V: View> {
    program: (Program, String),
    kernel: ocl::Kernel,
    queue: ocl::Queue,
    phantom: PhantomData<(S, V)>,
}

impl<S: Scene, V: View> PostprocBuilder<S, V> {
    pub fn program(&self) -> &Program {
        &self.program
    }
}

impl<S: Scene, V: View> Postproc<S, V> {
    pub fn builder() -> PostprocCollector<S, V> {
        PostprocCollector {
            hooks:
                ListHook::builder()
                .add_hook(crate::source())
                .build(),
            phantom: PhantomData,
        }
    }
}

impl<S: Scene, V: View> PostprocCollector<S, V> {
    pub fn add_hook<H: Hook + 'static>(&mut self, hook: H) {
        self.hooks.add_hook(hook);
    }

    pub fn collect(mut self) -> crate::Result<PostprocBuilder<S, V>> {
        let mut cache = HashSet::<u64>::new();
        self.hooks.add_hook(
            MemHook::builder()
            .add_file(&Path::new("__gen/postproc.h"), S::source())?
            .build()
        );
        let program = Program::new(&self.hooks, &Path::new("clay_core/postprocess.c"))?;

        Ok(PostprocBuilder { program, phantom: PhantomData })
    }
}

impl<S: Scene, V: View> PostprocBuilder<S, V> {
    pub fn build(self, context: &Context) -> crate::Result<Postproc<S, V>> {
        let queue = context.queue().clone();

        let ocl_prog = self.program.build(context)?;

        let kernel = ocl::Kernel::builder()
        .program(&ocl_prog.0)
        .name("draw")
        .queue(queue.clone())
        .arg(prm::Int2::zero()) // screen size
        .arg(0i32) // passes
        .arg(None::<&ocl::Buffer<prm::Float3>>) // color buffer
        .arg(None::<&ocl::Buffer<u8>>) // image
        .build()?;

        Ok(Postproc {
            program: (self.program, ocl_prog.1), kernel,
            queue, phantom: PhantomData,
        })
    }
}

impl<S: Scene, V: View> Postproc<S, V> {
    pub fn program(&self) -> &(Program, String) {
        &self.program
    }

    pub fn process(
        &mut self,
        screen: &RenderBuffer,
        image: &mut Image,
    ) -> crate::Result<()> {
        if screen.size() != image.size() {
            return Err(crate::Error::Other(format!(
                "screen size ({}) != image size ({})",
                screen.size(), image.size(),
            )));
        }

        let kernel = &mut self.kernels.draw;
        let dims = screen.dims();
        let dims = prm::Int2::new(dims.0 as i32, dims.1 as i32);
        kernel.set_arg(0, &dims)?;
        kernel.set_arg(1, &(screen.n_passes() as i32))?;
        kernel.set_arg(2, screen.color_mut())?;
        kernel.set_arg(3, screen.bytes_mut())?;

        unsafe {
            kernel
            .cmd()
            .global_work_size(screen.dims())
            .enq()?;
        }

        self.queue.finish()?;
        Ok(())
    }
}
