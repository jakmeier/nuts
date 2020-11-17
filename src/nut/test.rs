mod base_tests;
mod capsule_tests;
mod domain_tests;
mod inchoate_tests;
mod lifecycle_tests;

use crate::*;
use std::cell::Cell;
use std::rc::Rc;

#[derive(Clone)]
struct TestActivity {
    counter: Rc<Cell<u32>>,
}

impl TestActivity {
    fn new() -> Self {
        let shared_counter = Rc::new(Cell::new(0));
        Self {
            counter: shared_counter,
        }
    }
    fn shared_counter_ref(&self) -> Rc<Cell<u32>> {
        self.counter.clone()
    }
    fn inc(&self, add: u32) {
        let i = self.counter.get();
        self.counter.as_ref().set(i + add)
    }
}

#[derive(Clone, Copy)]
enum TestDomains {
    DomainA,
    _DomainB,
}
domain_enum!(TestDomains);

struct TestUpdateMsg;
struct TestForInt(usize);
struct TestMessage(u32);
struct TestMessageNoClone;
