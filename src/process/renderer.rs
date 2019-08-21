use std::{
    path::Path,
    collections::HashSet,
    marker::PhantomData,
};
use ocl::{self, prm, builders::KernelBuilder};
use ocl_include::{Hook, MemHook, ListHook};
use crate::{
    Push,
    Scene, View,
    Context, Program,
    Customer,
    RenderBuffer,
};


pub struct RendererBuilder<S: Scene, V: View> {
    hook: ListHook,
    phantom: PhantomData<(S, V)>,
}

pub struct Renderer<S: Scene, V: View> {
    program: Program,
    dims: (usize, usize),
    pub scene: S,
    pub view: V,
}

pub struct RendererData<S: Scene, V: View> {
    screen: RenderBuffer,
    scene_data: S::Data,
    view_data: V::Data,
}

impl<S: Scene, V: View> Renderer<S, V> {
    pub fn new<H: Hook + 'static>(
        hook: H,
        dims: (usize, usize),
        scene: S, view: V,
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
            hook:
                ListHook::builder()
                .add_hook(crate::source())
                .build(),
            phantom: PhantomData,
        }
    }

    pub fn program(&self) -> &Program {
        &self.program
    }
}

impl<S: Scene, V: View> RendererBuilder<S, V> {
    pub fn add_hook<H: Hook + 'static>(&mut self, hook: H) {
        self.hook.add_hook(hook);
    }

    pub fn build(
        self, dims: (usize, usize),
        scene: S, view: V,
    ) -> crate::Result<Renderer<S, V>> {
        Renderer::<S, V>::new(
            self.hook,
            dims, scene, view,
        )
    }
}

impl<S: Scene, V: View> Customer for Renderer<S, V> {
    type Data = RendererData<S, V>;

    fn new_data(&self, context: &Context) -> Self::Data {

    }

    fn update_data(&self, context: &Context, data: &mut Self::Data) -> crate::Result<()> {
        Ok(())
    }
}

impl<S: Scene, V: View> Push for RendererData<S, V> {
    fn args_def(kb: &mut KernelBuilder) {
        kb.arg(prm::Int2::zero()); // screen size
        kb.arg(None::<&ocl::Buffer<prm::Float3>>); // color buffer
        kb.arg(None::<&ocl::Buffer<u32>>); // random
        S::args_def(kb);
        V::args_def(kb);
    }
    fn args_set(&self, i: usize, k: &mut ocl::Kernel) -> crate::Result<()> {
        let mut j = i;

        let dims = screen.dims();
        let dims = prm::Int2::new(dims.0 as i32, dims.1 as i32);
        self.kernel.set_arg(i + 0, &dims)?;
        self.kernel.set_arg(i + 1, screen.color_mut())?;
        self.kernel.set_arg(i + 2, screen.random_mut())?;
        j += 3;

        view.args_set(j, &mut self.kernel)?;
        j += V::args_count();

        scene.args_set(j, &mut self.kernel)?;
        //j += S::args_count();

        k.set_default_global_work_size(screen.dims());
    }
    fn args_count() -> usize {
        3 + S::args_count() + V::args_count()
    }
}
