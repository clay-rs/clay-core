use crate::{Source, Push};


/// Filter performs arbitrary transformation of rendered picture.
pub trait Filter: Source + Push {

}
