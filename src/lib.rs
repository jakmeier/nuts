mod activity;
pub use activity::*;

#[cfg(test)]
mod test;

use crate::Activity;
use std::any::Any;
use std::cell::RefCell;

thread_local!(static NUT: RefCell<Nut> = RefCell::new(Nut::new()));
struct Nut {
    // XXX: Vec is not right, should be append only Vec (because index are used)
    activities: ActivityContainer,
    updates: Vec<Handler>,
}

// pub type Handler<ACTIVITY> = Box<dyn Fn(&mut ACTIVITY) -> ()>;
// type ActivityContainer = Vec<Box<dyn Activity>>;
type ActivityContainer = Vec<Box<dyn Any>>; // XXX: Somehow, this only works with Any and not with Acticity
type Handler = Box<dyn Fn(&mut ActivityContainer)>;

impl Nut {
    fn new() -> Self {
        Self {
            activities: Vec::new(),
            updates: Vec::new(),
        }
    }
    fn update(&mut self) {
        for f in &self.updates {
            f(&mut self.activities);
        }
    }
}

pub fn activity<A>(activity: A) -> ActivityId
where
    A: Activity,
{
    NUT.with(|nut| {
        let mut nut = nut.borrow_mut();
        nut.activities.push(Box::new(activity));
        ActivityId::new::<A>(nut.activities.len() - 1)
    })
}
pub fn register<A, F>(id: ActivityId, f: F)
where
    A: Activity,
    F: Fn(&mut A) + 'static,
{
    NUT.with(|nut| {
        let mut nut = nut.borrow_mut();
        let index = id.index;
        nut.updates
            .push(Box::new(move |activities: &mut ActivityContainer| {
                let a = activities[index].as_mut().downcast_mut::<A>().unwrap();
                f(a);
            }));
    });
}

pub fn update() {
    NUT.with(|nut| nut.borrow_mut().update())
}
