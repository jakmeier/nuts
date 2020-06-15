//! Managed State
//!
//! Objects to which multiple activities have access

use std::any::{Any, TypeId};
use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct ManagedState {
    domains: Vec<DomainState>,
    // TODO: something like
    // HashMap<domain_id, Box<dyn DomainObj>>

    // Idea to consider: In paddle, there could be a Shred frame
    // that matches domains to shred::world and activities that
    // implement shred::System. This could make a shred system an activity
    // which can be dispatched in the paddle layer
    // In here, I just want to make sure this kind of thing is possible
    // but not add too much specific features

    // More importantly, the graphics state needs to be accessible somehow...
    // It could be linked to acitivities that have draw, or maybe (also) to a specific domain
    // Or maybe the drawing state should be outside of webnut, just scheduling is treated specially for drawing in here
    // (This is justified because it is relevant to thread assignment in the browser)
}

#[derive(Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub(crate) struct DomainId(Option<usize>);

pub trait DomainEnumeration {
    fn id(&self) -> usize;
}

#[derive(Default)]
pub struct DomainState {
    objects: HashMap<TypeId, Box<dyn Any>>,
}

impl DomainId {
    pub(crate) fn new(d: impl DomainEnumeration) -> DomainId {
        DomainId(Some(d.id()))
    }
}

impl ManagedState {
    pub(crate) fn get_mut(&mut self, id: DomainId) -> Option<&mut DomainState> {
        id.0.map(move |i| &mut self.domains[i])
    }
    pub(crate) fn prepare(&mut self, id: DomainId) {
        if let Some(n) = id.0 {
            while self.domains.len() < n {
                self.domains.push(Default::default());
            }
        }
    }
}

impl DomainState {
    // TODO: Some way to add and borrow data
    // pub fn get<T: Any>(&self) -> &T {
    //     self.objects
    //         .get(&TypeId::of::<T>())
    //         .map(|obj| obj.downcast().as_ref().unwrap())
    //         .expect("Not in domain")
    // }
}
