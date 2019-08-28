use std::{
    path::Path,
    collections::HashSet,
    marker::PhantomData,
};
use ocl::{self, prm, builders::KernelBuilder};
use ocl_include::{Hook, MemHook, ListHook};
use crate::{
    Push, Store,
    Scene, View,
    Context, Program,
    RenderBuffer,
};


pub struct RendererBuilder<S: Scene, V: View> {
    list_hook: ListHook,
    phantom: PhantomData<(S, V)>,
}

pub struct Renderer<S: Scene, V: View> {
    program: Program,
    dims: (usize, usize),
    pub scene: S,
    pub view: V,
}

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

    pub fn builder() -> RendererBuilder<S, V> {
        RendererBuilder {
            list_hook:
                ListHook::builder()
                .add_hook(crate::source())
                .build(),
            phantom: PhantomData,
        }
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

impl<S: Scene, V: View> RendererBuilder<S, V> {
    pub fn add_hook<H: Hook + 'static>(&mut self, hook: H) {
        self.list_hook.add_hook(hook);
    }

    pub fn build(
        self, dims: (usize, usize),
        scene: S, view: V,
    ) -> crate::Result<Renderer<S, V>> {
        Renderer::<S, V>::new(
            dims, scene, view,
            self.list_hook,
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
        kb.arg(None::<&ocl::Buffer<prm::Float3>>); // color buffer
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

pub struct RenderWorker<S: Scene, V: View> {
    data: RenderData<S, V>,
    kernel: ocl::Kernel,
    context: Context,
}

impl<S: Scene, V: View> RenderWorker<S, V> {
    pub fn new(
        context: &Context,
        program: &Program,
        data: RenderData<S, V>,
    ) -> crate::Result<(Self, String)> {
        let queue = context.queue().clone();

        let (ocl_prog, message) = program.build(context)?;

        let mut kb = ocl::Kernel::builder();
        kb.program(&ocl_prog)
        .name("render")
        .queue(queue.clone());
        RenderData::<S, V>::args_def(&mut kb);
        
        let kernel = kb.build()?;

        Ok((RenderWorker {
            data, kernel,
            context: context.clone(),
        }, message))
    }

    pub fn data(&self) -> &RenderData<S, V> {
        &self.data
    }
    pub fn data_mut(&mut self) -> &mut RenderData<S, V> {
        &mut self.data
    }

    pub fn run(&mut self) -> crate::Result<()> {
        self.data.args_set(0, &mut self.kernel)?;
        unsafe {
            self.kernel.cmd()
            .global_work_size(self.data.screen.dims())
            .enq()?;
        }
        self.context.queue().finish()?;
        self.data_mut().screen.pass();

        Ok(())
    }
}
