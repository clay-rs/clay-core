use std::path::PathBuf;
use crate::PushDyn;


/// Filter performs arbitrary transformation of rendered picture.
pub trait Filter: PushDyn {
    fn source_file(&self) -> PathBuf;
}
