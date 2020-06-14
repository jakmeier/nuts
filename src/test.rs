use crate::*;
use std::cell::Cell;
use std::rc::Rc;

struct TestActivity {
    counter: Rc<Cell<u32>>,
}

impl Activity for TestActivity {}

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

#[test]
// A simple sanity test for registering an activity.
// The registered function should crucially only be called once.
// The test should be considered in combination with `closure_registration_negative`
fn closure_registration() {
    let a = TestActivity::new();
    let counter = a.shared_counter_ref();
    let id = crate::new_activity(a, true);
    crate::subscribe(id, Topic::update(), |activity: &mut TestActivity| {
        activity.inc(1);
    });
    assert_eq!(counter.get(), 0, "Closure called before update call");
    crate::update();
    assert_eq!(counter.get(), 1);
    crate::update();
    assert_eq!(counter.get(), 2);
}

#[test]
fn active_inactive() {
    let a = TestActivity::new();
    let counter = a.shared_counter_ref();
    // Start as not active
    let id = crate::new_activity(a, false);

    // Register for active only
    crate::subscribe(id, Topic::update(), |activity: &mut TestActivity| {
        activity.inc(1);
    });

    crate::update();
    assert_eq!(counter.get(), 0, "Called inactive activity");
    crate::set_active(id, true);
    crate::update();
    assert_eq!(counter.get(), 1, "Activation for activity didn't work");
}

#[test]
fn enter_leave() {
    let a = TestActivity::new();
    let counter = a.shared_counter_ref();
    let id = crate::new_activity(a, true);

    crate::subscribe(id, Topic::enter(), |activity: &mut TestActivity| {
        activity.inc(1);
    });
    crate::subscribe(id, Topic::leave(), |activity: &mut TestActivity| {
        activity.inc(10);
    });

    crate::update();
    assert_eq!(counter.get(), 0, "Called enter/leave without status change");

    crate::set_active(id, false);
    assert_eq!(counter.get(), 10);

    crate::set_active(id, true);
    assert_eq!(counter.get(), 11);
}
