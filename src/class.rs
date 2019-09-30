use crate::Source;


/// An interface in OpenCL code.
pub trait Class {
    /// Class name (e.g. `shape`)
    fn name() -> String;
    /// List of methods of the class.
    fn methods() -> Vec<String>;
}

/// An implementation of an interface in OpenCL.
pub trait Instance<C: Class>: Source + Sized + 'static {
    // Class of an instance.
    //type Class: Class = C;
    
    /// Name of the instance of the class (e.g. `sphere` as instance of class `shape`).
    fn inst_name() -> String;
}
