use std::{
    path::Path,
    collections::HashSet,
    time::{Instant, Duration},
};
use ocl::{self, prm, builders::KernelBuilder};
use ocl_include::{Hook, MemHook, ListHook};
use crate::{
    prelude::*,
    scene::Scene,
    view::View,
    Context,
    process::{Program, Kernel},
    buffer::RenderBuffer,
};


/// Defines the whole raytracing process.
///
/// It stores scene and viewer and produces workers for specific device.
pub struct Renderer<S: Scene, V: View> {
    program: Program,
    dims: (usize, usize),
    pub scene: S,
    pub view: V,
}

/// Device data of the renderer.
pub struct RenderData<S: Scene, V: View> {
    screen: RenderBuffer,
    scene_data: S::Data,
    view_data: V::Data,
}

impl<S: Scene, V: View> Renderer<S, V> {
    pub fn new<H: Hook + 'static>(
        dims: (usize, usize),
        scene: S, view: V,
        hook: H,
    ) -> crate::Result<Self> {
        let mut inst_cache = HashSet::<u64>::new();
        let list_hook = ListHook::builder()
        .add_hook(hook)
        .add_hook(
            MemHook::builder()
            .add_file(&Path::new("__gen/scene.h"), S::source(&mut inst_cache))?
            .add_file(&Path::new("__gen/view.h"), V::source(&mut inst_cache))?
            .build()
        )
        .build();
        let program = Program::new(&list_hook, &Path::new("clay_core/render.c"))?;

        Ok(Self { program, dims, scene, view })
    }

    pub fn program(&self) -> &Program {
        &self.program
    }

    pub fn create_worker(&self, context: &Context) -> crate::Result<(RenderWorker<S, V>, String)> {
        RenderWorker::new(
            context,
            self.program(),
            self.create_data(context)?,
        )
    }
}

impl<S: Scene, V: View> Store for Renderer<S, V> {
    type Data = RenderData<S, V>;

    fn create_data(&self, context: &Context) -> crate::Result<Self::Data> {
        Ok(Self::Data {
            screen: RenderBuffer::new(context, self.dims)?,
            scene_data: self.scene.create_data(context)?,
            view_data: self.view.create_data(context)?,
        })
    }

    fn update_data(&self, context: &Context, data: &mut Self::Data) -> crate::Result<()> {
        self.scene.update_data(context, &mut data.scene_data)?;
        self.view.update_data(context, &mut data.view_data)?;
        Ok(())
    }
}

impl<S: Scene, V: View> RenderData<S, V> {
    pub fn buffer(&self) -> &RenderBuffer {
        &self.screen
    }
    pub fn buffer_mut(&mut self) -> &mut RenderBuffer {
        &mut self.screen
    }
    pub fn scene(&self) -> &S::Data {
        &self.scene_data
    }
    pub fn scene_mut(&mut self) -> &mut S::Data {
        &mut self.scene_data
    }
    pub fn view(&self) -> &V::Data {
        &self.view_data
    }
    pub fn view_mut(&mut self) -> &mut V::Data {
        &mut self.view_data
    }
}

impl<S: Scene, V: View> Push for RenderData<S, V> {
    fn args_count() -> usize {
        3 + S::Data::args_count() + V::Data::args_count()
    }
    fn args_def(kb: &mut KernelBuilder) {
        kb.arg(prm::Int2::zero()); // screen size
        kb.arg(None::<&ocl::Buffer<f32>>); // color buffer
        kb.arg(None::<&ocl::Buffer<u32>>); // random
        S::Data::args_def(kb);
        V::Data::args_def(kb);
    }
    fn args_set(&mut self, i: usize, k: &mut ocl::Kernel) -> crate::Result<()> {
        let mut j = i;

        let dims = self.screen.dims();
        let dims_prm = prm::Int2::new(dims.0 as i32, dims.1 as i32);
        k.set_arg(i + 0, &dims_prm)?;
        k.set_arg(i + 1, self.screen.color_mut())?;
        k.set_arg(i + 2, self.screen.random_mut())?;
        j += 3;

        self.scene_data.args_set(j, k)?;
        j += S::Data::args_count();

        self.view_data.args_set(j, k)?;
        //j += V::Data::args_count();

        Ok(())
    }
}

/// Worker of the renderer.
///
/// It actually runs ray tracing process on the specific device and handles render data.
pub struct RenderWorker<S: Scene, V: View> {
    data: RenderData<S, V>,
    kernel: Kernel<RenderData<S, V>>,
}

impl<S: Scene, V: View> RenderWorker<S, V> {
    pub fn new(
        context: &Context,
        program: &Program,
        data: RenderData<S, V>,
    ) -> crate::Result<(Self, String)> {
        let (kernel, message) = Kernel::new(context, program)?;
        Ok((Self { data, kernel }, message))
    }

    pub fn data(&self) -> &RenderData<S, V> {
        &self.data
    }
    pub fn data_mut(&mut self) -> &mut RenderData<S, V> {
        &mut self.data
    }

    pub fn context(&self) -> &Context {
        self.kernel.context()
    }

    /// Run one ray tracing pass.
    /// During this process there only one ray will be casted for each pixel.
    pub fn run(&mut self) -> crate::Result<()> {
        self.kernel.run(&self.data, self.data.screen.dims())?;

        self.context.queue().finish()?;
        self.data_mut().screen.pass();

        Ok(())
    }

    /// Repeat ray tracing passes until elapsed time exceeds the given one.
    pub fn run_for(&mut self, time: Duration) -> crate::Result<usize> {
        let inst = Instant::now();
        let mut passes = 1;
        self.run()?;
        while inst.elapsed() < time {
            self.run()?;
            passes += 1;
        }
        Ok(passes)
    }
}
