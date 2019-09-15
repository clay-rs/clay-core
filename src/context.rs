use ocl;


/// Context that represents separate processing entity (often the same as a device).
#[derive(Clone)]
pub struct Context {
    platform:  ocl::Platform,
    device:    ocl::Device,
    context:   ocl::Context,
    queue:     ocl::Queue,
}

impl PartialEq for Context {
    fn eq(&self, rhs: &Self) -> bool {
        self.queue.as_core().as_ptr() == rhs.queue.as_core().as_ptr()
    }
}

impl Context {
    pub fn new(platform: ocl::Platform, device: ocl::Device) -> crate::Result<Self> {
        let context = ocl::Context::builder()
        .platform(platform)
        .devices(device.clone())
        .build()?;

        let queue = ocl::Queue::new(&context, device, None)?;

        Ok(Self { platform, device, context, queue })
    }

    pub fn platform(&self) -> &ocl::Platform {
        &self.platform
    }
    pub fn device(&self) -> &ocl::Device {
        &self.device
    }
    pub fn context(&self) -> &ocl::Context {
        &self.context
    }
    pub fn queue(&self) -> &ocl::Queue {
        &self.queue
    }
}
