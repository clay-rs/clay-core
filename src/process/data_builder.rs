use crate::{Push, Context};


/// Something that could build device-specific data.
pub trait DataBuilder {
    /// Data that is stored on the device.
    type Data: Push;

    /// Creates device data.
    fn new_data(&self, context: &Context) -> crate::Result<Self::Data>;

    /// Updates device data.
    fn update_data(&self, context: &Context, data: &mut Self::Data) -> crate::Result<()>;
}
