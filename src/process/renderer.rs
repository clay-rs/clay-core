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
    list_hook: ListHook,
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

impl<S: Scene, V: View> Customer for Renderer<S, V> {
    type Data = RendererData<S, V>;

    fn new_data(&self, context: &Context) -> crate::Result<Self::Data> {
        Ok(Self::Data {
            screen: RenderBuffer::new(context, self.dims)?,
            scene_data: self.scene.new_data(context)?,
            view_data: self.view.new_data(context)?,
        })
    }

    fn update_data(&self, context: &Context, data: &mut Self::Data) -> crate::Result<()> {
        self.scene.update_data(context, &mut data.scene_data)?;
        self.view.update_data(context, &mut data.view_data)?;
        Ok(())
    }
}

impl<S: Scene, V: View> Push for RendererData<S, V> {
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

        k.set_default_global_work_size(dims.into());
        Ok(())
    }
}
