use crate::nut::Handler;
use std::any::Any;
use std::collections::HashMap;
use std::ops::{Index, IndexMut};

pub trait Activity: Any {}

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub struct ActivityId<A> {
    pub(crate) index: usize,
    phantom: std::marker::PhantomData<A>,
}

/// A collection of heterogenous Activities
///
/// Needs stores a list of dynamic `Any` trait objects, not `Activity` because
/// trait objects only allow access to methods of that trait, not their super-traits.  
#[derive(Default)]
pub(crate) struct ActivityContainer {
    data: Vec<Option<Box<dyn Any>>>,
    active: Vec<bool>,
}

/// Handlers stored per Activity
#[derive(Default)]
pub(crate) struct ActivityHandlerContainer {
    data: HashMap<usize, Vec<Handler>>,
}

impl<A: Activity> ActivityId<A> {
    pub fn new(index: usize) -> Self {
        Self {
            index,
            phantom: Default::default(),
        }
    }
}

// TODO?: could be a trait
impl ActivityContainer {
    pub fn add<A: Activity>(&mut self, a: A, start_active: bool) -> ActivityId<A> {
        let i = self.data.len();
        self.data.push(Some(Box::new(a)));
        self.active.push(start_active);
        ActivityId::new(i)
    }
    pub fn is_active<A: Activity>(&self, id: ActivityId<A>) -> bool {
        self.active[id.index]
    }
    pub fn set_active<A: Activity>(&mut self, id: ActivityId<A>, active: bool) {
        self.active[id.index] = active
    }
}

impl<A: Activity> Index<ActivityId<A>> for ActivityContainer {
    type Output = dyn Any;
    fn index(&self, id: ActivityId<A>) -> &Self::Output {
        self.data[id.index]
            .as_ref()
            .expect("Missing activity")
            .as_ref()
    }
}
impl<A: Activity> IndexMut<ActivityId<A>> for ActivityContainer {
    fn index_mut(&mut self, id: ActivityId<A>) -> &mut Self::Output {
        self.data[id.index]
            .as_mut()
            .expect("Missing activity")
            .as_mut()
    }
}

impl ActivityHandlerContainer {
    pub fn iter_for<A: Activity>(&self, id: ActivityId<A>) -> impl Iterator<Item = &Handler> {
        self.data.get(&id.index).into_iter().flat_map(|f| f.iter())
    }
}
impl<A: Activity> Index<ActivityId<A>> for ActivityHandlerContainer {
    type Output = Vec<Handler>;
    fn index(&self, id: ActivityId<A>) -> &Self::Output {
        &self.data[&id.index]
    }
}
impl<A: Activity> IndexMut<ActivityId<A>> for ActivityHandlerContainer {
    fn index_mut(&mut self, id: ActivityId<A>) -> &mut Self::Output {
        self.data.entry(id.index).or_insert(Default::default())
    }
}

impl<A> Copy for ActivityId<A> {}
impl<A> Clone for ActivityId<A> {
    fn clone(&self) -> Self {
        *self
    }
}
