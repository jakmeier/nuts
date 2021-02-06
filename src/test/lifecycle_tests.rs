use super::*;

#[test]
fn active_inactive() {
    let a = TestActivity::new();
    let counter = a.shared_counter_ref();
    // Start as not active
    let id = crate::new_activity(a);
    id.set_status(LifecycleStatus::Inactive);

    // Register for active only
    id.subscribe(|activity: &mut TestActivity, _msg: &TestUpdateMsg| {
        activity.inc(1);
    });

    crate::publish(TestUpdateMsg);
    assert_eq!(counter.get(), 0, "Called inactive activity");
    id.set_status(LifecycleStatus::Active);
    crate::publish(TestUpdateMsg);
    assert_eq!(counter.get(), 1, "Activation for activity didn't work");
}

#[test]
fn enter_leave() {
    let a = TestActivity::new();
    let counter = a.shared_counter_ref();
    let id = crate::new_activity(a);

    id.on_enter(|activity: &mut TestActivity| {
        activity.inc(1);
    });
    id.on_leave(|activity: &mut TestActivity| {
        activity.inc(10);
    });

    crate::publish(TestUpdateMsg);
    assert_eq!(counter.get(), 0, "Called enter/leave without status change");

    id.set_status(LifecycleStatus::Inactive);
    assert_eq!(counter.get(), 10);

    id.set_status(LifecycleStatus::Active);
    assert_eq!(counter.get(), 11);
}

#[test]
fn set_status_inside_publish() {
    let a = TestActivity::new();
    let counter = a.shared_counter_ref();
    let id = crate::new_activity(a);
    id.subscribe(move |activity, _msg: &TestUpdateMsg| {
        activity.inc(1);
        id.set_status(LifecycleStatus::Inactive);
    });
    crate::publish(TestUpdateMsg);
    crate::publish(TestUpdateMsg);
    crate::publish(TestUpdateMsg);
    assert_eq!(1, counter.get());
}

#[test]
fn on_delete() {
    let a = TestActivity::new();
    let counter = a.shared_counter_ref();
    let id = crate::new_activity(a);
    id.on_delete(|a| a.inc(1));
    assert_eq!(0, counter.get());

    id.set_status(LifecycleStatus::Deleted);
    assert_eq!(1, counter.get());
}

#[test]
fn on_delete_domained() {
    let a = TestActivity::new();
    let counter = a.shared_counter_ref();
    let d = TestDomains::DomainA;
    crate::store_to_domain(&d, 7usize);

    let id = crate::new_domained_activity(a, &d);
    id.on_delete_domained(|a, domain| {
        let x: usize = *domain.get();
        assert_eq!(7, x);
        a.inc(1)
    });
    assert_eq!(0, counter.get()); // Make sure subscription has not been called, yet

    id.set_status(LifecycleStatus::Deleted);

    assert_eq!(1, counter.get()); // Make sure subscription has been called
}

#[test]
fn delete_without_handler() {
    let aid = crate::new_activity(());
    aid.set_status(LifecycleStatus::Deleted);
}

#[test]
fn delete_twice() {
    let aid = crate::new_activity(());
    aid.set_status(LifecycleStatus::Deleted);
    aid.set_status(LifecycleStatus::Deleted);
}

#[test]
#[should_panic]
fn activate_after_delete() {
    let a = TestActivity::new();
    let d = TestDomains::DomainA;
    let id = crate::new_domained_activity(a, &d);
    id.set_status(LifecycleStatus::Deleted);
    id.set_status(LifecycleStatus::Active);
}

#[test]
// Subscription handlers should not be called after an activity has been deleted
fn call_subscription_after_delete() {
    let a = TestActivity::new();
    let counter = a.shared_counter_ref();
    let d = TestDomains::DomainA;

    let id = crate::new_domained_activity(a, &d);
    id.subscribe_domained(|a, _domain, _msg: &TestMessage| a.inc(1));
    assert_eq!(0, counter.get()); // Make sure subscription has not been called, yet

    // Make sure subscription has ben registered properly
    crate::publish(TestMessage(0));
    assert_eq!(1, counter.get());
    // Make sure subscription is no longer called
    id.set_status(LifecycleStatus::Deleted);
    crate::publish(TestMessage(0));
    assert_eq!(1, counter.get());
}

#[test]
// Subscription handlers should not be called after an activity has been deleted, regardless of it being executed deferred or not
fn deferred_delete_with_hanging_subscription() {
    let outer = crate::new_activity(());
    let a = TestActivity::new();
    let counter = a.shared_counter_ref();
    let counter_clone = counter.clone();
    let id = crate::new_activity(a);
    id.subscribe(|a, msg: &TestMessage| a.inc(msg.0));
    outer.subscribe(move |_, &()| {
        assert_eq!(0, counter_clone.get());
        // This should be called but deferred
        crate::publish(TestMessage(1));
        assert_eq!(0, counter_clone.get());

        id.set_status(LifecycleStatus::Deleted);

        // This should not be called at all
        crate::publish(TestMessage(10));
        assert_eq!(0, counter_clone.get());
    });
    assert_eq!(0, counter.get());
    crate::publish(());
    assert_eq!(1, counter.get());
}

#[test]
// Subscriptions to a deleted activity should be ignored
fn subscribe_after_delete() {
    let a = TestActivity::new();
    let counter = a.shared_counter_ref();
    let d = TestDomains::DomainA;

    let id = crate::new_domained_activity(a, &d);
    id.set_status(LifecycleStatus::Deleted);

    id.subscribe_domained(|a, _domain, _msg: &TestMessage| a.inc(1));
    assert_eq!(0, counter.get()); // Make sure subscription has not been called, yet
                                  // Make sure subscription is not called
    crate::publish(TestMessage(0));
    assert_eq!(0, counter.get());
}

#[test]
fn delete_with_on_leave() {
    let a = TestActivity::new();
    let counter = a.shared_counter_ref();
    let d = TestDomains::DomainA;

    let id = crate::new_domained_activity(a, &d);
    id.on_leave_domained(|a, _domain| a.inc(1));

    assert_eq!(0, counter.get());
    id.set_status(LifecycleStatus::Deleted);
    assert_eq!(1, counter.get());
}

#[test]
fn create_new_after_delete() {
    let a = TestActivity::new();
    let id_a = crate::new_activity(a);
    id_a.set_status(LifecycleStatus::Deleted);
    let num_a: UncheckedActivityId = id_a.into();

    let b = (TestActivity::new(),);
    let id_b = crate::new_activity(b);
    let num_b: UncheckedActivityId = id_b.into();

    assert_ne!(num_a, num_b);
}
