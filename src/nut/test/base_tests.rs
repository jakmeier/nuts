//! Test suite for activity creation and subscription registration.

use super::*;
#[test]
// A simple sanity test for registering an activity.
// The registered function should crucially only be called once.
// The test should be considered in combination with `closure_registration_negative`
fn closure_registration() {
    let a = TestActivity::new();
    let counter = a.shared_counter_ref();
    let id = crate::new_activity(a);
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
fn domained_activity() {
    let a = TestActivity::new();
    let d = TestDomains::DomainA;
    crate::store_to_domain(&d, 7usize);
    let id = crate::new_domained_activity(a, &d);
    id.subscribe_domained(|_activity, domain, _msg: &TestUpdateMsg| {
        let x: usize = *domain.get();
        assert_eq!(7, x);
    });
    crate::publish(TestUpdateMsg);
}

#[test]
fn message_passing() {
    // Set up activity that increases a counter by the value specified in messages of type TestMessage
    let a = TestActivity::new();
    let counter = a.shared_counter_ref();
    let id = crate::new_activity(a);
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
fn owned_message() {
    let a = TestActivity::new();
    let counter = a.shared_counter_ref();
    let id = crate::new_activity(a);
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
    let id = crate::new_domained_activity(a, &d);
    id.subscribe_domained_owned(|activity, domain, _msg: TestMessageNoClone| {
        let x: usize = *domain.get();
        assert_eq!(7, x);
        activity.inc(1);
    });
    crate::publish(TestMessageNoClone);
    assert_eq!(1, counter.get()); // Make sure subscription has been called
}

#[test]
fn publish_inside_publish() {
    const LAYERS: u32 = 5;

    let a = TestActivity::new();
    let counter = a.shared_counter_ref();
    let id = crate::new_activity(a);
    id.subscribe(|activity, _msg: &TestUpdateMsg| {
        if activity.counter.get() < LAYERS {
            activity.inc(1);
            crate::publish(TestUpdateMsg);
        }
    });
    crate::publish(TestUpdateMsg);

    assert_eq!(LAYERS, counter.get());
}
