use crate::{Push, Context};


/// Something that tells worker what to do
pub trait Customer {
    /// Data that is stored on the device.
    type Data: Push;

    /// Creates device data.
    fn new_data(&self, context: &Context) -> crate::Result<Self::Data>;

    /// Updates device data.
    fn update_data(&self, context: &Context, data: &mut Self::Data) -> crate::Result<()>;
}
