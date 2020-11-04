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
    assert_eq!(0, counter.get()); // Make sure subscription has been called

    id.set_status(LifecycleStatus::Deleted);

    assert_eq!(1, counter.get()); // Make sure subscription has been called
}
