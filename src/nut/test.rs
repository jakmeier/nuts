use crate::*;
use std::cell::Cell;
use std::rc::Rc;

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

#[test]
// A simple sanity test for registering an activity.
// The registered function should crucially only be called once.
// The test should be considered in combination with `closure_registration_negative`
fn closure_registration() {
    let a = TestActivity::new();
    let counter = a.shared_counter_ref();
    let id = crate::new_activity(a, true);
    id.subscribe(|activity: &mut TestActivity, _: &TestUpdateMsg| {
        activity.inc(1);
    });
    assert_eq!(counter.get(), 0, "Closure called before update call");
    crate::publish(TestUpdateMsg);
    assert_eq!(counter.get(), 1);
    crate::publish(TestUpdateMsg);
    assert_eq!(counter.get(), 2);
}

#[test]
fn active_inactive() {
    let a = TestActivity::new();
    let counter = a.shared_counter_ref();
    // Start as not active
    let id = crate::new_activity(a, false);

    // Register for active only
    id.subscribe(|activity: &mut TestActivity, _msg: &TestUpdateMsg| {
        activity.inc(1);
    });

    crate::publish(TestUpdateMsg);
    assert_eq!(counter.get(), 0, "Called inactive activity");
    crate::set_active(id, true);
    crate::publish(TestUpdateMsg);
    assert_eq!(counter.get(), 1, "Activation for activity didn't work");
}

#[test]
fn enter_leave() {
    let a = TestActivity::new();
    let counter = a.shared_counter_ref();
    let id = crate::new_activity(a, true);

    id.on_enter(|activity: &mut TestActivity| {
        activity.inc(1);
    });
    id.on_leave(|activity: &mut TestActivity| {
        activity.inc(10);
    });

    crate::publish(TestUpdateMsg);
    assert_eq!(counter.get(), 0, "Called enter/leave without status change");

    crate::set_active(id, false);
    assert_eq!(counter.get(), 10);

    crate::set_active(id, true);
    assert_eq!(counter.get(), 11);
}

#[test]
fn domained_activity() {
    let a = TestActivity::new();
    let d = TestDomains::DomainA;
    crate::store_to_domain(&d, 7usize);
    let id = crate::new_domained_activity(a, &d, true);
    id.subscribe_domained(|_activity, domain, _msg: &TestUpdateMsg| {
        let x: usize = *domain.get();
        assert_eq!(7, x);
    });
    crate::publish(TestUpdateMsg);
}

struct TestMessage(u32);
#[test]
fn message_passing() {
    // Set up activity that increases a counter by the value specified in messages of type TestMessage
    let a = TestActivity::new();
    let counter = a.shared_counter_ref();
    let id = crate::new_activity(a, true);
    id.subscribe(|activity, msg: &TestMessage| {
        activity.inc(msg.0);
    });

    // Send different values and check that subscribed code has been called
    crate::publish(TestMessage(13));
    assert_eq!(counter.get(), 13);

    crate::publish(TestMessage(13));
    assert_eq!(counter.get(), 26);

    crate::publish(TestMessage(100));
    assert_eq!(counter.get(), 126);
}

#[test]
fn publish_inside_publish() {
    const LAYERS: u32 = 5;

    let a = TestActivity::new();
    let counter = a.shared_counter_ref();
    let id = crate::new_activity(a, true);
    id.subscribe(|activity, _msg: &TestUpdateMsg| {
        if activity.counter.get() < LAYERS {
            activity.inc(1);
            crate::publish(TestUpdateMsg);
        }
    });
    crate::publish(TestUpdateMsg);

    assert_eq!(LAYERS, counter.get());
}

#[test]
fn set_active_inside_publish() {
    let a = TestActivity::new();
    let counter = a.shared_counter_ref();
    let id = crate::new_activity(a, true);
    id.subscribe(move |activity, _msg: &TestUpdateMsg| {
        activity.inc(1);
        crate::set_active(id, false);
    });
    crate::publish(TestUpdateMsg);
    crate::publish(TestUpdateMsg);
    crate::publish(TestUpdateMsg);
    assert_eq!(1, counter.get());
}

struct TestMessageNoClone;
#[test]
fn owned_message() {
    let a = TestActivity::new();
    let counter = a.shared_counter_ref();
    let id = crate::new_activity(a, true);
    id.subscribe_owned(|activity, _msg: TestMessageNoClone| {
        activity.inc(1);
    });
    crate::publish(TestMessageNoClone);
    assert_eq!(1, counter.get()); // Make sure subscription has been called
}
#[test]
fn owned_domained_message() {
    let a = TestActivity::new();
    let counter = a.shared_counter_ref();
    let d = TestDomains::DomainA;
    crate::store_to_domain(&d, 7usize);
    let id = crate::new_domained_activity(a, &d, true);
    id.subscribe_domained_owned(|activity, domain, _msg: TestMessageNoClone| {
        let x: usize = *domain.get();
        assert_eq!(7, x);
        activity.inc(1);
    });
    crate::publish(TestMessageNoClone);
    assert_eq!(1, counter.get()); // Make sure subscription has been called
}
