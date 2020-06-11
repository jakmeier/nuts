use std::any::Any;
use std::any::TypeId;

pub trait Activity: Any {}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub struct ActivityId {
    pub(crate) t: TypeId,
    pub(crate) index: usize,
}

impl ActivityId {
    pub fn new<A: Activity>(index: usize) -> Self {
        Self {
            t: TypeId::of::<A>(),
            index,
        }
    }
    
}
