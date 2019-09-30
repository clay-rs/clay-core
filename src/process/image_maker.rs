use std::{
    path::Path,
};
use ocl::{self, prm, builders::KernelBuilder};
use crate::{
    prelude::*,
    Context,
    process::{Program, Kernel, KernelSubtype},
    buffer::{DoubleBuffer, Image},
};


pub struct ImageMaker {
    kernel: Kernel<WrapStatic>,
}

struct WrapStatic {}
struct Wrap<'a, 'b, 'c> {
    context: &'c Context,
    dims: (usize, usize),
    db: &'b mut DoubleBuffer<f32>,
    img: &'a Image,
}
impl<'a, 'b, 'c> KernelSubtype<WrapStatic> for Wrap<'a, 'b, 'c> {}

impl Push for WrapStatic {
    fn args_count() -> usize {
        5
    }
    fn args_def(kb: &mut KernelBuilder) {
        kb.arg(prm::Int2::zero()); // screen size
        kb.arg(0i32); // dst passes
        kb.arg(0i32); // src passes
        kb.arg(None::<&ocl::Buffer<f32>>); // dst buffer
        kb.arg(None::<&ocl::Buffer<f32>>); // src buffer
    }
    fn args_set(&mut self, i: usize, k: &mut ocl::Kernel) -> crate::Result<()> {
        unreachable!()
    }
}
impl<'a, 'b, 'c> Push for Wrap<'a, 'b, 'c> {
    fn args_count() -> usize {
        WrapStatic::args_count()
    }
    fn args_def(kb: &mut KernelBuilder) {
        WrapStatic::args_def(kb)
    }
    fn args_set(&mut self, i: usize, k: &mut ocl::Kernel) -> crate::Result<()> {
        let dims = prm::Int2::new(self.dims.0 as i32, self.dims.1 as i32);
        k.set_arg(i + 0, &dims)?;
        k.set_arg(i + 1, &(self.n_passes as i32))?;
        k.set_arg(i + 2, &(self.rb.n_passes() as i32))?;
        k.set_arg(i + 3, self.db.dst_buffer_mut())?;
        if *self.rb.context() != *self.context {
            k.set_arg(i + 4, self.db.src_buffer())?;
        } else {
            k.set_arg(i + 4, self.rb.color())?;
        }
    }
}

impl Maker {
    pub fn new(context: &Context) -> crate::Result<Self> {
        let program = Program::new(&crate::ocl_src(), &Path::from("clay_core/"))?;
        let kernel = Kernel::new(context, &program)?.0;
        Ok(Self { kernel })
    }

    pub fn accumulate<'a, I: Iterator<Item=&'a RenderBuffer>>(
        &mut self, db: DoubleBuffer<f32>, rbs: I,
    ) -> crate::Result<()> {
        let dims = prm::Int2::new(db.dims().0 as i32, db.dims().1 as i32);
        let mut n_passes = 0;
        for rb in rbs {
            let wrap = AccumWrap {
                context: self.kernel.context(),
                dims: db.dims(),
                n_passes,
                db, rb,
            };
            self.kernel.run(dims, &wrap)?;
            n_passes += rb.n_passes();
        }
    }
}
