/// Something that could be packed to a pair of i32 and f32 buffers
pub trait Pack {
    /// Size of integer part of an object.
    fn size_int() -> usize;
    /// Size of float part of an object.
    fn size_float() -> usize;

    /// Write an object into int and float buffers.
    ///
    /// Buffers *must* be of size greater or equal to object's one.
    fn pack(&self, buffer_int: &mut [i32], buffer_float: &mut [f32]);

    /// Read an object from int and float buffers.
    ///
    /// Buffers *must* be of size greater or equal to object's one.
    fn unpack(buffer_int: &[i32], buffer_float: &[f32]) -> Self;
}
